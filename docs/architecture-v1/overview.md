# Deep Dive: Architecture Overview

[← Back to main](../architecture.md)

---

Telex's architecture follows a principle common in well-designed Rust systems: **clear ownership boundaries with explicit data flow**. Let's examine each component and how they interact.

## The High-Level Picture

```
┌─────────────────────────────────────────────────────────────┐
│                        telex::run()                           │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐   │
│  │ StateStorage│  │ FocusManager │  │     Terminal      │   │
│  │  (Rc-based) │  │  (Vec-based) │  │ (crossterm+buffer)│   │
│  └─────────────┘  └──────────────┘  └───────────────────┘   │
│         │                │                    │             │
│         ▼                │                    │             │
│  ┌─────────────┐         │                    │             │
│  │    Scope    │─────────┼────────────────────┤             │
│  └─────────────┘         │                    │             │
│         │                │                    │             │
│         ▼                ▼                    ▼             │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐   │
│  │  Component  │─▶│     View     │─▶│      Buffer       │   │
│  │  (closure)  │  │  (enum tree) │  │  (cell diffing)   │   │
│  └─────────────┘  └──────────────┘  └───────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

### 1. `telex::run()` - The Orchestrator

```rust
pub fn run<C: Component>(root: C) -> Result<()>
```

The `run` function is the application's entry point and main loop owner. It:

- **Owns** the Terminal, FocusManager, and StateStorage
- **Coordinates** the render cycle
- **Dispatches** keyboard events to the appropriate handlers

**Design Principle: Single Point of Control**

By having `run` own everything, we avoid the complexity of distributed state. There's one place where the main loop lives, one place where terminal setup/cleanup happens.

```rust
pub fn run<C: Component>(root: C) -> Result<()> {
    let mut terminal = Terminal::new()?;        // We own this
    let mut focus = FocusManager::new();        // We own this
    let storage = Rc::new(StateStorage::new()); // Shared with components

    loop {
        let cx = Scope::with_storage(Rc::clone(&storage));
        let view = root.render(cx);
        focus.collect_focusables(&view);
        terminal.draw(&view, focus.focus_index())?;

        // Event handling...
    }

    terminal.cleanup()?;
    Ok(())
}
```

### 2. StateStorage - Hook State Persistence

```rust
pub struct StateStorage {
    states: RefCell<Vec<Rc<dyn Any>>>,
    index: RefCell<usize>,
}
```

StateStorage persists hook state across renders. It lives for the entire application lifetime, wrapped in `Rc` so components can reference it.

**Why `Rc<StateStorage>` not owned by run?**

Components need access to the storage through `Scope`. If `run` owned it directly, we'd need to pass `&mut StateStorage` through everything, creating lifetime issues. With `Rc`, we can clone the reference cheaply.

**Why `RefCell` inside?**

The storage needs to be mutated (adding new state) while being shared (multiple hooks accessing it). `RefCell` provides runtime-checked interior mutability.

### 3. Scope - The Component's Window to the World

```rust
pub struct Scope {
    storage: Rc<StateStorage>,
}
```

Scope is intentionally minimal. It's a handle that gives components access to hooks without exposing implementation details.

**Design Principle: Minimal Surface Area**

Scope only exposes what components need:

```rust
impl Scope {
    pub fn use_state<T: 'static>(&self, init: impl FnOnce() -> T) -> State<T> {
        self.storage.use_state(init)
    }
    // Future: use_effect, use_context, use_async, etc.
}
```

Components don't know about `StateStorage`, `RefCell`, or `Rc`. They just call `use_state` and get a handle back.

### 4. Component - User Code

```rust
pub trait Component {
    fn render(&self, cx: Scope) -> View;
}

impl<F: Fn(Scope) -> View> Component for F {
    fn render(&self, cx: Scope) -> View { self(cx) }
}
```

Components are typically closures:

```rust
telex::run(|cx| {
    let count = cx.use_state(|| 0);
    view! { <Text>{count.get()}</Text> }
})
```

**Design Principle: Components are Pure Functions**

A component takes a `Scope` and returns a `View`. It doesn't have direct access to:
- The terminal
- Other components' state
- The event queue

This isolation makes components:
- Easy to test (just call them with a mock Scope)
- Easy to reason about (output depends only on state)
- Composable (no hidden dependencies)

### 5. View - The UI Tree

```rust
pub enum View {
    Text(TextNode),
    VStack(VStackNode),
    HStack(HStackNode),
    Button(ButtonNode),
    Empty,
}
```

View is an enum representing the UI tree. Each variant carries its data:

```rust
pub struct ButtonNode {
    pub label: String,
    pub on_press: Option<Callback>,
}

pub struct VStackNode {
    pub children: Vec<View>,
}
```

**Design Principle: Data, Not Behavior**

View nodes are data structures, not objects with methods. Behavior (rendering, focus handling) is implemented externally:

```rust
// Rendering is a function that operates on View
fn render_view(buffer: &mut Buffer, view: &View, area: Rect, ctx: &mut RenderContext)

// Focus collection is a function that operates on View
fn collect_focusables(&mut self, view: &View)
```

This separation means:
- Views are easily serializable (for debugging, testing)
- New behaviors can be added without modifying View
- The View enum remains simple

### 6. FocusManager - Input Navigation

```rust
pub struct FocusManager {
    focus_index: usize,
    focusables: Vec<Option<Callback>>,
}
```

FocusManager tracks which element is focused and handles Tab navigation.

**Why store callbacks, not indices?**

When the user presses Enter, we need to call the callback immediately. Storing the callback directly avoids a lookup:

```rust
pub fn activate(&self) {
    if let Some(Some(callback)) = self.focusables.get(self.focus_index) {
        callback();  // Direct call, no lookup needed
    }
}
```

**Why rebuild every frame?**

The View tree can change between renders. Rebuilding is O(n) where n is the number of widgets—trivial for TUI apps.

### 7. Terminal - Platform Abstraction

```rust
pub struct Terminal {
    stdout: Stdout,
    buffer: Buffer,
    prev_buffer: Buffer,
}
```

Terminal wraps crossterm and provides double-buffering.

**Design Principle: Abstraction Layers**

```
User code
    ↓
View (our abstraction)
    ↓
Buffer (our abstraction)
    ↓
crossterm (platform abstraction)
    ↓
OS terminal API
```

Each layer hides complexity from the one above:
- User code doesn't know about buffers
- Buffer doesn't know about ANSI escape codes
- crossterm handles Windows vs Unix differences

### 8. Buffer - Efficient Rendering

```rust
pub struct Buffer {
    cells: Vec<Cell>,
    width: u16,
    height: u16,
}

pub struct Cell {
    ch: char,
    fg: Color,
    bg: Color,
}
```

Buffer is a 2D grid of cells with diffing support.

**Design Principle: Immutable Render, Mutable Buffer**

The render process:
1. Clear buffer (set all cells to default)
2. Render View into buffer (write cells)
3. Diff against previous buffer
4. Write only changed cells to terminal
5. Swap buffers

This is a form of double-buffering common in graphics programming.

## Data Flow

### Ownership Flow

```
run() owns:
├── Terminal (exclusive)
├── FocusManager (exclusive)
└── Rc<StateStorage> (shared)

Component receives:
└── Scope (contains Rc<StateStorage>)

Scope dispenses:
└── State<T> (contains Rc<StateInner<T>>)
```

**Key Insight: Shared Ownership is Explicit**

Only `StateStorage` and `State<T>` use `Rc`. Everything else has clear, single ownership. This makes the code easier to reason about.

### Data Flow per Frame

```
1. run() creates Scope with StateStorage reference
2. Component renders, calling use_state() hooks
3. Hooks return State<T> handles
4. Component builds View tree with callbacks
5. View tree is passed to FocusManager (read-only)
6. View tree is passed to Terminal for rendering
7. Terminal diffs and updates screen
8. run() waits for input
```

### Callback Flow

```
User presses Enter
    ↓
run() receives KeyCode::Enter
    ↓
run() calls focus.activate()
    ↓
FocusManager calls stored callback
    ↓
Callback calls state.update(...)
    ↓
State mutates via RefCell
    ↓
Next render sees updated value
```

## Layered Architecture

Telex follows a layered architecture where each layer only depends on layers below it:

```
┌────────────────────────────────────┐
│         User Application           │  ← Highest level
├────────────────────────────────────┤
│     Component + view! macro        │
├────────────────────────────────────┤
│      View + State + Scope          │
├────────────────────────────────────┤
│     Buffer + Render + Focus        │
├────────────────────────────────────┤
│           Terminal                 │
├────────────────────────────────────┤
│          crossterm                 │  ← Lowest level
└────────────────────────────────────┘
```

**No Circular Dependencies**

- Terminal doesn't know about Views
- Views don't know about Terminal
- Render bridges them, knowing both

## Module Structure

```
crates/telex/src/
├── lib.rs         # Public API, run()
├── view.rs        # View enum, builders
├── state.rs       # State<T>
├── scope.rs       # Scope, StateStorage
├── component.rs   # Component trait
├── focus.rs       # FocusManager
├── buffer.rs      # Cell, Buffer, Rect
├── render.rs      # View → Buffer
├── terminal.rs    # crossterm wrapper
└── prelude.rs     # Re-exports

crates/telex-macro/src/
└── lib.rs         # view! proc macro
```

**Design Principle: One Concept Per Module**

Each module owns one concept:
- `state.rs` knows about reactive state
- `focus.rs` knows about focus management
- `buffer.rs` knows about cell buffers

This makes the codebase navigable and testable.

## Why This Architecture?

### Testability

Each component can be tested in isolation:
- Test `Buffer` without a terminal
- Test `View` builders without rendering
- Test `State` without a UI

### Flexibility

The layered design allows replacing parts:
- Swap crossterm for another terminal library
- Use a different rendering backend
- Change focus strategy

### Simplicity

Despite handling complex concerns (state, rendering, input), each piece is simple:
- `State<T>` is ~60 lines
- `FocusManager` is ~50 lines
- `Buffer` is ~80 lines

Simple pieces are easy to understand, debug, and modify.

[← Back to main](../architecture.md) | [Next: Key Design Decisions →](design-decisions.md)
