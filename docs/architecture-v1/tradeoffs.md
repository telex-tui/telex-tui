# Deep Dive: Trade-offs

[← Back to main](../architecture.md)

---

Every design decision is a trade-off. This document honestly examines what Telex gives up in exchange for its benefits, and why those trade-offs are acceptable.

## Trade-off 1: Enum for View

### What We Chose

```rust
pub enum View {
    Text(TextNode),
    VStack(VStackNode),
    HStack(HStackNode),
    Button(ButtonNode),
    Empty,
}
```

### What We Gained

| Benefit | Why It Matters |
|---------|----------------|
| Pattern matching | Exhaustive checks, compiler-verified completeness |
| No vtable | Faster dispatch, smaller size |
| Trivial Clone | Just clone the enum, no trait gymnastics |
| Easy Debug | Derive Debug, inspect entire tree |
| Type safety | Each variant has its own type |

### What We Lost

**Extensibility without modification:**

```rust
// Users can't do this:
struct MyCustomWidget { ... }
impl Widget for MyCustomWidget { ... }

// They'd need to modify the View enum
```

**Open set of types:**

```rust
// With traits, unknown types work:
fn render(widget: &dyn Widget) { ... }

// With enums, all types must be known:
fn render(view: &View) {
    match view {
        // Must list all variants
    }
}
```

### Why It's Acceptable

1. **Telex controls all widgets** - We define Text, Button, List, etc. Users compose, they don't extend.

2. **Composition works:**
   ```rust
   // User "extends" by composition
   fn MyFancyButton(cx: Scope, label: String) -> View {
       view! {
           <Button on_press={...}>{label}</Button>
       }
   }
   ```

3. **Enum is standard Rust** - `Option`, `Result`, `io::Error` are all enums.

4. **Can migrate later** - If we need extensibility, we can add `View::Custom(Box<dyn Widget>)`.

---

## Trade-off 2: Rc for State

### What We Chose

```rust
pub struct State<T> {
    inner: Rc<StateInner<T>>,
}

struct StateInner<T> {
    value: RefCell<T>,
}
```

### What We Gained

| Benefit | Why It Matters |
|---------|----------------|
| Cheap cloning | Multiple closures share state |
| No lifetimes | Clean API, no `'a` annotations |
| Interior mutability | Mutate through shared reference |
| Simple ownership | No complex borrowing |

### What We Lost

**Compile-time borrow checking:**

```rust
// This compiles but panics at runtime
let a = state.inner.value.borrow();
let b = state.inner.value.borrow_mut();  // PANIC
```

**Thread safety:**

```rust
// Rc is not Send or Sync
std::thread::spawn(move || {
    state.update(...);  // ERROR: Rc cannot be sent between threads
});
```

**Zero-cost abstraction:**

```rust
// Rc has overhead:
// - Reference count (8 bytes)
// - Heap allocation
// - Atomic operations (well, Rc uses non-atomic, but still)
```

### Why It's Acceptable

1. **Runtime panics are caught immediately:**
   ```rust
   // This pattern never causes double-borrow:
   let val = state.get();        // Borrow ends here
   state.update(|n| *n += 1);    // Fresh borrow
   ```

2. **TUI apps are single-threaded:**
   - One terminal
   - One input stream
   - One render loop
   - No need for `Arc`

3. **Overhead is negligible:**
   - Rc clone: ~5 ns
   - User interaction: ~100 ms
   - Ratio: 1:20,000,000

---

## Trade-off 3: Rebuild Focus List Each Frame

### What We Chose

```rust
pub fn collect_focusables(&mut self, view: &View) {
    self.focusables.clear();
    self.collect_recursive(view);
}
```

### What We Gained

| Benefit | Why It Matters |
|---------|----------------|
| Always correct | Tree changes? We rebuild. |
| Simple implementation | ~20 lines of code |
| No bookkeeping | No IDs, no registration |
| No state to synchronize | Focus list = View tree |

### What We Lost

**O(n) per frame:**

```rust
// Every frame, we walk the entire tree
// n = number of widgets
```

**Focus stability:**

```rust
// If focus_index = 1 and we insert a widget before it,
// focus stays at index 1 but points to different widget
```

### Why It's Acceptable

1. **n is small:**
   - Typical TUI: 10-100 widgets
   - 100 widgets × 50 ns each = 5 μs
   - Frame budget: 16 ms
   - Overhead: 0.03%

2. **Focus changes are rare:**
   - User presses Tab once per second
   - We rebuild 60 times per second
   - 59 rebuilds "wasted" but still fast

3. **Stability is a UX problem, not a bug:**
   - UI that changes while focused is confusing anyway
   - Stable IDs add complexity
   - Can add later if needed

---

## Trade-off 4: Full Buffer Diff

### What We Chose

```rust
pub fn diff(&self, other: &Buffer) -> Vec<(u16, u16, &Cell)> {
    for y in 0..self.height {
        for x in 0..self.width {
            if self.get(x, y) != other.get(x, y) {
                changes.push((x, y, self.get(x, y)));
            }
        }
    }
    changes
}
```

### What We Gained

| Benefit | Why It Matters |
|---------|----------------|
| Correctness | Every change is caught |
| Simplicity | One simple loop |
| No bookkeeping | No dirty regions to track |
| Predictable | Same cost every frame |

### What We Lost

**O(width × height) per frame:**

```rust
// 200 columns × 50 rows = 10,000 comparisons
// Even if only one cell changed
```

**Memory bandwidth:**

```rust
// Reading 10,000 cells × 2 buffers = 240 KB
// Every frame
```

### Why It's Acceptable

1. **Cell comparison is cheap:**
   - Compare 3 fields: char, fg, bg
   - ~5 ns per cell
   - 10,000 cells = 50 μs

2. **Terminal I/O dominates:**
   - Writing one cell: ~10 μs
   - Diffing 10,000 cells: 50 μs
   - Writing 100 changed cells: 1 ms
   - I/O is the bottleneck, not diffing

3. **Dirty regions add complexity:**
   - Need to track what changed
   - Bugs if tracking is wrong
   - Marginal benefit for TUI sizes

---

## Trade-off 5: Hook Order Dependency

### What We Chose

```rust
fn Counter(cx: Scope) -> View {
    let a = cx.use_state(|| 0);    // Hook 0
    let b = cx.use_state(|| "");   // Hook 1

    // Hooks must be called in same order every render
}
```

### What We Gained

| Benefit | Why It Matters |
|---------|----------------|
| Familiar model | Same as React |
| Simple implementation | Just a Vec + index |
| No keys/IDs needed | Implicit ordering |
| Clean syntax | Just call use_state() |

### What We Lost

**Conditional hooks panic:**

```rust
fn Broken(cx: Scope, flag: bool) -> View {
    if flag {
        let a = cx.use_state(|| 0);  // Sometimes Hook 0
    }
    let b = cx.use_state(|| "");     // Sometimes Hook 0, sometimes Hook 1
    // PANIC: type mismatch on second render
}
```

**Loop hooks need care:**

```rust
fn AlsoBroken(cx: Scope, items: &[Item]) -> View {
    for item in items {
        let state = cx.use_state(|| 0);  // Different count each render!
    }
}
```

### Why It's Acceptable

1. **Same limitation as React:**
   - Millions of React components work this way
   - Developers learn the rules
   - Linters catch violations

2. **Errors are immediate:**
   - Wrong order → panic first render
   - No silent corruption
   - Easy to debug

3. **Alternatives are complex:**
   - Named hooks need string keys
   - Automatic IDs need tree diffing
   - Both add significant complexity

4. **We can add rules checking:**
   ```rust
   // Future: Clippy-like lint
   #[deny(conditional_hooks)]
   fn Counter(cx: Scope) -> View { ... }
   ```

---

## Summary: The Trade-off Philosophy

### 1. Simple Now, Complex Later

Start with the simplest solution that works:

```
Simple solution → Find limitations → Add complexity where needed
```

Not:

```
Anticipate all needs → Build complex solution → Hope it's right
```

### 2. Correct Over Optimal

Every trade-off prioritizes correctness:

| Choice | Correct | Optimal |
|--------|---------|---------|
| Full rebuild | Yes | No |
| Full diff | Yes | No |
| Runtime borrow check | Yes | Slower |
| Panic on misuse | Yes | Ugly |

Optimization comes later, with profiling to guide it.

### 3. DX Over Implementation Elegance

The user-facing API is simple:

```rust
fn Counter(cx: Scope) -> View {
    let count = cx.use_state(|| 0);
    view! { <Button>{count}</Button> }
}
```

The implementation is allowed to be complex:

```rust
// Fine if users don't see this
Rc<StateInner<T>>
RefCell<Vec<Rc<dyn Any>>>
```

### 4. Defer, Don't Prevent

None of these trade-offs are permanent:

| Current | Future Option |
|---------|---------------|
| Enum View | Add Custom(Box<dyn Widget>) |
| Rc | Add Arc version for threading |
| Full rebuild | Add incremental updates |
| Full diff | Add dirty rectangles |
| Order dependency | Add named hooks |

We can add complexity when we have evidence we need it.

[← Back to main](../architecture.md) | [Next: Future Considerations →](future.md)
