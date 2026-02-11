# Hello World

Let's understand what makes a Telex app.

```rust
use telex::prelude::*;

struct App;

impl Component for App {
    fn render(&self, _cx: Scope) -> View {
        View::text("Hello, Telex!")
    }
}

fn main() {
    telex::run(App).unwrap();
}
```

Run with: `cargo run -p telex-tui --example 01_hello_world`

## Breaking it down

### The prelude

```rust
use telex::prelude::*;
```

This imports everything you need: `Component`, `View`, `Scope`, the `with!` macro, and common types.

### The component

```rust
struct App;

impl Component for App {
    fn render(&self, _cx: Scope) -> View {
        View::text("Hello, Telex!")
    }
}
```

A component is any struct that implements `Component`. The `render` method returns a `View` - what gets drawn to the screen.

The `cx: Scope` parameter gives you access to hooks (state, effects, etc). We'll use it in the next chapter.

### The view

```rust
View::text("Hello, Telex!")
```

`View` is an enum with variants for every widget type. `View::text()` creates a simple text display.

Views compose. You can nest them:

```rust
View::vstack()
    .child(View::text("Line 1"))
    .child(View::text("Line 2"))
    .build()
```

### Running the app

```rust
fn main() {
    telex::run(App).unwrap();
}
```

`telex::run()` starts the event loop. It takes over the terminal, renders your component, and handles input until the user quits (`Ctrl+Q`).

## The render cycle

Telex re-renders your component when:
- State changes
- A stream emits a value
- The terminal resizes

You don't manually trigger renders. Change state, and the UI updates.

## Try it

Press **F1** in any running example to see what it demonstrates.

Next: [State and Buttons](./state-and-buttons.md)
