# How Telex Works: Rust Design & Thinking

This document explains the Rust design decisions behind Telex.

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

📖 **[Deep Dive: Architecture Overview →](architecture/overview.md)**

---

## 3. Key Design Decisions

### 3.1 View as an Enum, Not Traits

```rust
pub enum View {
    Text(TextNode),
    VStack(VStackNode),
    Button(ButtonNode),
    // ...
}
```

**Why not `Box<dyn Widget>`?**

- **No vtable overhead** - Pattern matching is cheaper than dynamic dispatch
- **Easier debugging** - Can derive Debug, see the whole tree
- **Clone is simple** - Just clone the enum, no `dyn Clone` gymnastics
- **Exhaustive matching** - Compiler ensures we handle all variants

**Trade-off:** Adding new widget types requires modifying the enum. But for a framework we control, this is fine. User-defined widgets would use composition, not new variants.

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
let count = cx.use_state(|| 0);
// Later, in a callback:
count.update(|n| *n += 1);
```

But closures that capture `&mut` references can't be `'static`, and our callbacks need to be stored and called later.

**The solution:** Make `State<T>` a cheap-to-clone handle:

- `Rc` gives us shared ownership - multiple closures can hold the same state
- `RefCell` gives us interior mutability - we can mutate through a shared reference
- Cloning `State<T>` just increments a reference count (one pointer copy)

**Why not Copy?**

We tried `impl Copy for State<T>`, but `Rc` isn't `Copy`. The cost of `.clone()` is minimal (just `Rc::clone`), and being explicit about cloning makes ownership clearer:

```rust
let c1 = count.clone();  // Explicit: this closure gets its own handle
let c2 = count.clone();
```

### 3.3 Scope and Hook Storage

```rust
pub struct StateStorage {
    /// Index-based state storage (legacy, for backwards compatibility)
    states: RefCell<Vec<Rc<dyn Any>>>,
    index: RefCell<usize>,
    /// TypeId-keyed state storage (new, order-independent)
    keyed_states: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
}

pub struct Scope {
    storage: Rc<StateStorage>,
}
```

**Two ways to create state:**

**1. Index-based (React-style)** - `cx.use_state(|| init)`

```rust
fn Counter(cx: Scope) -> View {
    let a = cx.use_state(|| 0);  // Hook 0
    let b = cx.use_state(|| ""); // Hook 1
    // ...
}
```

On first render: creates new state, pushes to `states` vec.
On re-render: retrieves existing state by index.
**Caveat:** Must be called in the same order every render.

**2. Keyed (order-independent)** - `state!(cx, || init)` or `cx.use_state_keyed::<K, _>(|| init)`

```rust
fn Counter(cx: Scope) -> View {
    // Safe in conditionals!
    if show_counter {
        let count = state!(cx, || 0);  // Key is baked into code location
    }
}
```

The `state!` macro generates an anonymous struct type at each call site, which becomes the key. Same call site = same state. Different call sites = different state.

For shared state across different call sites, use an explicit key:

```rust
struct SharedCounterKey;

let count_a = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);
let count_b = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);
// count_a and count_b are the SAME state!
```

**Why `dyn Any`?**

Each hook can have a different type. We use `Any` for type erasure and `downcast_ref` to recover the concrete type.

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

Telex is single-threaded (TUI apps typically are). `Rc` is cheaper than `Arc` (no atomic operations). If we need threading later, we can switch.

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

### 3.6 Focus Management

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

**Initial focus:**

TextInput supports `focused: true` to indicate it should receive initial focus. The focus manager tracks this and applies initial focus only once per app lifecycle.

**Cursor navigation:**

TextInput and TextArea support cursor movement via `on_cursor_change` callbacks:
- TextInput: left/right arrow keys
- TextArea: up/down/left/right with line wrapping

**Why rebuild every frame?**

The view tree might change between renders (conditional elements, different counts). Rebuilding is simple and correct. Optimization can come later.

### 3.7 Double Buffering and Diffing

```rust
pub struct Terminal {
    buffer: Buffer,      // Current frame
    prev_buffer: Buffer, // Previous frame
}
```

**Why diff?**

Terminals are slow. Writing every cell every frame causes flicker and lag. By only writing cells that changed, we get smooth updates.

**Two-pass rendering (Canvas):**

The Canvas widget uses the Kitty graphics protocol for pixel-level drawing. This bypasses the character buffer entirely:

1. **Pass 1:** Character buffer - all widgets rendered to cell grid, diffed, and flushed
2. **Pass 2:** Canvas graphics - Kitty escape sequences written directly to terminal

Canvas widgets reserve their cell area (filled with spaces) in Pass 1, then the actual pixel graphics are overlaid in Pass 2.

### 3.8 The view! Macro

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

### 3.9 Unicode and Grapheme Cluster Handling

Terminals operate in columns, but Unicode is complex. A single "character" as perceived by a user might be:
- Multiple code points (e.g., `é` = `e` + combining accent)
- A single emoji that displays as 2 columns wide
- An emoji sequence (e.g., family emoji = multiple code points)

**The problem:**

```
Terminal columns:  [0][1][2][3][4][5][6][7]
ASCII "hello":      h  e  l  l  o
Emoji "😊😊":       😊    😊          <- Each emoji needs 2 columns
```

If we iterate by `char` instead of grapheme clusters, or assume width=1, rendering breaks.

**The solution:**

```rust
// Cell tracks whether it's a "continuation" of a wide character
pub struct Cell {
    pub ch: char,
    pub wide_continuation: bool,  // True = skip when rendering
    // ...
}
```

When writing strings to the buffer:

```rust
pub fn write_str(&mut self, x: u16, y: u16, s: &str, fg: Color, bg: Color) {
    for grapheme in s.graphemes(true) {  // Iterate by grapheme cluster
        let width = UnicodeWidthStr::width(grapheme);  // 1 or 2 columns

        // Wide char at line end? Write space instead (can't split across lines)
        if width == 2 && col + 1 >= self.width {
            self.set_cell(col, y, Cell::new(' ', fg, bg));
            break;
        }

        self.set_cell(col, y, Cell::new(ch, fg, bg));

        // Mark second column as continuation (renderer skips it)
        if width == 2 {
            self.set_cell(col + 1, y, Cell::wide_continuation(fg, bg));
        }
        col += width as u16;
    }
}
```

**Key insight:** The continuation cell pattern means:
- The wide character is stored in column N
- Column N+1 is marked `wide_continuation: true` with a space
- The renderer skips continuation cells (the wide char already occupies the space)
- Diffing works correctly (both cells compared)

**Edge case - line boundaries:**

When a 2-column emoji would overflow the line, we write a space instead and stop. This is why you see a gap at the end of some lines where an `x` would fit but an emoji won't - the emoji needs 2 columns but only 1 remains.

**Dependencies:**
- `unicode-segmentation` - Grapheme cluster iteration
- `unicode-width` - Display width calculation (UAX #11)

📖 **[Deep Dive: Key Design Decisions →](architecture/design-decisions.md)**

---

## 4. Data Flow

### Render Cycle

```
1. Create Scope with StateStorage
        │
        ▼
2. Call component.render(cx)
        │
        ▼
3. Component calls cx.use_state() to get/create state
        │
        ▼
4. Component returns View tree
        │
        ▼
5. FocusManager collects focusables from View
        │
        ▼
6. Render View to Buffer with focus highlighting
        │
        ▼
7. Diff Buffer against previous, write changes
        │
        ▼
8. Wait for input event
        │
        ├──▶ Tab: focus_next(), goto 1
        ├──▶ Enter: activate() callback, goto 1
        └──▶ Ctrl+Q: exit
```

### State Update Flow

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
dirty flag set (for future optimization)
      │
      ▼
Next render cycle sees new value via count.get()
```

📖 **[Deep Dive: Data Flow →](architecture/data-flow.md)**

---

## 5. What We Avoided

### 5.1 Lifetimes in the Public API

No `'a` parameters in `State`, `Scope`, or `View`. Users don't need to think about lifetimes. We achieve this through `Rc` and owned data.

### 5.2 Unsafe Code

Everything is safe Rust. `RefCell` gives us interior mutability with runtime borrow checking. The only way to panic is calling hooks in wrong order or double-borrowing state (both are programmer errors).

### 5.3 Global State

No `lazy_static!` or thread-locals for state storage. State lives in `StateStorage` which is passed explicitly through `Scope`. This makes testing easier and avoids hidden dependencies.

### 5.4 Async Complexity

Telex is synchronous. State updates happen immediately, and the next render sees them. Async support is planned.

📖 **[Deep Dive: What We Avoided →](architecture/what-we-avoided.md)**

---

## 6. Trade-offs Acknowledged

| Choice | Benefit | Cost |
|--------|---------|------|
| Enum for View | Fast, simple, debuggable | Can't extend without modifying enum |
| Rc for State | Cheap cloning, no lifetimes | Runtime borrow checking, not thread-safe |
| Rebuild focus list each frame | Simple, always correct | O(n) per frame |
| Full buffer diff | Simple implementation | O(width×height) per frame |
| Hook order dependency | Familiar React model | Can panic if order changes |

These are reasonable for a TUI framework. Terminals are small, updates are infrequent (human speed), and simplicity aids correctness.

📖 **[Deep Dive: Trade-offs →](architecture/tradeoffs.md)**

---

## 7. Future Considerations

### Layout

The current layout (equal distribution) is naive. Real layout needs:
- Constraint solving (min/max/preferred sizes)
- Flex factors
- Possibly a layout cache

### Async

`use_async` will need:
- A way to trigger re-renders from async tasks
- Probably `tokio` or `async-std` integration
- State that represents Loading/Ready/Error

### Performance

If needed:
- Cache layout calculations
- Incremental View diffing (not just buffer diffing)
- Smarter focus tracking with stable IDs

But premature optimization is the root of all evil. Current approach is fast enough for terminals.

📖 **[Deep Dive: Future Considerations →](architecture/future.md)**
