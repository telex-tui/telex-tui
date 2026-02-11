# Error Boundaries

Catch panics in child views and render a fallback instead of crashing.

```rust
View::error_boundary()
    .child(risky_view)
    .fallback(View::text("Something went wrong"))
    .build()
```

Run with: `cargo run -p telex-tui --example 37_error_boundary`

## What are error boundaries?

Error boundaries wrap a child view tree and catch any panics that occur during rendering. Instead of crashing the entire application, the boundary renders a fallback view.

This is useful for:
- Isolating untrusted or experimental UI sections
- Graceful degradation when data is unexpected
- Preventing one broken widget from taking down the whole app

## Builder API

```rust
View::error_boundary()
    .child(view)        // the view that might panic
    .fallback(view)     // shown if child panics
    .build()
```

If no fallback is provided, the default is `View::text("[error boundary: child panicked]")`.

## Example: safe data display

```rust
let data = state!(cx, || None::<UserProfile>);

View::error_boundary()
    .child({
        let profile = data.get().unwrap();  // might panic if None
        View::text(format!("Welcome, {}!", profile.name))
    })
    .fallback(View::text("Profile unavailable"))
    .build()
```

## Example: isolating widgets

```rust
View::vstack()
    .child(View::text("Header — always visible"))
    .child(
        View::error_boundary()
            .child(possibly_broken_widget())
            .fallback(
                View::styled_text("Widget failed to render")
                    .color(Color::Red)
                    .build()
            )
            .build()
    )
    .child(View::text("Footer — always visible"))
    .build()
```

## What gets caught

Error boundaries catch **panics** during child view rendering. This includes:
- `unwrap()` on `None` or `Err`
- `panic!()` calls
- Index out of bounds
- Any other panic in the child view tree

Error boundaries do **not** catch:
- Panics in effect cleanup functions
- Panics in other threads (those abort the process)
- Logic errors that don't panic

## Tips

**Keep fallbacks simple** — The fallback should be a static view that can't itself panic.

**Nest boundaries** — You can nest error boundaries to isolate different parts of your UI independently.

**Don't overuse** — Error boundaries are for genuinely risky rendering. If you can validate data before rendering, prefer that.
