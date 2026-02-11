# Deep Dive: Key Design Decisions

[← Back to main](../architecture.md)

---

This section explores the "why" behind each major design decision in Telex, with comparisons to alternatives and analysis of trade-offs.

## 3.1 View as an Enum, Not Traits

### The Decision

```rust
pub enum View {
    Text(TextNode),
    VStack(VStackNode),
    HStack(HStackNode),
    Button(ButtonNode),
    Empty,
}
```

### The Alternative: Trait Objects

The "OOP" approach would use trait objects:

```rust
pub trait Widget {
    fn render(&self, buffer: &mut Buffer, area: Rect);
    fn is_focusable(&self) -> bool;
    fn as_any(&self) -> &dyn Any;  // For downcasting
}

pub struct View {
    widget: Box<dyn Widget>,
}
```

### Why Enum is Better Here

**1. Pattern Matching is Exhaustive**

```rust
// Enum: Compiler ensures we handle all cases
fn render_view(view: &View, ...) {
    match view {
        View::Text(n) => ...,
        View::VStack(n) => ...,
        View::Button(n) => ...,
        // If we forget a variant, compiler error!
    }
}

// Trait: No such guarantee
fn render_widget(widget: &dyn Widget, ...) {
    widget.render(...);  // Hope render() is implemented correctly
}
```

**2. Clone is Trivial**

```rust
// Enum: Just derive Clone
#[derive(Clone)]
pub enum View { ... }

// Trait: Clone is a nightmare
pub trait Widget: Clone { }  // Doesn't work! Clone isn't object-safe

// You need workarounds like:
pub trait WidgetClone {
    fn clone_box(&self) -> Box<dyn Widget>;
}
```

**3. No Vtable Overhead**

```rust
// Enum: Size is known at compile time
size_of::<View>()  // Fixed size, no indirection

// Trait object: Always a pointer + vtable pointer
size_of::<Box<dyn Widget>>()  // 16 bytes (two pointers)
```

**4. Debug is Easy**

```rust
// Enum: Just derive Debug
#[derive(Debug)]
pub enum View { ... }

// Trait: Need manual implementation
impl Debug for dyn Widget { ... }  // Complex, loses type info
```

### When Would Trait Objects Be Better?

- **Plugin systems** - Unknown types at compile time
- **Many types** - 100+ widget types would bloat the enum
- **User extensibility** - Users define custom widgets

Telex controls all widget types, so enum is the right choice.

### The Rust Idiom

Rust prefers "closed" polymorphism (enum) over "open" polymorphism (traits) when:
- All variants are known at compile time
- You need to match on variants
- You need Clone, Debug, or other non-object-safe traits

This is why `Option<T>`, `Result<T, E>`, and `std::io::Error` are enums.

---

## 3.2 State<T> with Rc<RefCell<T>>

### The Decision

```rust
pub struct State<T> {
    inner: Rc<StateInner<T>>,
}

struct StateInner<T> {
    value: RefCell<T>,
    dirty: RefCell<bool>,
}
```

### Breaking It Down

**Why `Rc`?**

Multiple closures need access to the same state:

```rust
let count = cx.use_state(|| 0);

// Two closures, both need `count`
let c1 = count.clone();
let c2 = count.clone();

view! {
    <Button on_press={move || c1.update(|n| *n -= 1)}>"-"</Button>
    <Button on_press={move || c2.update(|n| *n += 1)}>"+"</Button>
}
```

`Rc` provides shared ownership. All clones point to the same allocation.

**Why `RefCell`?**

We need to mutate through a shared reference:

```rust
impl<T> State<T> {
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        //         ^^^^^ &self, not &mut self
        f(&mut self.inner.value.borrow_mut());
    }
}
```

Without `RefCell`, `update` would need `&mut self`, breaking shared ownership.

**Why wrap in our own `State<T>`?**

Ergonomics:

```rust
// Without wrapper:
let count = Rc::new(RefCell::new(0));
*count.borrow_mut() += 1;
let val = *count.borrow();

// With wrapper:
let count = cx.use_state(|| 0);
count.update(|n| *n += 1);
let val = count.get();
```

### Alternative: Signals (à la Leptos/Sycamore)

```rust
// Signal-based approach
let count = create_signal(cx, 0);
let doubled = create_memo(cx, move || count() * 2);
```

Signals provide:
- Fine-grained reactivity (only update what depends on changed data)
- Automatic dependency tracking
- Potentially less re-rendering

We chose `State<T>` because:
- Simpler to implement
- More familiar to React developers
- Sufficient for TUI apps (small trees, infrequent updates)

Signals could be added later without breaking the API.

### The Borrow Checking Trade-off

```rust
// This panics at runtime
let a = count.inner.value.borrow();
let b = count.inner.value.borrow_mut();  // PANIC: already borrowed
```

We've traded compile-time borrow checking for runtime checking. This is acceptable because:
1. The pattern (read in render, write in callbacks) is well-defined
2. Panics are immediate and obvious
3. The API makes misuse unlikely

---

## 3.3 Scope and Hook Storage

### The Decision

```rust
pub struct StateStorage {
    states: RefCell<Vec<Rc<dyn Any>>>,
    index: RefCell<usize>,
}

pub struct Scope {
    storage: Rc<StateStorage>,
}
```

### How Hooks Work

Hooks rely on call order, just like React:

```rust
fn MyComponent(cx: Scope) -> View {
    // First render:
    let a = cx.use_state(|| 0);    // Creates state at index 0
    let b = cx.use_state(|| "");   // Creates state at index 1

    // Second render:
    let a = cx.use_state(|| 0);    // Retrieves state at index 0
    let b = cx.use_state(|| "");   // Retrieves state at index 1
}
```

The implementation:

```rust
pub fn use_state<T: 'static>(&self, init: impl FnOnce() -> T) -> State<T> {
    let mut index = self.index.borrow_mut();
    let mut states = self.states.borrow_mut();

    let state = if *index < states.len() {
        // State exists, retrieve it
        states[*index]
            .downcast_ref::<State<T>>()
            .expect("type mismatch")
            .clone()
    } else {
        // First render, create state
        let state = State::new(init());
        states.push(Rc::new(state.clone()));
        state
    };

    *index += 1;
    state
}
```

### Why `dyn Any`?

Each hook can have a different type:

```rust
let count = cx.use_state(|| 0);        // State<i32>
let name = cx.use_state(|| String::new()); // State<String>
let items = cx.use_state(Vec::new);    // State<Vec<_>>
```

We need type erasure. `dyn Any` allows storing different types and recovering them:

```rust
// Store any type
states.push(Rc::new(my_state) as Rc<dyn Any>);

// Recover the type
let state = states[i].downcast_ref::<State<i32>>().unwrap();
```

### Alternative: Typed Slots

```rust
struct HookSlots {
    slot_0: Option<State<i32>>,
    slot_1: Option<State<String>>,
    // ...
}
```

Problems:
- Fixed number of hooks
- Types hardcoded
- Not composable

### Alternative: Generic Storage

```rust
struct Storage<H0, H1, H2, ...> {
    h0: Option<H0>,
    h1: Option<H1>,
    // ...
}
```

Problems:
- Type must encode all hooks used
- Different components have different types
- Combinatorial explosion

### The React Model Works

The call-order approach, despite its limitations, has proven effective:
- Works for millions of React components
- Simple to understand
- Simple to implement
- Errors are caught quickly (first conditional hook call)

---

## 3.4 Callbacks as Rc<dyn Fn()>

### The Decision

```rust
pub type Callback = Rc<dyn Fn()>;

pub struct ButtonNode {
    pub label: String,
    pub on_press: Option<Callback>,
}
```

### Why Not `Box<dyn Fn()>`?

`Box` provides unique ownership, but we need to clone Views:

```rust
// We need this for diffing, caching, etc.
let view_copy = view.clone();

// Box<dyn Fn()> isn't Clone!
```

### Why Not `fn()`?

Function pointers can't capture state:

```rust
fn increment() {
    // How do we access `count`?
}

let on_press: fn() = increment;
```

### Why Not Generic Callbacks?

```rust
pub struct ButtonNode<F: Fn()> {
    pub on_press: F,
}
```

Problems:
- `View` would need type parameters
- Different buttons would have different types
- Can't store in a `Vec<View>`

### Why Not Store Just State Handles?

```rust
pub struct ButtonNode {
    pub on_press: ButtonAction,  // Increment, Decrement, etc.
    pub state: State<i32>,
}
```

Problems:
- Limited to predefined actions
- State type must be known
- Not composable

### The Trait Object Trade-off

`Rc<dyn Fn()>` uses dynamic dispatch (vtable lookup). Overhead:
- One pointer dereference
- Indirect function call

For UI callbacks (called on user input, ~10/second max), this is negligible.

### Fn vs FnMut vs FnOnce

```rust
Callback = Rc<dyn Fn()>     // ✓ Can call multiple times, no mutation
Callback = Rc<dyn FnMut()>  // ✗ Would need RefCell to call
Callback = Rc<dyn FnOnce()> // ✗ Can only call once
```

We use `Fn` because:
- Callbacks may be called multiple times (button pressed repeatedly)
- All mutation goes through `State<T>`'s interior mutability

---

## 3.5 The Component Trait

### The Decision

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

### Blanket Implementation Magic

The blanket impl means any closure works as a component:

```rust
// This works because of the blanket impl
telex::run(|cx| view! { <Text>"Hello"</Text> })

// Equivalent to:
struct MyComponent;
impl Component for MyComponent {
    fn render(&self, cx: Scope) -> View {
        view! { <Text>"Hello"</Text> }
    }
}
telex::run(MyComponent)
```

### Why `Fn(Scope) -> View`?

**Why `Fn`, not `FnOnce`?**
```rust
// We need to call render multiple times (re-renders)
loop {
    let view = root.render(cx);  // Called every frame
}
```

**Why `Fn`, not `FnMut`?**
```rust
// FnMut would need &mut self, complicating ownership
fn render(&mut self, cx: Scope) -> View;  // Harder to work with
```

Components should be pure functions of state. Mutation happens through `State<T>`.

### Alternative: Struct Components

```rust
#[derive(Component)]
struct Counter {
    label: String,
}

impl Counter {
    fn render(&self, cx: Scope) -> View {
        let count = cx.use_state(|| 0);
        view! { <Text>{self.label}: {count}</Text> }
    }
}
```

This could be added later! The blanket impl doesn't prevent struct-based components.

### The Props Question

Currently, we don't have a clean props system:

```rust
// We want:
fn Greeting(cx: Scope, name: String) -> View { ... }

// But the trait is:
fn render(&self, cx: Scope) -> View;
```

Future solution: A `#[component]` macro that handles props:

```rust
#[component]
fn Greeting(cx: Scope, name: String) -> View {
    view! { <Text>"Hello, "{name}</Text> }
}
```

---

## 3.6 Focus Management

### The Decision

```rust
pub struct FocusManager {
    focus_index: usize,
    focusables: Vec<Option<Callback>>,
}
```

### Linear Focus Order

We traverse the View tree depth-first, collecting focusables:

```
<VStack>
  <Button>A</Button>     ← index 0
  <HStack>
    <Button>B</Button>   ← index 1
    <Button>C</Button>   ← index 2
  </HStack>
  <Button>D</Button>     ← index 3
</VStack>
```

Tab cycles: A → B → C → D → A → ...

### Alternative: 2D Navigation

```
┌───┬───┬───┐
│ A │ B │ C │
├───┴───┴───┤
│     D     │
└───────────┘

Arrow keys move spatially
```

More complex:
- Need position information
- Need size information
- Need to handle edge cases (what's "right" of A?)

Linear is simpler and works well for TUI.

### Why Rebuild Every Frame?

```rust
pub fn collect_focusables(&mut self, view: &View) {
    self.focusables.clear();  // Clear and rebuild
    self.collect_recursive(view);
}
```

The View tree might change:
- Conditional rendering: `if show { <Button/> }`
- Lists: `items.iter().map(|i| <Button/>)`

Rebuilding is O(n) where n = number of widgets. For TUI (~100 widgets), this is microseconds.

### Alternative: Stable IDs

```rust
pub struct ButtonNode {
    pub id: FocusId,
    // ...
}
```

Track focus by ID, not index. More robust to tree changes, but:
- User must provide IDs (DX cost)
- Or we generate IDs (complexity)

We can add this later if needed.

---

## 3.7 Double Buffering and Diffing

### The Decision

```rust
pub struct Terminal {
    buffer: Buffer,
    prev_buffer: Buffer,
}
```

### Why Diff?

Terminals are slow. Writing every cell every frame causes:
- Visible flicker (partial updates visible)
- Input lag (terminal busy processing output)
- High CPU (escape code processing)

By diffing, we write only changes:

```
Frame 1: "Hello, World!" (14 writes)
Frame 2: "Hello, Rust!!" (5 writes: positions 7-11)
```

### The Diff Algorithm

```rust
pub fn diff(&self, other: &Buffer) -> Vec<(u16, u16, &Cell)> {
    let mut changes = Vec::new();
    for y in 0..self.height {
        for x in 0..self.width {
            if self.get(x, y) != other.get(x, y) {
                changes.push((x, y, self.get(x, y).unwrap()));
            }
        }
    }
    changes
}
```

O(width × height) comparison. For 200×50 terminal = 10,000 comparisons. Trivial.

### Alternative: Dirty Rectangles

Track which regions changed, only compare those:

```rust
struct DirtyRegion {
    x: u16, y: u16,
    width: u16, height: u16,
}
```

More complex, but could reduce comparisons. Not needed yet.

### Alternative: Immediate Mode

Write directly to terminal, no buffering:

```rust
fn render_text(text: &str) {
    print!("{}", text);  // Immediate
}
```

Problems:
- Can't diff (don't know previous state)
- Can't batch updates
- Flicker on complex UIs

### The Swap Trick

```rust
std::mem::swap(&mut self.buffer, &mut self.prev_buffer);
```

After rendering:
- Current buffer becomes previous buffer
- Previous buffer (now current) gets cleared and reused

No allocations per frame. Buffer memory is reused.

---

## 3.8 The view! Macro

### The Decision

```rust
view! {
    <VStack>
        <Text>"Hello"</Text>
        <Button on_press={|| count.update(|n| *n += 1)}>"+"</Button>
    </VStack>
}
```

Expands to:

```rust
rte::View::vstack()
    .child(rte::View::text("Hello"))
    .child(
        rte::View::button()
            .on_press(|| count.update(|n| *n += 1))
            .label("+")
            .build()
    )
    .build()
```

### Why a Proc Macro?

We want JSX-like syntax. Rust's declarative macros (`macro_rules!`) can't parse:
- `<Tag>` opening tags
- `prop={expr}` attributes
- `</Tag>` closing tags

Proc macros can parse arbitrary syntax using `syn`.

### The Builder Pattern

Why do macros generate builders, not struct literals?

```rust
// Struct literal requires all fields
View::Button(ButtonNode {
    label: "+".to_string(),
    on_press: Some(...),  // Must provide even if default
})

// Builder allows omitting defaults
View::button()
    .label("+")
    // on_press defaults to None
    .build()
```

### Alternative: Nested Function Calls

```rust
vstack(vec![
    text("Hello"),
    button("+", || count.update(|n| *n += 1)),
])
```

Problems:
- Props as positional args (which is which?)
- Harder to read with nesting
- No visual structure

### Alternative: No Macro

```rust
View::vstack()
    .child(View::text("Hello"))
    .child(View::button().label("+").on_press(...).build())
    .build()
```

This works! The macro is just sugar. Users can choose.

### Parsing Strategy

```rust
impl Parse for ViewNode {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![<]) {
            // Parse element
            input.parse::<Token![<]>()?;
            let tag: Ident = input.parse()?;
            // Parse props...
            // Parse children...
            // Parse closing tag...
        } else if input.peek(LitStr) {
            // Parse string literal
        } else if input.peek(syn::token::Brace) {
            // Parse expression
        }
    }
}
```

`syn` makes this straightforward. Each branch handles one case.

### Error Messages

Good macro errors require care:

```rust
// Bad error
error: expected `>`
  --> src/main.rs:5:15

// Good error
error: Mismatched tags: expected </VStack>, found </HStack>
  --> src/main.rs:8:5
```

We validate tag matching and provide specific messages.

---

## Summary: Design Principles

1. **Prefer enums over traits** when variants are known
2. **Use Rc for shared ownership**, RefCell for interior mutability
3. **Trade compile-time for runtime checking** when patterns are well-defined
4. **Blanket impls** make APIs flexible without complexity
5. **Rebuild is okay** when the operation is fast enough
6. **Diff to minimize work** when output is expensive
7. **Macros for ergonomics**, not magic—keep manual option

[← Back to main](../architecture.md) | [Next: Data Flow →](data-flow.md)
