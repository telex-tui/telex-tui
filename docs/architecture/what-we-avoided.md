# Deep Dive: What We Avoided

[← Back to main](../architecture.md)

---

Good design is as much about what you leave out as what you include. This document explains the pitfalls Telex deliberately avoids and why.

## 1. Lifetimes in the Public API

### The Problem

Lifetimes are Rust's most powerful feature and its steepest learning curve:

```rust
// Scary API
pub struct State<'a, T> {
    inner: &'a RefCell<T>,
}

pub struct Scope<'a> {
    storage: &'a StateStorage,
}

fn Counter<'a>(cx: Scope<'a>) -> View<'a> {
    let count: State<'a, i32> = cx.use_state(|| 0);
    // ...
}
```

Users would need to:
- Understand lifetime annotations
- Propagate lifetimes through their code
- Deal with lifetime errors

### Our Solution

All public types are `'static`:

```rust
// Clean API
pub struct State<T> {
    inner: Rc<StateInner<T>>,  // Owned, not borrowed
}

pub struct Scope {
    storage: Rc<StateStorage>,  // Owned, not borrowed
}

fn Counter(cx: Scope) -> View {
    let count: State<i32> = cx.use_state(|| 0);
    // ...
}
```

### How We Achieved This

**Owned data instead of references:**

```rust
// Instead of borrowing...
struct BadView<'a> {
    content: &'a str,
}

// ...we own the data
struct GoodView {
    content: String,
}
```

**Rc for shared ownership:**

```rust
// Instead of shared references...
struct BadScope<'a> {
    storage: &'a StateStorage,
}

// ...we use Rc
struct GoodScope {
    storage: Rc<StateStorage>,
}
```

### The Trade-off

| Approach | Benefit | Cost |
|----------|---------|------|
| References | Zero allocation, compile-time checking | Complex API, lifetime annotations |
| Owned/Rc | Simple API, no lifetimes | Allocations, runtime cost |

For UI frameworks, the runtime cost is negligible. API simplicity is paramount.

### When Lifetimes Would Be Better

Low-level, performance-critical code:
- Parsers operating on input slices
- Zero-copy deserialization
- Embedded systems with no allocator

Telex is none of these. Users shouldn't need a PhD to use a UI framework.

---

## 2. Unsafe Code

### The Temptation

`unsafe` can make things "easier":

```rust
// "Just use raw pointers"
struct State {
    ptr: *mut dyn Any,
}

impl State {
    fn get<T>(&self) -> &T {
        unsafe { &*(self.ptr as *const T) }
    }
}
```

Problems:
- No borrow checking
- No lifetime tracking
- Memory corruption possible
- Undefined behavior lurking

### Our Approach: Zero Unsafe

Telex contains no `unsafe` blocks. Everything is safe Rust.

**Interior mutability via RefCell:**

```rust
// Safe interior mutability
struct StateInner<T> {
    value: RefCell<T>,
}

impl<T> State<T> {
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.inner.value.borrow_mut());
        // RefCell checks at runtime that we're not double-borrowing
    }
}
```

**Type erasure via Any:**

```rust
// Safe type erasure
states: Vec<Rc<dyn Any>>

// Safe downcast
let state = any_ref.downcast_ref::<State<T>>()
    .expect("type mismatch");
```

### Runtime Panics vs UB

Our code can panic:

```rust
// This panics
let a = cell.borrow();
let b = cell.borrow_mut();  // Panic: already borrowed

// This panics
let state = any.downcast_ref::<i32>().expect("wrong type");
```

But panics are:
- **Deterministic** - Same input always panics
- **Debuggable** - Stack trace points to the problem
- **Safe** - No memory corruption, no security holes

Undefined behavior is:
- **Non-deterministic** - Might work in debug, crash in release
- **Invisible** - Might corrupt memory silently
- **Dangerous** - Security vulnerabilities, data corruption

### When Unsafe Would Be Needed

- FFI with C libraries
- Implementing core data structures (like Vec itself)
- Extreme performance requirements
- Hardware access

Telex doesn't need any of these.

---

## 3. Global State

### The Temptation

Global state is convenient:

```rust
// "Just use lazy_static"
lazy_static! {
    static ref APP_STATE: Mutex<AppState> = Mutex::new(AppState::new());
}

fn Counter() -> View {
    let count = APP_STATE.lock().unwrap().count;
    // ...
}
```

Problems:
- Hidden dependencies
- Testing is hard (global state persists between tests)
- Initialization order issues
- Thread safety complexity

### Our Approach: Explicit Passing

State is explicitly passed through Scope:

```rust
pub fn run<C: Component>(root: C) -> Result<()> {
    // State created here, explicitly
    let storage = Rc::new(StateStorage::new());

    loop {
        // State passed here, explicitly
        let cx = Scope::with_storage(Rc::clone(&storage));
        let view = root.render(cx);
        // ...
    }
}
```

Components receive state through `Scope`:

```rust
fn Counter(cx: Scope) -> View {
    // cx carries the state, no globals needed
    let count = cx.use_state(|| 0);
    // ...
}
```

### Benefits

**Testing:**
```rust
#[test]
fn test_counter() {
    // Fresh state for each test
    let storage = Rc::new(StateStorage::new());
    let cx = Scope::with_storage(storage);

    // Test in isolation
    let view = Counter(cx);
    assert!(matches!(view, View::VStack(_)));
}
```

**Multiple instances:**
```rust
// Could run multiple apps (useful for testing, embedding)
fn main() {
    let app1 = spawn(|| telex::run(App1));
    let app2 = spawn(|| telex::run(App2));
    // Each has its own state
}
```

**Clear dependencies:**
```rust
// You can see exactly what a component needs
fn Counter(cx: Scope) -> View {
    //      ^^ this is the only input
}
```

### The Thread-Local Alternative

Some frameworks use thread-locals:

```rust
thread_local! {
    static CURRENT_SCOPE: RefCell<Option<Scope>> = RefCell::new(None);
}

fn use_state<T>(init: impl FnOnce() -> T) -> State<T> {
    CURRENT_SCOPE.with(|scope| {
        scope.borrow().as_ref().unwrap().use_state(init)
    })
}
```

This enables:
```rust
fn Counter() -> View {
    let count = use_state(|| 0);  // No cx parameter!
    // ...
}
```

We chose explicit passing because:
- Clearer data flow
- Easier testing
- No hidden magic
- Works with any threading model

---

## 4. Async Complexity

### The Temptation

Async everywhere:

```rust
async fn Counter(cx: Scope) -> View {
    let data = fetch_data().await;
    view! { <Text>{data}</Text> }
}
```

Problems:
- Colored function problem (async infects everything)
- Runtime dependency (tokio, async-std)
- Complexity (futures, pinning, lifetimes)
- Harder to understand control flow

### Our Approach: Synchronous Core

Phase 2 is entirely synchronous:

```rust
pub fn run<C: Component>(root: C) -> Result<()> {
    loop {
        let view = root.render(cx);  // Sync
        terminal.draw(&view)?;       // Sync
        terminal.poll_event()?;      // Sync (with timeout)
    }
}
```

State updates are immediate:

```rust
let on_press = move || {
    count.update(|n| *n += 1);  // Happens now
    // Next render sees the new value
};
```

### Benefits

**Simple mental model:**
- Render runs
- View is returned
- Screen is updated
- Input is handled
- Repeat

**No runtime dependency:**
```toml
[dependencies]
rte = "0.1"
# No tokio, no async-std
```

**Deterministic testing:**
```rust
#[test]
fn test_render() {
    let view = Counter(cx);  // Just call it
    // No .await, no executor, no runtime
}
```

### When Async Will Be Needed

Phase 6 will add `use_async` for:
- Network requests
- File I/O
- Long-running computations

But it will be:
- Opt-in (only for async operations)
- Isolated (doesn't infect the component model)
- Simple (probably just `use_async(|| async { ... })`)

---

## Summary: Principles of Avoidance

### 1. Complexity Budget

Every feature has a complexity cost. We avoid features whose cost exceeds their benefit:

| Feature | Benefit | Cost | Decision |
|---------|---------|------|----------|
| Lifetimes in API | Zero-cost | Learning curve, annotations | Avoid |
| Unsafe code | Performance | Memory safety risk | Avoid |
| Global state | Convenience | Testing, reasoning | Avoid |
| Async core | Flexibility | Colored functions, runtime | Avoid (for now) |

### 2. User-Facing Simplicity

```rust
// What users see
fn Counter(cx: Scope) -> View {
    let count = cx.use_state(|| 0);
    view! { <Button on_press={...}>{count}</Button> }
}
```

No lifetimes. No unsafe. No globals. No async.

### 3. Implementation Complexity is OK

We accept complexity in the implementation:
- `Rc<RefCell<T>>` is more complex than `&mut T`
- `dyn Any` is more complex than generics
- Explicit passing is more verbose than globals

But users don't see this. They get a clean API.

### 4. Defer, Don't Deny

We're not saying "never":
- Lifetimes might appear in advanced APIs
- Unsafe might be needed for performance-critical paths
- Async will come in Phase 6

We're saying "not yet" and "not by default."

[← Back to main](../architecture.md) | [Next: Trade-offs →](tradeoffs.md)
