# How Telex Works: Rust Design & Thinking

This document explains the Rust design decisions behind Telex.

> **Looking for the original architecture document?** The v1 architecture
> (before external events, error boundaries, and the hook cleanup) is preserved
> in [architecture-v1.md](architecture-v1.md) and [architecture-v1/](architecture-v1/).
> It tells the story of how Telex started and why the foundations were built
> the way they were. This document describes where Telex is now.

---

## 1. The Core Challenge

Building a React-like UI framework in Rust is hard because:

1. **Closures and lifetimes don't mix easily** - Event handlers need to outlive the render, but closures that capture references have limited lifetimes
2. **No garbage collection** - We can't just throw closures around and let the GC sort it out
3. **Ownership is strict** - A button's `on_press` callback needs to modify state, but who owns that state?

Telex's design navigates these constraints while keeping the API ergonomic.

📖 **[Deep Dive: The Core Challenge →](architecture/core-challenge.md)**

---

## 2. Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                        telex::run()                              │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐       │
│  │ StateStorage│  │ FocusManager │  │     Terminal      │       │
│  │  (Rc-based) │  │  (Vec-based) │  │ (crossterm+buffer)│       │
│  └─────────────┘  └──────────────┘  └───────────────────┘       │
│         │                │                    │                  │
│         ▼                │                    │                  │
│  ┌─────────────┐  ┌──────────────┐            │                  │
│  │    Scope    │  │   Channels   │  (drain)    │                  │
│  └─────────────┘  └──────────────┘            │                  │
│         │                                     │                  │
│         ▼                ▼                     ▼                  │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐       │
│  │  Component  │─▶│     View     │─▶│      Buffer       │       │
│  │  (closure)  │  │  (enum tree) │  │  (cell diffing)   │       │
│  └─────────────┘  └──────────────┘  └───────────────────┘       │
│                          │                                       │
│                          ▼                                       │
│                   ┌──────────────┐                                │
│                   │   Effects    │  (flush after render)          │
│                   └──────────────┘                                │
└──────────────────────────────────────────────────────────────────┘
                           ▲
                    ┌──────┴───────┐
                    │ External     │  MIDI, WebSocket, serial,
                    │ Sources      │  file watcher, network...
                    └──────────────┘
```

Two data paths feed the render loop:
1. **User input** — keyboard/mouse via crossterm (same as always)
2. **External events** — any thread can send messages into the loop via ports/channels

📖 **[Deep Dive: Architecture Overview →](architecture/overview.md)**

---

## 3. Key Design Decisions

### 3.1 View as an Enum (With Escape Hatch)

```rust
pub enum View {
    Text(TextNode),
    VStack(VStackNode),
    Button(ButtonNode),
    // ... built-in widgets
    Custom(Box<dyn Widget>),  // User-defined widgets
}
```

**Why enum for built-in widgets?**

- **No vtable overhead** - Pattern matching is cheaper than dynamic dispatch
- **Easier debugging** - Can derive Debug, see the whole tree
- **Clone is simple** - Just clone the enum, no `dyn Clone` gymnastics
- **Exhaustive matching** - Compiler ensures we handle all variants

**Why the Custom escape hatch?**

Users can compose existing widgets to build new ones, but some things can't be
composed — a piano roll, a waveform display, a spectrogram. `View::Custom`
lets users define widgets that participate in layout, focus, and rendering:

```rust
trait Widget {
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn focusable(&self) -> bool { false }
    fn handle_key(&mut self, key: KeyEvent) -> bool { false }
}
```

The Canvas widget covers pixel-level graphics (Kitty protocol). Custom widgets
cover character-cell layouts that don't fit the built-in set.

### 3.2 State<T> with Rc<RefCell<T>>

```rust
pub struct State<T> {
    inner: Rc<StateInner<T>>,
}

struct StateInner<T> {
    value: RefCell<T>,
    dirty: RefCell<bool>,
}
```

**The problem:** We want this API:
```rust
let count = state!(cx, || 0);
// Later, in a callback:
count.update(|n| *n += 1);
```

But closures that capture `&mut` references can't be `'static`, and our callbacks need to be stored and called later.

**The solution:** Make `State<T>` a cheap-to-clone handle:

- `Rc` gives us shared ownership - multiple closures can hold the same state
- `RefCell` gives us interior mutability - we can mutate through a shared reference
- Cloning `State<T>` just increments a reference count (one pointer copy)
- The `dirty` flag tracks mutations for render skipping

### 3.3 Scope and Keyed Hook Storage

```rust
pub struct StateStorage {
    /// TypeId-keyed state storage (order-independent)
    keyed_states: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
    /// TypeId-keyed effect storage (order-independent)
    keyed_effects: RefCell<HashMap<TypeId, EffectState>>,
}

pub struct Scope {
    storage: Rc<StateStorage>,
}
```

**Keyed state (the only API):**

```rust
fn Counter(cx: Scope) -> View {
    // Safe in conditionals! Order doesn't matter.
    if show_counter {
        let count = state!(cx, || 0);
    }
}
```

The `state!` macro generates an anonymous struct type at each call site, which becomes the `TypeId` key. Same call site = same state. Different call sites = different state. No hook ordering rules.

For shared state across different call sites, use an explicit key:

```rust
struct SharedCounterKey;

let count_a = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);
let count_b = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);
// count_a and count_b are the SAME state!
```

**Why `dyn Any`?**

Each hook can have a different type. We use `Any` for type erasure and `downcast_ref` to recover the concrete type.

> **History:** Telex originally had index-based hooks (React-style, call-order
> dependent). The keyed API is strictly better — order-independent, safe in
> conditionals, no panic on reorder. The index-based API was removed before
> the ecosystem formed. See [architecture-v1.md](architecture-v1.md) §3.3
> for the original dual-API design.

### 3.4 Callbacks as Rc<dyn Fn()>

```rust
pub type Callback = Rc<dyn Fn()>;

pub struct ButtonNode {
    pub label: String,
    pub on_press: Option<Callback>,
}
```

**Why Rc?**

- `Box<dyn Fn()>` would work but isn't `Clone`
- We need to clone the `View` tree (for diffing, passing around)
- `Rc<dyn Fn()>` is cloneable - cloning just shares the same closure

**Why not Arc?**

Telex is single-threaded (TUI apps typically are). `Rc` is cheaper than `Arc` (no atomic operations). The thread boundary is at the channel — `mpsc::Sender` is `Send`, everything else stays on the main thread.

### 3.5 The Component Trait

```rust
pub trait Component {
    fn render(&self, cx: Scope) -> View;
}

impl<F> Component for F
where
    F: Fn(Scope) -> View,
{
    fn render(&self, cx: Scope) -> View {
        self(cx)
    }
}
```

**Blanket impl for closures:**

This lets users write:
```rust
telex::run(|cx| view! { ... })
```

Instead of defining a struct that implements `Component`. The closure *is* the component.

**Why `Fn` not `FnMut` or `FnOnce`?**

- `Fn` means we can call it multiple times (re-renders)
- `Fn` means no mutation of captured variables (mutation goes through `State`)
- The component itself is stateless; state lives in `StateStorage`

**Component identity:**

Components have implicit identity via their call site — the same mechanism
that `state!` and `effect!` use for keying. The framework tracks a stable key
for every subtree, enabling future optimizations (memoization, selective
re-rendering) without breaking the existing component API.

### 3.6 Effects

```rust
fn Logger(cx: Scope) -> View {
    let count = state!(cx, || 0);

    // Runs when count changes
    effect!(cx, count.get(), |&c| {
        println!("Count changed to {}", c);
        || {} // cleanup
    });

    // Runs once on initialization
    effect_once!(cx, || {
        println!("Component mounted");
        || println!("Cleanup on exit")
    });

    view! { <Text>{count.get()}</Text> }
}
```

Effects run **after** render, not during. The run loop:
1. Renders the view tree
2. Draws to buffer
3. Flushes pending effects
4. If effects modified state, re-renders once more (capped to prevent loops)

Effects use the same `TypeId` keying as state — order-independent, safe in
conditionals. Cycle detection panics if effects run more than 100 times in 10
frames.

### 3.7 Ports and Channels

The foundational primitive for external event sources. A port is a typed,
bidirectional connection between the component tree and the outside world.

```rust
// Bidirectional port
let midi = cx.use_port::<MidiIn, MidiOut>();

// Inbound-only shorthand
let (tx, messages) = cx.use_channel::<MidiMessage>();
```

**How it works:**

- `use_port` / `use_channel` creates `mpsc::channel` pairs
- The `Sender` is `Send` — hand it to any thread (MIDI, network, etc.)
- At the top of each frame, the run loop drains all registered channels
- Components see only messages that arrived since last frame
- Outbound senders are just `mpsc::Sender` — no framework involvement

**Drain strategies:**

The default drains all pending messages. For high-frequency sources,
configurable strategies prevent unbounded growth:

```rust
// Drain all (default)
let (tx, messages) = cx.use_channel::<SensorReading>();

// Ring buffer: keep last 100
let (tx, messages) = cx.use_channel_with::<SensorReading>(ChannelOpts::ring(100));

// Latest value only
let (tx, latest) = cx.use_channel_with::<SensorReading>(ChannelOpts::latest());
```

**Why this matters:**

Every previous Telex hook creates state that lives *inside* the component tree.
Ports are different — the `Sender` is designed to *leave*, crossing the thread
boundary. This makes external event sources (MIDI, WebSocket, file watchers,
serial ports, server push) first-class citizens rather than awkward hacks.

### 3.8 Error Boundaries

External sources crash. Device unplugged, network dropped, malformed data. A
panic in a callback or effect shouldn't kill the entire app.

```rust
view! {
    <ErrorBoundary fallback={|err| view! { <Text>{format!("Error: {}", err)}</Text> }}>
        <MidiPanel />
    </ErrorBoundary>
}
```

When a panic occurs inside an error boundary:
1. The panic is caught (via `catch_unwind`)
2. The subtree is replaced with the fallback view
3. Effect cleanups for the failed subtree run
4. The rest of the app continues

Without error boundaries, panics propagate to the run loop and crash the app
(same as before — opt-in safety, not forced overhead).

### 3.9 use_reducer

Pairs with ports for complex state management. External events become
dispatched actions, state transitions are explicit:

```rust
let (state, dispatch) = cx.use_reducer(AppState::Idle, |state, action| {
    match (state, action) {
        (Idle, Action::StartRecord) => Recording { buffer: vec![] },
        (Recording { buf }, Action::NoteOn(n)) => {
            buf.push(n);
            Recording { buffer: buf }
        },
        (Recording { .. }, Action::Stop) => Idle,
        (state, _) => state,
    }
});

// Wire port messages into the reducer
for msg in midi.rx.get() {
    dispatch(msg.into());
}
```

Cleanly separates "what happened" (events) from "what it means" (state
transitions). Same insight as Elm Architecture, arriving from a different
direction.

### 3.10 Focus Management

```rust
pub struct FocusManager {
    focus_index: usize,
    focusables: Vec<Focusable>,
    scroll_states: Vec<(u16, u16)>,
    initial_focus_applied: bool,
    in_modal: bool,
    saved_focus_index: Option<usize>,
}
```

**Linear collection approach:**

Every render, we walk the `View` tree and collect focusable elements in order.

**Modal focus containment:**

When a modal is visible, `collect_focusables` only collects from within the modal. Focus is trapped inside. When the modal closes, focus is restored to the previously focused element.

**Why rebuild every frame?**

The view tree might change between renders (conditional elements, different counts). Rebuilding is simple and correct. Optimization can come later.

### 3.11 Double Buffering and Diffing

```rust
pub struct Terminal {
    buffer: Buffer,      // Current frame
    prev_buffer: Buffer, // Previous frame
}
```

**Why diff?**

Terminals are slow. Writing every cell every frame causes flicker and lag. By only writing cells that changed, we get smooth updates.

**Dirty-based render skipping:**

`State` tracks a `dirty` flag on every mutation. The run loop checks: if no
state is dirty, no terminal input arrived, and no channel data came in, the
entire render pass is skipped. Reduces idle CPU from ~5-10% to near zero.
When external data arrives via a port and updates state, the dirty flag
naturally triggers a re-render.

**Two-pass rendering (Canvas):**

The Canvas widget uses the Kitty graphics protocol for pixel-level drawing. This bypasses the character buffer entirely:

1. **Pass 1:** Character buffer - all widgets rendered to cell grid, diffed, and flushed
2. **Pass 2:** Canvas graphics - Kitty escape sequences written directly to terminal

### 3.12 The view! Macro

```rust
view! {
    <Button on_press={move || count.update(|n| *n += 1)}>"+"</Button>
}
// Expands to:
telex::View::button()
    .on_press(move || count.update(|n| *n += 1))
    .label("+")
    .build()
```

**Proc macro approach:**

We use `syn` to parse a custom JSX-like syntax and generate builder calls.

**Why builders?**

Builders let us have optional props with defaults. The macro generates the builder chain. Missing optional props just aren't called.

### 3.13 Unicode and Grapheme Cluster Handling

Terminals operate in columns, but Unicode is complex:

- **Grapheme clusters**: User-perceived characters (e.g., emoji, combining chars)
- **Display width**: ASCII = 1 column, emoji/CJK = 2 columns
- **Continuation cells**: Wide chars occupy 2 cells; second cell marked `wide_continuation: true`

```rust
pub fn write_str(&mut self, x: u16, y: u16, s: &str, fg: Color, bg: Color) {
    for grapheme in s.graphemes(true) {
        let width = UnicodeWidthStr::width(grapheme);
        // ... handle width, continuation cells, line boundaries
    }
}
```

Dependencies: `unicode-segmentation`, `unicode-width`

📖 **[Deep Dive: Key Design Decisions →](architecture/design-decisions.md)**

---

## 4. Data Flow

### Render Cycle

```
1. Drain all port/channel receivers into state
        │
        ▼
2. Check dirty flags — skip render if nothing changed
        │
        ▼
3. Create Scope with StateStorage
        │
        ▼
4. Call component.render(cx)
        │
        ▼
5. Component calls state!(), effect!(), use_port(), etc.
        │
        ▼
6. Component returns View tree
        │
        ▼
7. FocusManager collects focusables from View
        │
        ▼
8. Render View to Buffer with focus highlighting
        │
        ▼
9. Diff Buffer against previous, write changes
        │
        ▼
10. Flush pending effects (may trigger one re-render)
        │
        ▼
11. Poll for input event (16ms timeout)
        │
        ├──▶ Key event: dispatch to focused widget, goto 1
        ├──▶ Resize: goto 1
        └──▶ Timeout (no input): goto 1
                (channel data triggers re-render via dirty flags)
```

### State Update Flow (User Input)

```
Button pressed
      │
      ▼
callback() called
      │
      ▼
count.update(|n| *n += 1)
      │
      ▼
StateInner.value mutated via RefCell
      │
      ▼
dirty flag set
      │
      ▼
Next frame: dirty check passes, re-render sees new value
```

### State Update Flow (External Events)

```
External thread (MIDI, WebSocket, etc.)
      │
      ▼
sender.send(message)
      │
      ▼
Message sits in mpsc channel
      │
      ▼
Top of next frame: run loop drains channel into state
      │
      ▼
dirty flag set
      │
      ▼
Component sees new messages via port.rx.get()
      │
      ▼
Component dispatches to reducer or updates state directly
```

📖 **[Deep Dive: Data Flow →](architecture/data-flow.md)**

---

## 5. What We Avoided

### 5.1 Lifetimes in the Public API

No `'a` parameters in `State`, `Scope`, or `View`. Users don't need to think about lifetimes. We achieve this through `Rc` and owned data.

### 5.2 Unsafe Code

Everything is safe Rust. `RefCell` gives us interior mutability with runtime borrow checking. The only way to panic is double-borrowing state (a programmer error, caught immediately).

### 5.3 Global State

No `lazy_static!` or thread-locals for state storage. State lives in `StateStorage` which is passed explicitly through `Scope`. This makes testing easier and avoids hidden dependencies.

### 5.4 Async Runtime

Telex uses threads + `mpsc` channels for background work, not an async
runtime. `use_async`, `use_stream`, `use_text_stream`, and `use_port` all
spawn plain threads and communicate via channels that the run loop polls.

This avoids the colored function problem — async Rust infects everything it
touches (`Send` bounds, runtime dependency, pinning). Threads + channels are
simple, predictable, and more than sufficient at TUI scale.

### 5.5 Reactive Signals

Telex re-renders the entire view tree each frame (when dirty) and relies on
buffer diffing to minimize terminal writes. We don't use fine-grained reactive
signals (Solid.js, Leptos style).

For TUI apps the tree is dozens of widgets, not thousands like web DOM. The
complexity cost of a reactive runtime is enormous and the payoff in a terminal
context is marginal. If tree evaluation ever becomes a bottleneck,
component-level memoization is a smaller intervention than a full reactive
rewrite.

📖 **[Deep Dive: What We Avoided →](architecture/what-we-avoided.md)**

---

## 6. Trade-offs Acknowledged

| Choice | Benefit | Cost |
|--------|---------|------|
| Enum for View (+ Custom) | Fast, simple, debuggable | Adding built-in widgets modifies enum |
| Rc for State | Cheap cloning, no lifetimes | Runtime borrow checking, not thread-safe |
| Rebuild focus list each frame | Simple, always correct | O(n) per frame |
| Full buffer diff | Simple implementation | O(width x height) per frame |
| Threads not async | Simple, no runtime dep | Thread-per-connection overhead |
| Re-render everything (when dirty) | Simple, always correct | Rebuilds unchanged subtrees |

These are reasonable for a TUI framework. Terminals are small, updates are
infrequent (human speed + external event rate), and simplicity aids correctness.

📖 **[Deep Dive: Trade-offs →](architecture/tradeoffs.md)**

---

## 7. Future Considerations

### Selective Re-rendering

As apps grow more ports feeding different parts of the UI, rebuilding the
entire tree because one MIDI note arrived becomes wasteful. Component-level
memoization would let subtrees skip re-evaluation when their inputs haven't
changed:

```rust
cx.memo(deps, |cx| { /* only re-evaluates when deps change */ })
```

Not needed yet — buffer diffing keeps terminal writes cheap regardless.

### The Model Layer

Ports and reducers hint at state that doesn't belong to any one component.
A MIDI connection, a WebSocket, a database handle — these are app-level
resources. `use_context` partially addresses this, but it's tied to the
render tree. A standalone "store" that lives outside the component tree
may eventually be needed.

### Layout

The current layout supports constraints (Fixed, Min, Max, Flex, Percent)
but could benefit from a layout cache and more sophisticated text measurement.

### Performance

If needed:
- Cache layout calculations
- Component-level memoization (selective re-rendering)
- Smarter focus tracking with stable IDs

But premature optimization is the root of all evil. Current approach is fast enough for terminals.

📖 **[Deep Dive: Future Considerations →](architecture/future.md)**
