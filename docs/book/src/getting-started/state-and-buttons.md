# State and Buttons

Static text is nice, but apps need interactivity. Let's add state.

```rust
use telex::prelude::*;

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let count = state!(cx, || 0);

        View::vstack()
            .child(View::text(format!("Count: {}", count.get())))
            .child(
                View::button()
                    .label("Increment")
                    .on_press(with!(count => move || count.update(|n| *n += 1)))
                    .build()
            )
            .build()
    }
}
```

Run with: `cargo run -p telex-tui --example 02_counter`

## The two macros you need

Telex state uses two macros that work together:

### `state!` - Create state

```rust
let count = state!(cx, || 0);
```

Creates a piece of reactive state with an initial value. Works everywhere - no restrictions on where you call it (unlike React hooks).

### `with!` - Attach callbacks

```rust
.on_press(with!(count => move || count.update(|n| *n += 1)))
```

Captures state for use in callbacks. Handles the cloning automatically.

That's it. Use `state!` to create, `with!` to attach.

## Reading and writing

```rust
count.get()                  // read current value
count.set(5)                 // set new value
count.update(|n| *n += 1)    // modify in place
```

When state changes, Telex re-renders automatically.

## Multiple states

Create as many as you need:

```rust
let count = state!(cx, || 0);
let name = state!(cx, || String::new());
let enabled = state!(cx, || true);
```

## Buttons

```rust
View::button()
    .label("Click me")
    .on_press(callback)
    .build()
```

Buttons use the builder pattern. Configure with methods, then `.build()` to finish.

## Example: Multiple buttons

```rust
let count = state!(cx, || 0);

View::vstack()
    .child(View::text(format!("Count: {}", count.get())))
    .child(
        View::hstack()
            .child(
                View::button()
                    .label("-")
                    .on_press(with!(count => move || count.update(|n| *n -= 1)))
                    .build()
            )
            .child(
                View::button()
                    .label("+")
                    .on_press(with!(count => move || count.update(|n| *n += 1)))
                    .build()
            )
            .build()
    )
    .build()
```

Each button gets its own `with!` callback, but they share the same `count` state.

## Under the hood

`state!` and `with!` are just conveniences:

```rust
// state! expands to:
let count = cx.use_state_keyed::</* auto key */, _>(|| 0);

// with! expands to:
let count_clone = count.clone();
move || count_clone.update(...)
```

**Footnote:** There's also `cx.use_state()` - an index-based API similar to React hooks. It works fine but requires calling hooks in the same order every render (no conditionals). `state!()` doesn't have this restriction, so we recommend it for 99% of cases. The 1% edge case where you might want `use_state()`: when you're porting React code and want to maintain the exact same hook order semantics.

Next: [Styling](./styling.md)
