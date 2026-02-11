# Keyed State

Order-independent state with the `state!` macro.

```rust
// Safe inside conditionals
if show_counter {
    let count = state!(cx, || 0);  // unique key per call site
    count.update(|n| *n += 1);
}
```

Run with: `cargo run -p telex-tui --example 27_keyed_state`

## The Hook Order Problem

React developers know this rule: **hooks must be called in the same order every render**. Telex's `use_state` has the same restriction:

```rust
// ❌ WRONG - This will panic!
fn render(&self, cx: Scope) -> View {
    if show_counter {
        let count = cx.use_state(|| 0);  // Sometimes called, sometimes not
    }
    let name = cx.use_state(String::new);  // Index shifts!
}
```

Why does this fail? `use_state` uses **index-based storage**:
- First `use_state` call → index 0
- Second `use_state` call → index 1
- Third `use_state` call → index 2

When you conditionally skip a hook, the indices shift on different renders, causing type mismatches and panics.

## The Solution: state!

The `state!` macro creates **keyed state** - each invocation gets a unique compile-time key based on its code location:

```rust
// ✅ CORRECT - This works!
fn render(&self, cx: Scope) -> View {
    if show_counter {
        let count = state!(cx, || 0);  // Key: unique to this line
    }
    let name = state!(cx, String::new);  // Key: unique to this line
}
```

Each `state!` call generates a unique zero-sized type as its key, so order doesn't matter.

## How It Works

Under the hood, `state!` expands to `use_state_keyed` with an automatically generated key type:

```rust
// What you write:
let count = state!(cx, || 0);

// What the macro generates (simplified):
struct __Key_file_line_27_col_17;
let count = cx.use_state_keyed::<__Key_file_line_27_col_17, _>(|| 0);
```

The key type is unique to that exact location in your source code, so each `state!` call gets its own storage slot.

## Basic Usage

```rust
let enabled = state!(cx, || true);

enabled.get()              // read
enabled.set(false)         // write
enabled.update(|b| *b = !*b)  // toggle
```

The API is identical to `use_state` - only the creation differs.

## Conditional State

The killer feature: state that exists only when needed:

```rust
let show_advanced = state!(cx, || false);

if show_advanced.get() {
    let detail_level = state!(cx, || 5);
    let custom_option = state!(cx, String::new);

    // Build advanced UI using these states
}
```

When `show_advanced` is false, the `detail_level` and `custom_option` states aren't created. When it becomes true, they're initialized. When it becomes false again, **the values are preserved** - turning it back on shows the same values.

## Real-World Example

A settings panel with collapsible sections:

```rust
let show_notifications = state!(cx, || false);
let show_appearance = state!(cx, || false);
let show_advanced = state!(cx, || false);

View::vstack()
    .child(section_header("Notifications", show_notifications.clone()))
    .child(if show_notifications.get() {
        let email = state!(cx, || true);
        let push = state!(cx, || true);
        let sms = state!(cx, || false);

        View::vstack()
            .child(checkbox("Email notifications", email))
            .child(checkbox("Push notifications", push))
            .child(checkbox("SMS notifications", sms))
            .build()
    } else {
        View::empty()
    })
    .child(section_header("Appearance", show_appearance.clone()))
    .child(if show_appearance.get() {
        let theme = state!(cx, || "dark".to_string());
        let font_size = state!(cx, || 14);

        View::vstack()
            .child(theme_picker(theme))
            .child(font_size_slider(font_size))
            .build()
    } else {
        View::empty()
    })
    .build()
```

Each section's state only exists when expanded, but values persist across collapse/expand.

## Dynamic Lists

Create state for dynamic collections:

```rust
let items = state!(cx, || vec![
    "Item 1".to_string(),
    "Item 2".to_string(),
]);

// Each item gets conditional state
for (i, item) in items.get().iter().enumerate() {
    if i == selected.get() {
        // State that only exists for the selected item
        let editing = state!(cx, || false);
        let draft = state!(cx, || item.clone());

        // Edit UI
    }
}
```

**Important:** This works because each `state!` call is at a unique source location. The loop body is the same location, so all iterations share the same key. For per-item state, see [Shared State](./shared-state.md).

## state! vs use_state

**Use `state!` when:**
- State might be created conditionally
- You're inside an if/match/loop where hooks might skip
- You want order-independence for safety

**Use `use_state` when:**
- All state is always created (no conditionals)
- You prefer explicit index-based ordering
- Maximum performance (tiny bit faster, no key lookup)

In practice, **prefer `state!` by default**. The performance difference is negligible, and the safety is worth it.

## Multiple Conditionals

You can nest conditionals freely:

```rust
if mode.get() == "advanced" {
    let option_a = state!(cx, || false);

    if option_a.get() {
        let sub_option = state!(cx, || "default");
        // Use sub_option
    }
}
```

Each `state!` has its own unique key, so they never interfere.

## State Lifecycle

**Creation:** State is created the first time its `state!` call is executed.

**Persistence:** Once created, state persists even if the conditional becomes false. It's stored by key, not by execution path.

**Cleanup:** State is only cleaned up when the component unmounts.

This means:
```rust
if show_counter {
    let count = state!(cx, || 0);
    count.update(|n| *n += 1);
}

// Later, show_counter becomes false, then true again
// The count value is preserved - it didn't reset to 0
```

## Common Patterns

**Toggle with state:**
```rust
let expanded = state!(cx, || false);
let toggle = with!(expanded => move || expanded.update(|b| *b = !*b));

View::button()
    .label(if expanded.get() { "Collapse" } else { "Expand" })
    .on_press(toggle)
    .build()
```

**Conditional form:**
```rust
if show_form.get() {
    let name = state!(cx, String::new);
    let email = state!(cx, String::new);

    View::vstack()
        .child(input_field("Name", name))
        .child(input_field("Email", email))
        .child(submit_button(name, email))
        .build()
}
```

**Mode-based state:**
```rust
match mode.get() {
    Mode::Edit => {
        let draft = state!(cx, || original.clone());
        edit_view(draft)
    }
    Mode::View => {
        view_display(original)
    }
    Mode::Preview => {
        let zoom = state!(cx, || 100);
        preview_view(original, zoom)
    }
}
```

Each mode gets its own isolated state.

## Debugging

If you see "State type mismatch" with `state!`, you might have:

```rust
// Same location, different types on different renders
if condition_a {
    let x = state!(cx, || 0i32);  // Key_123 -> i32
}
if condition_b {
    let x = state!(cx, || "hello");  // Key_123 -> String (CONFLICT!)
}
```

Solution: Use different variable names or move them to different locations:

```rust
if condition_a {
    let num = state!(cx, || 0i32);
}
if condition_b {
    let text = state!(cx, || "hello");
}
```

## Performance

`state!` has a tiny overhead compared to `use_state`:
- Index lookup: O(1) array access
- Key lookup: O(1) hash map access

The difference is **1-2 nanoseconds per access** - completely negligible in UI code.

## Limitations

`state!` keys are based on **source location**, not execution context:

```rust
// This doesn't work as you might expect
for i in 0..5 {
    let count = state!(cx, || 0);  // All iterations share the SAME state!
}
```

Each loop iteration is the same source location, so they get the same key. For per-item state, use explicit keys with `use_state_keyed` (see [Shared State](./shared-state.md)).

## Tips

**Default to state!** - Unless you have a specific reason to use `use_state`, prefer `state!` for safety.

**Don't mix in same function** - Pick one style per component. Mixing makes it hard to reason about.

**Location matters** - Moving `state!` to a different line creates a new state. Don't refactor carelessly.

**Works with all types** - Any type that works with `use_state` works with `state!`.

**Clone is cheap** - `State<T>` is `Rc<RefCell<T>>` under the hood, so cloning the handle is cheap.

Next: [Shared State](./shared-state.md)
