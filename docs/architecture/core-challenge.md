# Deep Dive: The Core Challenge

[← Back to main](../architecture.md)

---

Building a React-like UI framework in Rust is fundamentally different from building one in JavaScript, and understanding why reveals deep insights into Rust's design philosophy.

## The JavaScript Model We're Trying to Emulate

In React, this code is trivial:

```javascript
function Counter() {
  const [count, setCount] = useState(0);

  return (
    <button onClick={() => setCount(count + 1)}>
      Count: {count}
    </button>
  );
}
```

Why does this "just work" in JavaScript?

1. **Closures capture freely** - The arrow function `() => setCount(count + 1)` captures `setCount` and `count` without any concern about lifetimes
2. **Garbage collection handles cleanup** - The closure can outlive the render; GC will clean it up when nothing references it
3. **Everything is a reference** - `setCount` is a reference to a function, `count` is a reference to a number (well, a primitive, but conceptually)

## Challenge 1: Closures and Lifetimes

In Rust, closures that capture references have lifetimes:

```rust
fn broken_example() {
    let count = 0;

    // This closure captures `&count`
    let callback = || println!("{}", count);

    // The closure's lifetime is tied to `count`
    // It cannot outlive this function
}

fn try_to_store_callback() {
    let count = 0;
    let callback = || println!("{}", count);

    // ERROR: Cannot store `callback` anywhere that outlives this function
    // because it contains a reference to `count`
    store_somewhere(callback);  // Won't compile
}
```

### Why This Matters for UI Frameworks

An event handler like `on_press` needs to:
1. Be stored in the button widget
2. Survive until the button is pressed (could be seconds later)
3. Still have access to the state it wants to modify

But the component function returns immediately after building the view tree. Any local variables are gone.

```rust
fn Counter(cx: Scope) -> View {
    let count = 0;  // Local variable, dies when function returns

    view! {
        <Button on_press={|| count += 1}>  // ERROR: count doesn't live long enough
            "Click me"
        </Button>
    }
}  // count is dropped here, but button still exists
```

### The 'static Bound Problem

To store a callback for later use, we typically need it to be `'static`:

```rust
pub struct ButtonNode {
    pub on_press: Option<Box<dyn Fn() + 'static>>,
    //                                   ^^^^^^^^ Must live forever
}
```

But a closure that captures a reference to a local variable is NOT `'static`:

```rust
let x = 5;
let closure = || println!("{}", x);  // Captures &x, not 'static

// This won't work:
let boxed: Box<dyn Fn() + 'static> = Box::new(closure);
// ERROR: closure may outlive the current function, but it borrows `x`
```

### Rust's Perspective

This isn't a limitation—it's Rust protecting you. In C++, you could capture a reference, the original variable could go out of scope, and you'd have a dangling reference:

```cpp
std::function<void()> create_callback() {
    int count = 0;
    return [&count]() { count++; };  // UB: dangling reference
}  // count is destroyed, lambda still holds reference
```

Rust's borrow checker prevents this category of bug entirely.

## Challenge 2: No Garbage Collection

JavaScript's GC makes memory management invisible:

```javascript
function createHandler() {
  const data = { value: 42 };
  return () => console.log(data.value);
  // `data` stays alive as long as the returned function exists
  // GC will clean it up when nothing references the function
}
```

In Rust, we must be explicit about ownership:

```rust
fn create_handler() -> impl Fn() {
    let data = Data { value: 42 };
    move || println!("{}", data.value)
    // `move` transfers ownership of `data` INTO the closure
    // The closure now OWNS the data
}
```

### The `move` Keyword

`move` is crucial but changes semantics:

```rust
let count = 5;

// Without move: closure borrows count
let borrow = || println!("{}", count);  // &count

// With move: closure owns a COPY of count
let own = move || println!("{}", count);  // count is copied into closure

// For Copy types like i32, this is fine
// But what about non-Copy types?
```

### Non-Copy Types

```rust
let data = String::from("hello");

let closure = move || println!("{}", data);
// `data` has been MOVED into the closure

println!("{}", data);  // ERROR: data has been moved
```

Once you `move` a non-Copy value into a closure, it's gone from the outer scope. You can't give the same data to two closures:

```rust
let data = String::from("hello");

let c1 = move || println!("{}", data);  // data moved here
let c2 = move || println!("{}", data);  // ERROR: data already moved
```

### The Sharing Problem

UI frameworks need SHARED MUTABLE STATE. Multiple callbacks need to access and modify the same state:

```rust
// We want:
fn Counter(cx: Scope) -> View {
    let count = ???;  // What type allows sharing?

    view! {
        <Button on_press={|| count -= 1}>"-"</Button>  // Needs access
        <Button on_press={|| count += 1}>"+"</Button>  // Also needs access
        <Text>{count}</Text>                           // Also needs access
    }
}
```

## Challenge 3: Ownership is Strict

Rust's ownership rules are simple but have profound implications:

1. Each value has exactly one owner
2. When the owner goes out of scope, the value is dropped
3. You can have either one mutable reference OR any number of immutable references

### The Callback Ownership Problem

```rust
struct Button {
    on_press: Box<dyn Fn()>,
}

let count = 0;

// Who owns count?
let button1 = Button {
    on_press: Box::new(|| count += 1)  // Needs &mut count
};
let button2 = Button {
    on_press: Box::new(|| count += 1)  // Also needs &mut count
};
// ERROR: Cannot have two &mut references to count
```

### Why Rust is Strict

This strictness prevents data races. In a concurrent context:

```rust
// Thread 1: button1.on_press()  -> count += 1
// Thread 2: button2.on_press()  -> count += 1
// Without synchronization, this is a data race
```

Even in single-threaded code, aliased mutation can cause bugs:

```rust
let mut vec = vec![1, 2, 3];
for item in &vec {
    if *item == 2 {
        vec.push(4);  // ERROR: cannot mutate while iterating
        // In C++, this would be iterator invalidation (undefined behavior)
    }
}
```

Rust's rules prevent these bugs at compile time.

## Telex's Solution: Interior Mutability with Rc<RefCell<T>>

We need:
1. **Shared ownership** - Multiple closures hold the same state
2. **Mutation through shared references** - `&self` methods that can modify state
3. **'static lifetime** - No references to local variables

### Rc: Shared Ownership

`Rc<T>` (Reference Counted) provides shared ownership:

```rust
use std::rc::Rc;

let data = Rc::new(42);

let clone1 = Rc::clone(&data);  // Increment ref count
let clone2 = Rc::clone(&data);  // Increment ref count

// All three (data, clone1, clone2) point to the same allocation
// When all are dropped, the data is freed
```

### RefCell: Interior Mutability

`RefCell<T>` provides interior mutability—mutation through `&self`:

```rust
use std::cell::RefCell;

let cell = RefCell::new(42);

// Borrow mutably through a shared reference
*cell.borrow_mut() += 1;

// Borrow checking happens at RUNTIME, not compile time
let a = cell.borrow();      // Immutable borrow
let b = cell.borrow_mut();  // PANIC: already borrowed
```

### Combined: Rc<RefCell<T>>

```rust
use std::rc::Rc;
use std::cell::RefCell;

let count = Rc::new(RefCell::new(0));

let c1 = Rc::clone(&count);  // Shared ownership
let c2 = Rc::clone(&count);  // Shared ownership

// Both closures can now exist and modify the same data
let increment = move || *c1.borrow_mut() += 1;
let decrement = move || *c2.borrow_mut() -= 1;

increment();
decrement();
println!("{}", count.borrow());  // 0
```

### Telex's State<T> Wrapper

We wrap this pattern in a clean API:

```rust
pub struct State<T> {
    inner: Rc<StateInner<T>>,
}

struct StateInner<T> {
    value: RefCell<T>,
}

impl<T> State<T> {
    pub fn get(&self) -> T where T: Clone {
        self.inner.value.borrow().clone()
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.inner.value.borrow_mut());
    }
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self { inner: Rc::clone(&self.inner) }
    }
}
```

Now the user writes:

```rust
let count = cx.use_state(|| 0);
let c1 = count.clone();
let c2 = count.clone();

// Clean, explicit, safe
let increment = move || c1.update(|n| *n += 1);
let decrement = move || c2.update(|n| *n -= 1);
```

## The Design Philosophy

### Explicit Over Implicit

Rust forces us to be explicit about sharing:

```rust
let c1 = count.clone();  // I am creating shared ownership
```

This is more verbose than JavaScript, but:
- You can see exactly what's being shared
- The compiler checks you're doing it correctly
- No hidden allocations or reference counting

### Compile-Time vs Runtime Checking

We've traded compile-time borrow checking for runtime checking (via RefCell):

| Approach | Compile Time | Runtime | Safety |
|----------|-------------|---------|--------|
| References | Borrow checker | No overhead | Memory safe |
| RefCell | None | Borrow tracking | Panic on misuse |

This is an acceptable trade-off because:
1. UI patterns are well-understood (callbacks that modify state)
2. Misuse panics immediately, not silently corrupts
3. The API makes misuse unlikely

### Why Not Unsafe?

We could use raw pointers and `unsafe`:

```rust
// DON'T DO THIS
let count: *mut i32 = ...;
let callback = move || unsafe { *count += 1 };
```

But this would:
- Lose Rust's safety guarantees
- Be easy to misuse
- Require manual memory management
- Potentially have undefined behavior

`Rc<RefCell<T>>` achieves the same goal safely.

## Summary

| JavaScript | Rust Problem | Telex Solution |
|------------|--------------|--------------|
| GC handles memory | Explicit ownership needed | Rc for shared ownership |
| Closures capture freely | Lifetimes restrict captures | `move` + Clone for handles |
| Mutation is unrestricted | Borrow checker is strict | RefCell for interior mutability |
| Everything is dynamic | Types must be known | `dyn Any` for type erasure |

Rust makes us work harder, but in exchange:
- No null pointer dereferences
- No dangling references
- No data races
- No use-after-free
- Predictable performance (no GC pauses)

These guarantees matter for production software, and Telex shows you can have them while still providing a pleasant API.

[← Back to main](../architecture.md) | [Next: Architecture Overview →](overview.md)
