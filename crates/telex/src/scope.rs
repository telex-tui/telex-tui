use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::async_state::{Async, AsyncHandle};
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

/// A pending effect to run after render (index-based).
struct PendingEffect {
    /// Index in the effects vec.
    index: usize,
    /// The effect function that returns an optional cleanup.
    effect_fn: EffectFn,
    /// New dependencies to store after running.
    new_deps: Option<Box<dyn Any>>,
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
#[derive(Default)]
pub struct StateStorage {
    /// Index-based state storage (legacy, for backwards compatibility)
    states: RefCell<Vec<Rc<dyn Any>>>,
    index: RefCell<usize>,
    /// TypeId-keyed state storage (order-independent)
    keyed_states: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    /// Index-based effect storage (legacy)
    effects: RefCell<Vec<EffectState>>,
    effect_index: RefCell<usize>,
    /// TypeId-keyed effect storage (order-independent)
    keyed_effects: RefCell<HashMap<TypeId, EffectState>>,
    /// Index-based effects scheduled to run after render
    pending_effects: RefCell<Vec<PendingEffect>>,
    /// Keyed effects scheduled to run after render
    pending_keyed_effects: RefCell<Vec<PendingKeyedEffect>>,
    /// Rolling count of effect executions for cycle detection
    effect_run_count: RefCell<usize>,
    /// Frames since last counter decay
    frames_since_decay: RefCell<usize>,
}

impl StateStorage {
    pub fn new() -> Self {
        Self {
            states: RefCell::new(Vec::new()),
            index: RefCell::new(0),
            keyed_states: RefCell::new(HashMap::new()),
            effects: RefCell::new(Vec::new()),
            effect_index: RefCell::new(0),
            keyed_effects: RefCell::new(HashMap::new()),
            pending_effects: RefCell::new(Vec::new()),
            pending_keyed_effects: RefCell::new(Vec::new()),
            effect_run_count: RefCell::new(0),
            frames_since_decay: RefCell::new(0),
        }
    }

    /// Reset the hook indices for a new render pass.
    /// Note: keyed_states don't need resetting - they're accessed by TypeId, not index.
    pub fn reset_index(&self) {
        *self.index.borrow_mut() = 0;
        *self.effect_index.borrow_mut() = 0;
    }

    /// Get or create state by TypeId key (order-independent).
    ///
    /// This is the new API that doesn't require hook ordering rules.
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

    /// Get or create state at the current index (legacy API).
    ///
    /// IMPORTANT: Hooks using this API must be called in the same order every render.
    /// Consider using `use_state_keyed` instead for order-independent state.
    pub fn use_state<T: 'static>(&self, init: impl FnOnce() -> T) -> State<T> {
        let mut index = self.index.borrow_mut();
        let mut states = self.states.borrow_mut();

        let state = if *index < states.len() {
            // State already exists, retrieve it
            let any = &states[*index];
            any.downcast_ref::<State<T>>()
                .expect("State type mismatch - hooks called in different order?")
                .clone()
        } else {
            // First render, create new state
            let state = State::new(init());
            states.push(Rc::new(state));
            states
                .last()
                .unwrap()
                .downcast_ref::<State<T>>()
                .unwrap()
                .clone()
        };

        *index += 1;
        state
    }

    /// Get or create async state at the current index.
    pub fn use_async<T, F>(&self, f: F) -> Async<T>
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> Result<T, String> + Send + 'static,
    {
        let mut index = self.index.borrow_mut();
        let mut states = self.states.borrow_mut();

        let handle = if *index < states.len() {
            // Async handle already exists, retrieve it
            let any = &states[*index];
            any.downcast_ref::<AsyncHandle<T>>()
                .expect("Async type mismatch - hooks called in different order?")
                .clone()
        } else {
            // First render, create new async handle
            let handle = AsyncHandle::new();
            states.push(Rc::new(handle.clone()));
            handle
        };

        *index += 1;

        // Start the async operation if not already started
        handle.start(f);

        // Poll and return current state
        handle.poll()
    }

    /// Get or create stream state at the current index.
    pub fn use_stream<T, F, I>(&self, stream_fn: F) -> StreamHandle<T>
    where
        T: Clone + Default + Send + 'static,
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = T> + Send + 'static,
    {
        let mut index = self.index.borrow_mut();
        let mut states = self.states.borrow_mut();

        let handle = if *index < states.len() {
            // Handle already exists, retrieve it
            let any = &states[*index];
            any.downcast_ref::<StreamHandle<T>>()
                .expect("Stream type mismatch - hooks called in different order?")
                .clone()
        } else {
            // First render, create new handle
            let handle = StreamHandle::new();
            states.push(Rc::new(handle.clone()));
            handle
        };

        *index += 1;

        // Start the stream if not already started
        handle.start(stream_fn);

        // Poll for updates
        handle.poll(|acc, item| *acc = item);

        handle
    }

    /// Get or create text stream state at the current index.
    /// Automatically concatenates string tokens.
    pub fn use_text_stream<F, I>(&self, stream_fn: F) -> TextStreamHandle
    where
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = String> + Send + 'static,
    {
        self.use_text_stream_with_restart(false, stream_fn)
    }

    /// Get or create text stream state, with option to restart.
    ///
    /// If `restart` is true and a previous stream exists, it will be reset
    /// before starting the new stream. Use this when you need to start
    /// a fresh stream (e.g., for a new chat message).
    pub fn use_text_stream_with_restart<F, I>(
        &self,
        restart: bool,
        stream_fn: F,
    ) -> TextStreamHandle
    where
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = String> + Send + 'static,
    {
        let mut index = self.index.borrow_mut();
        let mut states = self.states.borrow_mut();

        let handle = if *index < states.len() {
            let any = &states[*index];
            any.downcast_ref::<TextStreamHandle>()
                .expect("TextStream type mismatch - hooks called in different order?")
                .clone()
        } else {
            let handle = TextStreamHandle::new();
            states.push(Rc::new(handle.clone()));
            handle
        };

        *index += 1;

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

    // ========== Effects ==========

    /// Schedule an effect to run after every render.
    pub fn use_effect<F, C>(&self, effect_fn: F)
    where
        F: FnOnce() -> C + 'static,
        C: FnOnce() + 'static,
    {
        let effect_idx = *self.effect_index.borrow();
        *self.effect_index.borrow_mut() += 1;

        // Always schedule - runs every render
        self.pending_effects.borrow_mut().push(PendingEffect {
            index: effect_idx,
            effect_fn: Box::new(move || {
                let cleanup = effect_fn();
                Some(Box::new(cleanup) as Box<dyn FnOnce()>)
            }),
            new_deps: None,
        });
    }

    /// Schedule an effect to run only once (on first render).
    pub fn use_effect_once<F, C>(&self, effect_fn: F)
    where
        F: FnOnce() -> C + 'static,
        C: FnOnce() + 'static,
    {
        let effect_idx = *self.effect_index.borrow();
        *self.effect_index.borrow_mut() += 1;

        let effects = self.effects.borrow();
        let should_run = effect_idx >= effects.len() || !effects[effect_idx].initialized;
        drop(effects);

        if should_run {
            self.pending_effects.borrow_mut().push(PendingEffect {
                index: effect_idx,
                effect_fn: Box::new(move || {
                    let cleanup = effect_fn();
                    Some(Box::new(cleanup) as Box<dyn FnOnce()>)
                }),
                new_deps: None,
            });
        }
    }

    /// Schedule an effect to run when dependencies change.
    pub fn use_effect_with<D, F, C>(&self, deps: D, effect_fn: F)
    where
        D: PartialEq + Clone + 'static,
        F: FnOnce(&D) -> C + 'static,
        C: FnOnce() + 'static,
    {
        let effect_idx = *self.effect_index.borrow();
        *self.effect_index.borrow_mut() += 1;

        let effects = self.effects.borrow();
        let should_run = if effect_idx >= effects.len() {
            // First render, always run
            true
        } else {
            // Compare deps
            match &effects[effect_idx].last_deps {
                Some(last_deps) => {
                    match last_deps.downcast_ref::<D>() {
                        Some(last) => *last != deps,
                        None => true, // Type mismatch, re-run
                    }
                }
                None => true,
            }
        };
        drop(effects);

        if should_run {
            let deps_for_effect = deps.clone();
            let deps_to_store = deps;
            self.pending_effects.borrow_mut().push(PendingEffect {
                index: effect_idx,
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
        let pending: Vec<_> = self.pending_effects.borrow_mut().drain(..).collect();
        let ran_any = !pending.is_empty();

        for pending_effect in pending {
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
                    │    • use_effect updating state without dependencies           │\n\
                    │    • use_effect_with updating its own dependency              │\n\
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

            let mut effects = self.effects.borrow_mut();

            // Ensure effects vec is large enough
            while effects.len() <= pending_effect.index {
                effects.push(EffectState {
                    cleanup: None,
                    last_deps: None,
                    initialized: false,
                });
            }

            // Run previous cleanup
            if let Some(cleanup) = effects[pending_effect.index].cleanup.take() {
                cleanup();
            }

            // Drop the borrow before running the effect (effect might access state)
            drop(effects);

            // Run effect, get cleanup
            let cleanup = (pending_effect.effect_fn)();

            // Store cleanup and mark initialized
            let mut effects = self.effects.borrow_mut();
            effects[pending_effect.index].cleanup = cleanup;
            effects[pending_effect.index].initialized = true;
            if let Some(new_deps) = pending_effect.new_deps {
                effects[pending_effect.index].last_deps = Some(new_deps);
            }
        }

        // Process keyed effects
        let pending_keyed: Vec<_> = self.pending_keyed_effects.borrow_mut().drain(..).collect();
        let ran_any = ran_any || !pending_keyed.is_empty();

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

    /// Run all cleanup functions (called on app exit).
    pub fn cleanup_all_effects(&self) {
        // Clean up index-based effects
        let mut effects = self.effects.borrow_mut();
        for effect in effects.iter_mut() {
            if let Some(cleanup) = effect.cleanup.take() {
                cleanup();
            }
        }
        drop(effects);

        // Clean up keyed effects
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
/// Provides access to hooks like `use_state`.
#[derive(Clone)]
pub struct Scope {
    storage: Rc<StateStorage>,
    commands: Option<Rc<CommandRegistry>>,
    context: Rc<ContextStorage>,
}

impl Scope {
    /// Create a new scope with fresh state storage.
    pub fn new() -> Self {
        Self {
            storage: Rc::new(StateStorage::new()),
            commands: None,
            context: Rc::new(ContextStorage::new()),
        }
    }

    /// Create a scope with existing storage (for re-renders).
    pub fn with_storage(storage: Rc<StateStorage>) -> Self {
        storage.reset_index();
        Self {
            storage,
            commands: None,
            context: Rc::new(ContextStorage::new()),
        }
    }

    /// Create a scope with existing storage and command registry.
    pub fn with_storage_and_commands(
        storage: Rc<StateStorage>,
        commands: Rc<CommandRegistry>,
    ) -> Self {
        storage.reset_index();
        Self {
            storage,
            commands: Some(commands),
            context: Rc::new(ContextStorage::new()),
        }
    }

    /// Create a scope with all dependencies.
    pub fn with_all(
        storage: Rc<StateStorage>,
        commands: Rc<CommandRegistry>,
        context: Rc<ContextStorage>,
    ) -> Self {
        storage.reset_index();
        Self {
            storage,
            commands: Some(commands),
            context,
        }
    }

    /// Get the underlying storage for persistence.
    pub fn storage(&self) -> Rc<StateStorage> {
        Rc::clone(&self.storage)
    }

    /// Create local state that persists across re-renders.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn Counter(cx: Scope) -> View {
    ///     let count = cx.use_state(|| 0);
    ///     // ...
    /// }
    /// ```
    ///
    /// **Note:** This API requires hooks to be called in the same order every render.
    /// For order-independent state, use `state!` macro instead.
    pub fn use_state<T: 'static>(&self, init: impl FnOnce() -> T) -> State<T> {
        self.storage.use_state(init)
    }

    /// Create keyed state that persists across re-renders (order-independent).
    ///
    /// Unlike `use_state`, this can be called conditionally or in any order.
    /// The type K acts as the key - same K always returns the same state.
    ///
    /// # Example
    /// ```rust,ignore
    /// // Define a key type for shared state
    /// struct CountKey;
    ///
    /// fn Counter(cx: Scope) -> View {
    ///     // Safe to use in conditionals!
    ///     let count = cx.use_state_keyed::<CountKey, _>(|| 0);
    ///     // ...
    /// }
    /// ```
    ///
    /// For independent state, prefer the `state!` macro which auto-generates the key:
    /// ```rust,ignore
    /// let count = state!(cx, || 0);
    /// ```
    pub fn use_state_keyed<K: 'static, T: 'static>(&self, init: impl FnOnce() -> T) -> State<T> {
        self.storage.use_state_keyed::<K, T>(init)
    }

    /// Load async data that persists across re-renders.
    ///
    /// The function is called once on first render. The result is cached
    /// and returned on subsequent renders.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn DataList(cx: Scope) -> View {
    ///     let data = cx.use_async(|| {
    ///         // This runs in a background thread
    ///         Ok(fetch_data())
    ///     });
    ///
    ///     match data {
    ///         Async::Loading => view! { <Text>"Loading..."</Text> },
    ///         Async::Ready(items) => view! { <List items={items} /> },
    ///         Async::Error(e) => view! { <Text>{format!("Error: {}", e)}</Text> },
    ///     }
    /// }
    /// ```
    pub fn use_async<T, F>(&self, f: F) -> Async<T>
    where
        T: Clone + Send + 'static,
        F: FnOnce() -> Result<T, String> + Send + 'static,
    {
        self.storage.use_async(f)
    }

    /// Stream data incrementally with automatic accumulation.
    ///
    /// Perfect for LLM token streaming or any iterator-based async data.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn ChatMessage(cx: Scope) -> View {
    ///     let stream = cx.use_stream(|| {
    ///         // Returns an iterator that yields items over time
    ///         vec!["Hello", " ", "world", "!"].into_iter()
    ///     });
    ///
    ///     if stream.is_loading() {
    ///         view! { <Text>{stream.get()}</Text><Text>"▌"</Text> }
    ///     } else {
    ///         view! { <Text>{stream.get()}</Text> }
    ///     }
    /// }
    /// ```
    pub fn use_stream<T, F, I>(&self, stream_fn: F) -> StreamHandle<T>
    where
        T: Clone + Default + Send + 'static,
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = T> + Send + 'static,
    {
        self.storage.use_stream(stream_fn)
    }

    /// Stream text with automatic concatenation.
    ///
    /// Convenience wrapper for `use_stream` that automatically concatenates
    /// string tokens. Ideal for LLM streaming responses.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn StreamingChat(cx: Scope) -> View {
    ///     let response = cx.use_text_stream(|| {
    ///         // Simulate LLM token stream
    ///         llm_client.stream_completion("Hello!")
    ///     });
    ///
    ///     let cursor = if response.is_loading() { "▌" } else { "" };
    ///     view! { <Text>{format!("{}{}", response.get(), cursor)}</Text> }
    /// }
    /// ```
    pub fn use_text_stream<F, I>(&self, stream_fn: F) -> TextStreamHandle
    where
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = String> + Send + 'static,
    {
        self.storage.use_text_stream(stream_fn)
    }

    /// Stream text with automatic concatenation and restart support.
    ///
    /// Like `use_text_stream`, but allows forcing a restart when `restart` is true.
    /// Use this when you need to start a fresh stream for each new request.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn Chat(cx: Scope) -> View {
    ///     let request_id = cx.use_state(|| 0u32);
    ///     let last_id = cx.use_state(|| 0u32);
    ///
    ///     let needs_restart = request_id.get() != last_id.get();
    ///     let stream = cx.use_text_stream_with_restart(needs_restart, || {
    ///         stream_response()
    ///     });
    ///
    ///     if needs_restart {
    ///         last_id.set(request_id.get());
    ///     }
    ///     // ...
    /// }
    /// ```
    pub fn use_text_stream_with_restart<F, I>(
        &self,
        restart: bool,
        stream_fn: F,
    ) -> TextStreamHandle
    where
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = String> + Send + 'static,
    {
        self.storage
            .use_text_stream_with_restart(restart, stream_fn)
    }

    /// Register a keyboard command/shortcut.
    ///
    /// The callback will be invoked when the key combination is pressed.
    /// Commands registered later in the render tree take precedence.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn App(cx: Scope) -> View {
    ///     let count = cx.use_state(|| 0);
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
    ///     // Provide user state for all children
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

    // ========== Effects ==========
    //
    // Experimental API - newly implemented, may have edge cases or API changes.

    /// Run a side effect after every render.
    ///
    /// The effect function is called after each render completes.
    /// Return a cleanup function that will be called before the next effect runs.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn Logger(cx: Scope) -> View {
    ///     let count = cx.use_state(|| 0);
    ///
    ///     cx.use_effect(|| {
    ///         println!("Rendered with count: {}", count.get());
    ///         || {} // cleanup (runs before next effect)
    ///     });
    ///
    ///     // ...
    /// }
    /// ```
    ///
    /// **Warning:** Be careful not to create infinite loops by updating state
    /// in an effect that runs every render.
    pub fn use_effect<F, C>(&self, effect_fn: F)
    where
        F: FnOnce() -> C + 'static,
        C: FnOnce() + 'static,
    {
        self.storage.use_effect(effect_fn)
    }

    /// Run a side effect only once (on first render).
    ///
    /// The effect function is called only on the first render.
    /// The cleanup function is called on app exit.
    ///
    /// # Example
    /// ```rust,ignore
    /// fn App(cx: Scope) -> View {
    ///     cx.use_effect_once(|| {
    ///         println!("App initialized");
    ///         || {
    ///             println!("App cleanup");
    ///         }
    ///     });
    ///
    ///     // ...
    /// }
    /// ```
    pub fn use_effect_once<F, C>(&self, effect_fn: F)
    where
        F: FnOnce() -> C + 'static,
        C: FnOnce() + 'static,
    {
        self.storage.use_effect_once(effect_fn)
    }

    /// Run a side effect when dependencies change.
    ///
    /// The effect function is called on first render and whenever the
    /// dependencies change (compared via `PartialEq`).
    ///
    /// # Example
    /// ```rust,ignore
    /// fn Counter(cx: Scope) -> View {
    ///     let count = cx.use_state(|| 0);
    ///
    ///     cx.use_effect_with(count.get(), |count| {
    ///         println!("Count changed to: {}", count);
    ///         || {} // cleanup
    ///     });
    ///
    ///     // ...
    /// }
    /// ```
    ///
    /// Multiple dependencies can be passed as a tuple:
    /// ```rust,ignore
    /// cx.use_effect_with((a.get(), b.get()), |(a, b)| {
    ///     println!("a={}, b={}", a, b);
    ///     || {}
    /// });
    /// ```
    pub fn use_effect_with<D, F, C>(&self, deps: D, effect_fn: F)
    where
        D: PartialEq + Clone + 'static,
        F: FnOnce(&D) -> C + 'static,
        C: FnOnce() + 'static,
    {
        self.storage.use_effect_with(deps, effect_fn)
    }

    // ========== Keyed Effects (order-independent) ==========
    //
    // These are the recommended effect APIs. Unlike index-based effects,
    // keyed effects can be used conditionally or in any order.
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

    /// Create or get a terminal handle.
    ///
    /// This uses keyed state internally so it's safe to use in conditionals.
    /// Each call site gets its own terminal handle based on the call location.
    ///
    /// # Example
    /// ```rust,ignore
    /// let terminal = cx.use_terminal();
    /// if !terminal.is_started() {
    ///     terminal.spawn("bash", &[], 80, 24);
    /// }
    /// terminal.poll();
    /// View::terminal().handle(terminal).build()
    /// ```
    #[track_caller]
    pub fn use_terminal(&self) -> crate::terminal_state::TerminalHandle {
        // For simplicity in MVP, just use indexed state
        // This requires maintaining call order like other use_* hooks
        let mut index = self.storage.index.borrow_mut();
        let mut states = self.storage.states.borrow_mut();

        if *index < states.len() {
            let any = &states[*index];
            *index += 1;
            any.downcast_ref::<crate::terminal_state::TerminalHandle>()
                .expect("TerminalHandle type mismatch - hooks called in different order?")
                .clone()
        } else {
            let handle = crate::terminal_state::TerminalHandle::new(24, 80);
            states.push(Rc::new(handle.clone()));
            *index += 1;
            handle
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
