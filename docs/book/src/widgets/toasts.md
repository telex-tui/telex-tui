# Toasts

Ephemeral notifications that auto-dismiss after a duration.

```rust
let toasts = state!(cx, || ToastQueue::with_duration(Duration::from_secs(3)));

// Show toasts
toasts.get().success("Saved successfully!");
toasts.get().error("Connection failed");
toasts.get().info("Processing...");

// Render toast container
View::toast_container()
    .queue(toasts.get())
    .position(ToastPosition::BottomRight)
    .build()
```

Run with: `cargo run -p telex-tui --example 21_toasts`

## Toast queue

Create a toast queue to manage notifications:

```rust
let toasts = state!(cx, || ToastQueue::with_duration(Duration::from_secs(3)));
```

The duration determines how long each toast stays visible before auto-dismissing.

## Toast types

Show different toast types with color coding:

```rust
toasts.get().info("This is informational");
toasts.get().success("Operation completed!");
toasts.get().warning("Warning: check your input");
toasts.get().error("Error: something went wrong");
```

Each type has a distinct color:
- **Info** - Blue/Cyan
- **Success** - Green
- **Warning** - Yellow
- **Error** - Red

## Positioning

Control where toasts appear on screen:

```rust
View::toast_container()
    .queue(toasts.get())
    .position(ToastPosition::BottomRight)  // default
    .build()
```

Available positions:
- `ToastPosition::TopRight`
- `ToastPosition::TopLeft`
- `ToastPosition::BottomLeft`
- `ToastPosition::BottomRight`

## Auto-dismiss

Toasts automatically disappear after their duration:

```rust
// 3 second default
ToastQueue::with_duration(Duration::from_secs(3))

// Custom duration per toast
toasts.get().info_with_duration("Quick message", Duration::from_secs(1));
toasts.get().error_with_duration("Important error", Duration::from_secs(10));
```

## Manual dismiss

Clear all toasts:

```rust
toasts.get().clear();
```

Clear a specific toast:

```rust
let id = toasts.get().info("Message");
toasts.get().remove(id);
```

## Stacking

Multiple toasts stack vertically:

```rust
toasts.get().info("First");
toasts.get().info("Second");
toasts.get().info("Third");
```

New toasts appear below/above existing ones depending on position.

## Real-world patterns

**Save confirmation:**
```rust
let on_save = with!(toasts, data => move || {
    match save_file(&data.get()) {
        Ok(_) => toasts.get().success("File saved successfully"),
        Err(e) => toasts.get().error(&format!("Save failed: {}", e)),
    }
});
```

**Long-running operation:**
```rust
let on_process = with!(toasts => move || {
    toasts.get().info("Processing...");

    std::thread::spawn(with!(toasts => move || {
        std::thread::sleep(Duration::from_secs(2));
        toasts.get().success("Processing complete!");
    }));
});
```

**Form validation:**
```rust
if form.get().is_valid() {
    submit_form();
    toasts.get().success("Form submitted!");
} else {
    toasts.get().error("Please fix the errors");
}
```

## Tips

**Don't overuse** - Too many toasts becomes noise. Reserve for important events.

**Match severity to type** - Use error for actual errors, not "Could not find item" (use info for that).

**Keep messages short** - One line is ideal. Long messages get truncated.

**Bottom right is standard** - Users expect notifications in the bottom right corner.

**Duration matters** - 3-5 seconds for info/success, 5-10 seconds for warnings/errors.

**Don't require action** - Toasts are non-blocking. If the user needs to respond, use a modal.

Next: [Canvas](./canvas.md)
