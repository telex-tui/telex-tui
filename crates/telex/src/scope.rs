use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::async_state::{Async, AsyncHandle};
use crate::channel::{ChannelDrain, ChannelHandle, IntervalHandle, PortHandle};
use crate::command::{CommandRegistry, KeyBinding};
use crate::context::ContextStorage;
use crate::state::State;
use crate::stream_state::{StreamHandle, TextStreamHandle};

/// Type alias for effect cleanup functions.
type CleanupFn = Box<dyn FnOnce()>;

/// Type alias for effect functions that return an optional cleanup.
type EffectFn = Box<dyn FnOnce() -> Option<CleanupFn>>;

/// State for a single effect hook.
struct EffectState {
    /// Cleanup function from the last effect run, if any.
    cleanup: Option<CleanupFn>,
    /// Dependencies from last run (boxed for type erasure).
    last_deps: Option<Box<dyn Any>>,
    /// Whether effect has ever run.
    initialized: bool,
}

/// A pending keyed effect to run after render.
struct PendingKeyedEffect {
    /// TypeId key for the effect.
    key: TypeId,
    /// The effect function that returns an optional cleanup.
    effect_fn: EffectFn,
    /// New dependencies to store after running.
    new_deps: Option<Box<dyn Any>>,
}

/// Maximum effect executions allowed within a window before we assume infinite loop.
/// This is generous enough for legitimate use cases but catches runaway effects.
const MAX_EFFECT_RUNS_PER_WINDOW: usize = 100;

/// Number of frames in the sliding window for effect cycle detection.
const EFFECT_WINDOW_FRAMES: usize = 10;

/// Storage for component state across re-renders.
pub struct StateStorage {
    /// TypeId-keyed state storage (order-independent)
    keyed_states: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    /// TypeId-keyed effect storage (order-independent)
    keyed_effects: RefCell<HashMap<TypeId, EffectState>>,
    /// Keyed effects scheduled to run after render
    pending_keyed_effects: RefCell<Vec<PendingKeyedEffect>>,
    /// Rolling count of effect executions for cycle detection
    effect_run_count: RefCell<usize>,
    /// Frames since last counter decay
    frames_since_decay: RefCell<usize>,
    /// Registered channels for the run loop to drain each frame
    channels: RefCell<Vec<Rc<dyn ChannelDrain>>>,
    /// Wake flag shared with channel senders for low-latency polling
    wake_flag: Arc<AtomicBool>,
}

impl Default for StateStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StateStorage {
    pub fn new() -> Self {
        Self {
            keyed_states: RefCell::new(HashMap::new()),
            keyed_effects: RefCell::new(HashMap::new()),
            pending_keyed_effects: RefCell::new(Vec::new()),
            effect_run_count: RefCell::new(0),
            frames_since_decay: RefCell::new(0),
            channels: RefCell::new(Vec::new()),
            wake_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get the wake flag (shared with channel senders for low-latency polling).
    pub fn wake_flag(&self) -> &Arc<AtomicBool> {
        &self.wake_flag
    }

    /// Get or create state by TypeId key (order-independent).
    ///
    /// The type K acts as the key - same K always returns the same state.
    pub fn use_state_keyed<K: 'static, T: 'static>(&self, init: impl FnOnce() -> T) -> State<T> {
        let key = TypeId::of::<K>();
        let mut keyed_states = self.keyed_states.borrow_mut();

        if let Some(any) = keyed_states.get(&key) {
            // State exists, retrieve it
            any.downcast_ref::<State<T>>()
                .expect("State type mismatch for keyed state")
                .clone()
        } else {
            // First access, create new state
            let state = State::new(init());
            keyed_states.insert(key, Rc::new(state.clone()));
            state
        }
    }

    // ========== Keyed Async/Stream/Terminal (order-independent) ==========

    /// Get or create async state by TypeId key (order-independent).
    pub fn use_async_keyed<K: 'static, T, F>(&self, f: F) -> Async<T>
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> Result<T, String> + Send + 'static,
    {
        let key = TypeId::of::<K>();
        let mut keyed_states = self.keyed_states.borrow_mut();

        let handle = if let Some(any) = keyed_states.get(&key) {
            any.downcast_ref::<AsyncHandle<T>>()
                .expect("Async type mismatch for keyed async state")
                .clone()
        } else {
            let handle = AsyncHandle::new();
            keyed_states.insert(key, Rc::new(handle.clone()));
            handle
        };

        drop(keyed_states);

        // Start the async operation if not already started
        handle.start(f);

        // Poll and return current state
        handle.poll()
    }

    /// Get or create stream state by TypeId key (order-independent).
    pub fn use_stream_keyed<K: 'static, T, F, I>(&self, stream_fn: F) -> StreamHandle<T>
    where
        T: Clone + Default + Send + 'static,
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = T> + Send + 'static,
    {
        let key = TypeId::of::<K>();
        let mut keyed_states = self.keyed_states.borrow_mut();

        let handle = if let Some(any) = keyed_states.get(&key) {
            any.downcast_ref::<StreamHandle<T>>()
                .expect("Stream type mismatch for keyed stream state")
                .clone()
        } else {
            let handle = StreamHandle::new();
            keyed_states.insert(key, Rc::new(handle.clone()));
            handle
        };

        drop(keyed_states);

        // Start the stream if not already started
        handle.start(stream_fn);

        // Poll for updates
        handle.poll(|acc, item| *acc = item);

        handle
    }

    /// Get or create text stream state by TypeId key (order-independent).
    pub fn use_text_stream_keyed<K: 'static, F, I>(&self, stream_fn: F) -> TextStreamHandle
    where
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = String> + Send + 'static,
    {
        self.use_text_stream_with_restart_keyed::<K, F, I>(false, stream_fn)
    }

    /// Get or create text stream state with restart support, by TypeId key (order-independent).
    pub fn use_text_stream_with_restart_keyed<K: 'static, F, I>(
        &self,
        restart: bool,
        stream_fn: F,
    ) -> TextStreamHandle
    where
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = String> + Send + 'static,
    {
        let key = TypeId::of::<K>();
        let mut keyed_states = self.keyed_states.borrow_mut();

        let handle = if let Some(any) = keyed_states.get(&key) {
            any.downcast_ref::<TextStreamHandle>()
                .expect("TextStream type mismatch for keyed text stream state")
                .clone()
        } else {
            let handle = TextStreamHandle::new();
            keyed_states.insert(key, Rc::new(handle.clone()));
            handle
        };

        drop(keyed_states);

        // Reset if requested (for starting a new stream)
        if restart {
            handle.reset();
        }

        // Start the stream if not already started
        handle.start(stream_fn);

        // Poll and accumulate text
        handle.poll_text();

        handle
    }

    /// Get or create a terminal handle by TypeId key (order-independent).
    pub fn use_terminal_keyed<K: 'static>(&self) -> crate::terminal_state::TerminalHandle {
        let key = TypeId::of::<K>();
        let mut keyed_states = self.keyed_states.borrow_mut();

        if let Some(any) = keyed_states.get(&key) {
            any.downcast_ref::<crate::terminal_state::TerminalHandle>()
                .expect("TerminalHandle type mismatch for keyed terminal state")
                .clone()
        } else {
            let handle = crate::terminal_state::TerminalHandle::new(24, 80);
            keyed_states.insert(key, Rc::new(handle.clone()));
            handle
        }
    }

    // ========== Reducer (order-independent) ==========

    /// Get or create a reducer by TypeId key.
    ///
    /// Returns `(state, dispatch)` where `dispatch` sends an action through
    /// the reducer to produce a new state.
    pub fn use_reducer_keyed<K: 'static, S: Clone + 'static, A: 'static>(
        &self,
        initial: S,
        reducer: impl Fn(S, A) -> S + 'static,
    ) -> (State<S>, Rc<dyn Fn(A)>) {
        let state = self.use_state_keyed::<K, S>(|| initial);
        let state_for_dispatch = state.clone();
        let reducer = Rc::new(reducer);
        let dispatch: Rc<dyn Fn(A)> = Rc::new(move |action: A| {
            let current = state_for_dispatch.get();
            let next = reducer(current, action);
            state_for_dispatch.set(next);
        });
        (state, dispatch)
    }

    // ========== Channels / Ports (order-independent) ==========

    /// Get or create a typed inbound channel by TypeId key.
    pub fn use_channel_keyed<K: 'static, T: 'static>(&self) -> ChannelHandle<T> {
        let key = TypeId::of::<K>();
        let mut keyed_states = self.keyed_states.borrow_mut();

        if let Some(any) = keyed_states.get(&key) {
            any.downcast_ref::<ChannelHandle<T>>()
                .expect("Channel type mismatch for keyed channel")
                .clone()
        } else {
            let handle = ChannelHandle::new(Arc::clone(&self.wake_flag));
            keyed_states.insert(key, Rc::new(handle.clone()));
            // Register for run-loop draining
            self.channels.borrow_mut().push(Rc::new(handle.clone()) as Rc<dyn ChannelDrain>);
            handle
        }
    }

    /// Get or create a bidirectional port by TypeId key.
    pub fn use_port_keyed<K: 'static, In: 'static, Out: 'static>(&self) -> PortHandle<In, Out> {
        let key = TypeId::of::<K>();
        let mut keyed_states = self.keyed_states.borrow_mut();

        if let Some(any) = keyed_states.get(&key) {
            any.downcast_ref::<PortHandle<In, Out>>()
                .expect("Port type mismatch for keyed port")
                .clone()
        } else {
            let handle = PortHandle::new(Arc::clone(&self.wake_flag));
            keyed_states.insert(key, Rc::new(handle.clone()));
            // Register the inbound channel for run-loop draining
            self.channels.borrow_mut().push(Rc::new(handle.rx.clone()) as Rc<dyn ChannelDrain>);
            handle
        }
    }

    /// Create or retrieve a periodic interval by TypeId key.
    ///
    /// Spawns a timer thread that fires at the given `duration`. The `callback`
    /// is called on the main thread each frame that one or more ticks arrived.
    pub fn use_interval_keyed<K: 'static>(
        &self,
        duration: std::time::Duration,
        callback: impl Fn() + 'static,
    ) {
        let key = TypeId::of::<K>();
        let keyed_states = self.keyed_states.borrow();

        if keyed_states.contains_key(&key) {
            // Already created — nothing to do. The timer thread is running
            // and the drain callback is registered.
            return;
        }
        drop(keyed_states);

        let handle = IntervalHandle::new(duration, Rc::new(callback), Arc::clone(&self.wake_flag));
        self.keyed_states
            .borrow_mut()
            .insert(key, Rc::new(handle.clone()));
        self.channels
            .borrow_mut()
            .push(Rc::new(handle) as Rc<dyn ChannelDrain>);
    }

    /// Drain all registered channels (called by the run loop each frame).
    pub fn drain_channels(&self) {
        let channels = self.channels.borrow();
        for ch in channels.iter() {
            ch.drain();
        }
    }

    /// Clear all channel frame buffers (called at start of each frame).
    pub fn clear_channels(&self) {
        let channels = self.channels.borrow();
        for ch in channels.iter() {
            ch.clear();
        }
    }

    /// Check if any channel has messages this frame.
    pub fn has_channel_data(&self) -> bool {
        let channels = self.channels.borrow();
        channels.iter().any(|ch| ch.has_messages())
    }

    // ========== Keyed Effects (order-independent) ==========

    /// Schedule a keyed effect to run only once (on first render).
    /// Order-independent - safe to use in conditionals.
    pub fn use_effect_once_keyed<K: 'static, F, C>(&self, effect_fn: F)
    where
        F: FnOnce() -> C + 'static,
        C: FnOnce() + 'static,
    {
        let key = TypeId::of::<K>();
        let keyed_effects = self.keyed_effects.borrow();
        let should_run = !keyed_effects.contains_key(&key)
            || !keyed_effects.get(&key).map(|e| e.initialized).unwrap_or(false);
        drop(keyed_effects);

        if should_run {
            self.pending_keyed_effects
                .borrow_mut()
                .push(PendingKeyedEffect {
                    key,
                    effect_fn: Box::new(move || {
                        let cleanup = effect_fn();
                        Some(Box::new(cleanup) as Box<dyn FnOnce()>)
                    }),
                    new_deps: None,
                });
        }
    }

    /// Schedule a keyed effect to run when dependencies change.
    /// Order-independent - safe to use in conditionals.
    pub fn use_effect_keyed<K: 'static, D, F, C>(&self, deps: D, effect_fn: F)
    where
        D: PartialEq + Clone + 'static,
        F: FnOnce(&D) -> C + 'static,
        C: FnOnce() + 'static,
    {
        let key = TypeId::of::<K>();
        let keyed_effects = self.keyed_effects.borrow();
        let should_run = match keyed_effects.get(&key) {
            None => true, // First render, always run
            Some(effect_state) => {
                match &effect_state.last_deps {
                    Some(last_deps) => {
                        match last_deps.downcast_ref::<D>() {
                            Some(last) => *last != deps,
                            None => true, // Type mismatch, re-run
                        }
                    }
                    None => true,
                }
            }
        };
        drop(keyed_effects);

        if should_run {
            let deps_for_effect = deps.clone();
            let deps_to_store = deps;
            self.pending_keyed_effects
                .borrow_mut()
                .push(PendingKeyedEffect {
                    key,
                    effect_fn: Box::new(move || {
                        let cleanup = effect_fn(&deps_for_effect);
                        Some(Box::new(cleanup) as Box<dyn FnOnce()>)
                    }),
                    new_deps: Some(Box::new(deps_to_store)),
                });
        }
    }

    /// Run all pending effects (called after render).
    /// Returns true if any effects actually ran (state may have changed).
    ///
    /// # Panics
    /// Panics if effects run more than MAX_EFFECT_RUNS_PER_WINDOW times within
    /// EFFECT_WINDOW_FRAMES frames, indicating a likely infinite loop.
    pub fn flush_effects(&self) -> bool {
        let pending_keyed: Vec<_> = self.pending_keyed_effects.borrow_mut().drain(..).collect();
        let ran_any = !pending_keyed.is_empty();

        for pending_effect in pending_keyed {
            // Cycle detection: check if we've exceeded the threshold
            let run_count = {
                let mut count = self.effect_run_count.borrow_mut();
                *count += 1;
                *count
            };

            if run_count > MAX_EFFECT_RUNS_PER_WINDOW {
                panic!(
                    "\n\
                    ┌─ Telex Effect Cycle Detected ─────────────────────────────────┐\n\
                    │                                                               │\n\
                    │  An effect has run {} times in {} frames.             │\n\
                    │  This usually means an effect is updating state that          │\n\
                    │  triggers itself to run again (infinite loop).                │\n\
                    │                                                               │\n\
                    │  Common causes:                                               │\n\
                    │    • effect! updating state without dependencies              │\n\
                    │    • effect! updating its own dependency                      │\n\
                    │                                                               │\n\
                    │  Fix: Make sure effects don't write to their own deps.        │\n\
                    │  Effects should flow outward (to external systems) or         │\n\
                    │  sideways (to different state), not back to their triggers.   │\n\
                    │                                                               │\n\
                    └───────────────────────────────────────────────────────────────┘",
                    run_count,
                    EFFECT_WINDOW_FRAMES
                );
            }

            // Run previous cleanup if this effect existed
            {
                let mut keyed_effects = self.keyed_effects.borrow_mut();
                if let Some(effect_state) = keyed_effects.get_mut(&pending_effect.key) {
                    if let Some(cleanup) = effect_state.cleanup.take() {
                        drop(keyed_effects); // Release borrow before running cleanup
                        cleanup();
                    }
                }
            }

            // Run effect, get cleanup
            let cleanup = (pending_effect.effect_fn)();

            // Store cleanup and mark initialized
            let mut keyed_effects = self.keyed_effects.borrow_mut();
            let effect_state = keyed_effects
                .entry(pending_effect.key)
                .or_insert_with(|| EffectState {
                    cleanup: None,
                    last_deps: None,
                    initialized: false,
                });
            effect_state.cleanup = cleanup;
            effect_state.initialized = true;
            if let Some(new_deps) = pending_effect.new_deps {
                effect_state.last_deps = Some(new_deps);
            }
        }

        ran_any
    }

    /// Called once per frame to decay the effect run counter.
    /// This implements a sliding window for cycle detection.
    pub fn decay_effect_counter(&self) {
        let mut frames = self.frames_since_decay.borrow_mut();
        *frames += 1;

        if *frames >= EFFECT_WINDOW_FRAMES {
            // Reset the window
            *frames = 0;
            *self.effect_run_count.borrow_mut() = 0;
        }
    }

    /// Run all cleanup functions (called on app exit).
    pub fn cleanup_all_effects(&self) {
        let mut keyed_effects = self.keyed_effects.borrow_mut();
        for effect in keyed_effects.values_mut() {
            if let Some(cleanup) = effect.cleanup.take() {
                cleanup();
            }
        }
    }
}

/// Context passed to components during rendering.
///
/// Provides access to hooks like `state!`, `effect!`, `stream!`, etc.
#[derive(Clone)]
pub struct Scope {
    storage: Rc<StateStorage>,
    commands: Option<Rc<CommandRegistry>>,
    context: Rc<ContextStorage>,
    /// Component identity for future memoization support.
    /// Set by the view! macro when rendering child components.
    component_id: Option<TypeId>,
}

impl Scope {
    /// Create a new scope with fresh state storage.
    pub fn new() -> Self {
        Self {
            storage: Rc::new(StateStorage::new()),
            commands: None,
            context: Rc::new(ContextStorage::new()),
            component_id: None,
        }
    }

    /// Create a scope with existing storage (for re-renders).
    pub fn with_storage(storage: Rc<StateStorage>) -> Self {
        Self {
            storage,
            commands: None,
            context: Rc::new(ContextStorage::new()),
            component_id: None,
        }
    }

    /// Create a scope with existing storage and command registry.
    pub fn with_storage_and_commands(
        storage: Rc<StateStorage>,
        commands: Rc<CommandRegistry>,
    ) -> Self {
        Self {
            storage,
            commands: Some(commands),
            context: Rc::new(ContextStorage::new()),
            component_id: None,
        }
    }

    /// Create a scope with all dependencies.
    pub fn with_all(
        storage: Rc<StateStorage>,
        commands: Rc<CommandRegistry>,
        context: Rc<ContextStorage>,
    ) -> Self {
        Self {
            storage,
            commands: Some(commands),
            context,
            component_id: None,
        }
    }

    /// Get the component identity (if set by the view! macro).
    pub fn component_id(&self) -> Option<TypeId> {
        self.component_id
    }

    /// Set the component identity (used by the view! macro).
    pub fn with_component_id(mut self, id: TypeId) -> Self {
        self.component_id = Some(id);
        self
    }

    /// Get the underlying storage for persistence.
    pub fn storage(&self) -> Rc<StateStorage> {
        Rc::clone(&self.storage)
    }

    /// Create keyed state that persists across re-renders (order-independent).
    ///
    /// The type K acts as the key - same K always returns the same state.
    /// Prefer the `state!` macro which auto-generates the key.
    ///
    /// # Example
    /// ```rust,ignore
    /// let count = state!(cx, || 0);
    /// ```
    pub fn use_state_keyed<K: 'static, T: 'static>(&self, init: impl FnOnce() -> T) -> State<T> {
        self.storage.use_state_keyed::<K, T>(init)
    }

    /// Load keyed async data (order-independent).
    /// Prefer the `async_data!` macro which auto-generates the key.
    ///
    /// # Example
    /// ```rust,ignore
    /// let data = async_data!(cx, || {
    ///     Ok(fetch_data())
    /// });
    /// ```
    pub fn use_async_keyed<K: 'static, T, F>(&self, f: F) -> Async<T>
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> Result<T, String> + Send + 'static,
    {
        self.storage.use_async_keyed::<K, T, F>(f)
    }

    /// Stream data with keyed state (order-independent).
    /// Prefer the `stream!` macro which auto-generates the key.
    ///
    /// # Example
    /// ```rust,ignore
    /// let elapsed = stream!(cx, || {
    ///     (0..).inspect(|_| std::thread::sleep(Duration::from_secs(1)))
    /// });
    /// ```
    pub fn use_stream_keyed<K: 'static, T, F, I>(&self, stream_fn: F) -> StreamHandle<T>
    where
        T: Clone + Default + Send + 'static,
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = T> + Send + 'static,
    {
        self.storage.use_stream_keyed::<K, T, F, I>(stream_fn)
    }

    /// Stream text with keyed state (order-independent).
    /// Prefer the `text_stream!` macro which auto-generates the key.
    ///
    /// # Example
    /// ```rust,ignore
    /// let logs = text_stream!(cx, || {
    ///     generate_log_entries()
    /// });
    /// ```
    pub fn use_text_stream_keyed<K: 'static, F, I>(&self, stream_fn: F) -> TextStreamHandle
    where
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = String> + Send + 'static,
    {
        self.storage.use_text_stream_keyed::<K, F, I>(stream_fn)
    }

    /// Stream text with restart support, keyed state (order-independent).
    /// Prefer the `text_stream_with_restart!` macro which auto-generates the key.
    pub fn use_text_stream_with_restart_keyed<K: 'static, F, I>(
        &self,
        restart: bool,
        stream_fn: F,
    ) -> TextStreamHandle
    where
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = String> + Send + 'static,
    {
        self.storage
            .use_text_stream_with_restart_keyed::<K, F, I>(restart, stream_fn)
    }

    /// Create or get a keyed terminal handle (order-independent).
    /// Prefer the `terminal!` macro which auto-generates the key.
    ///
    /// # Example
    /// ```rust,ignore
    /// let terminal = terminal!(cx);
    /// ```
    pub fn use_terminal_keyed<K: 'static>(&self) -> crate::terminal_state::TerminalHandle {
        self.storage.use_terminal_keyed::<K>()
    }

    /// Create a reducer (order-independent).
    /// Prefer the `reducer!` macro which auto-generates the key.
    ///
    /// Returns `(state, dispatch)` where `dispatch` sends actions through
    /// the reducer function to produce new state.
    ///
    /// # Example
    /// ```rust,ignore
    /// let (state, dispatch) = reducer!(cx, AppState::Idle, |state, action| {
    ///     match (state, action) {
    ///         (_, Action::Reset) => AppState::Idle,
    ///         (s, _) => s,
    ///     }
    /// });
    /// ```
    pub fn use_reducer_keyed<K: 'static, S: Clone + 'static, A: 'static>(
        &self,
        initial: S,
        reducer: impl Fn(S, A) -> S + 'static,
    ) -> (State<S>, Rc<dyn Fn(A)>) {
        self.storage.use_reducer_keyed::<K, S, A>(initial, reducer)
    }

    /// Create a typed inbound channel (order-independent).
    /// Prefer the `channel!` macro which auto-generates the key.
    ///
    /// # Example
    /// ```rust,ignore
    /// let ch = channel!(cx, String);
    /// let tx = ch.tx();
    /// // hand tx to an external thread
    /// for msg in ch.get() { /* ... */ }
    /// ```
    pub fn use_channel_keyed<K: 'static, T: 'static>(&self) -> ChannelHandle<T> {
        self.storage.use_channel_keyed::<K, T>()
    }

    /// Create a bidirectional port (order-independent).
    /// Prefer the `port!` macro which auto-generates the key.
    ///
    /// # Example
    /// ```rust,ignore
    /// let midi = port!(cx, MidiIn, MidiOut);
    /// let inbound_tx = midi.rx.tx();  // external sends MidiIn here
    /// let outbound_tx = midi.tx();    // component sends MidiOut here
    /// for msg in midi.rx.get() { /* ... */ }
    /// ```
    pub fn use_port_keyed<K: 'static, In: 'static, Out: 'static>(&self) -> PortHandle<In, Out> {
        self.storage.use_port_keyed::<K, In, Out>()
    }

    /// Create a periodic interval (order-independent).
    /// Prefer the `interval!` macro which auto-generates the key.
    ///
    /// The callback runs on the main thread each frame that the timer fired.
    /// Internally uses a channel + timer thread.
    ///
    /// # Example
    /// ```rust,ignore
    /// let count = state!(cx, || 0u64);
    /// let c = count.clone();
    /// interval!(cx, Duration::from_secs(1), move || {
    ///     c.update(|n| *n += 1);
    /// });
    /// ```
    pub fn use_interval_keyed<K: 'static>(
        &self,
        duration: std::time::Duration,
        callback: impl Fn() + 'static,
    ) {
        self.storage.use_interval_keyed::<K>(duration, callback)
    }

    /// Register a keyboard command/shortcut.
    ///
    /// The callback will be invoked when the key combination is pressed.
    /// Commands registered later in the render tree take precedence.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn App(cx: Scope) -> View {
    ///     let count = state!(cx, || 0);
    ///     let c = count.clone();
    ///
    ///     // Ctrl+R to reset counter
    ///     cx.use_command(KeyBinding::ctrl('r'), move || {
    ///         c.set(0);
    ///     });
    ///
    ///     view! { <Text>{format!("Count: {}", count.get())}</Text> }
    /// }
    /// ```
    pub fn use_command<F>(&self, binding: KeyBinding, callback: F)
    where
        F: Fn() + 'static,
    {
        if let Some(ref commands) = self.commands {
            commands.register(binding, Rc::new(callback));
        }
    }

    /// Provide a value in the context for child components to access.
    ///
    /// Values are stored by type, so each type can only have one value.
    /// Providing a value of a type that already exists will replace it.
    ///
    /// # Example
    /// ```rust,ignore
    /// #[derive(Clone)]
    /// struct UserState {
    ///     name: String,
    ///     logged_in: bool,
    /// }
    ///
    /// fn App(cx: Scope) -> View {
    ///     cx.provide_context(UserState {
    ///         name: "Alice".to_string(),
    ///         logged_in: true,
    ///     });
    ///
    ///     view! { <Header /> }
    /// }
    /// ```
    pub fn provide_context<T: Clone + 'static>(&self, value: T) {
        self.context.provide(value);
    }

    /// Get a value from the context.
    ///
    /// Returns None if no value of this type has been provided by a parent.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn Header(cx: Scope) -> View {
    ///     let user = cx.use_context::<UserState>();
    ///
    ///     match user {
    ///         Some(u) => view! { <Text>{format!("Hello, {}", u.name)}</Text> },
    ///         None => view! { <Text>"Not logged in"</Text> },
    ///     }
    /// }
    /// ```
    pub fn use_context<T: Clone + 'static>(&self) -> Option<T> {
        self.context.get::<T>()
    }

    /// Get the context storage (for passing to child scopes).
    pub fn context(&self) -> Rc<ContextStorage> {
        Rc::clone(&self.context)
    }

    // ========== Keyed Effects (order-independent) ==========
    //
    // Use the effect!() and effect_once!() macros for convenient access.

    /// Run a keyed side effect only once (on first render).
    /// Order-independent - safe to use in conditionals.
    ///
    /// Prefer the `effect_once!` macro which auto-generates the key:
    /// ```rust,ignore
    /// effect_once!(cx, || {
    ///     println!("initialized");
    ///     || { println!("cleanup"); }
    /// });
    /// ```
    pub fn use_effect_once_keyed<K: 'static, F, C>(&self, effect_fn: F)
    where
        F: FnOnce() -> C + 'static,
        C: FnOnce() + 'static,
    {
        self.storage.use_effect_once_keyed::<K, F, C>(effect_fn)
    }

    /// Run a keyed side effect when dependencies change.
    /// Order-independent - safe to use in conditionals.
    ///
    /// Prefer the `effect!` macro which auto-generates the key:
    /// ```rust,ignore
    /// effect!(cx, count.get(), |&c| {
    ///     println!("count changed to {}", c);
    ///     || {}  // cleanup
    /// });
    /// ```
    pub fn use_effect_keyed<K: 'static, D, F, C>(&self, deps: D, effect_fn: F)
    where
        D: PartialEq + Clone + 'static,
        F: FnOnce(&D) -> C + 'static,
        C: FnOnce() + 'static,
    {
        self.storage.use_effect_keyed::<K, D, F, C>(deps, effect_fn)
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
