# Cheatsheet

Quick reference for common patterns.

See the full [CHEATSHEET.md](https://github.com/telex-tui/telex-tui/blob/main/CHEATSHEET.md) in the repository root.

## Quick snippets

### Basic app
```rust
struct App;
impl Component for App {
    fn render(&self, cx: Scope) -> View {
        View::text("Hello")
    }
}
fn main() { telex::run(App).unwrap(); }
```

### State
```rust
let count = state!(cx, || 0);
count.get()               // read
count.set(5)              // write
count.update(|n| *n += 1) // modify
```

### Callbacks
```rust
with!(count => move || count.update(|n| *n += 1))
```

### Layouts
```rust
View::vstack().child(...).child(...).build()
View::hstack().child(...).child(...).build()
```

### Button
```rust
View::button().label("Click").on_press(callback).build()
```

### Input
```rust
View::text_input().value(v.get()).on_change(cb).build()
```

### List
```rust
View::list().items(vec).selected(idx).on_select(cb).build()
```

### Modal
```rust
View::modal().visible(show.get()).on_dismiss(cb).child(...).build()
```

### Table
```rust
View::table()
    .column("Name")
    .column("Value")
    .rows(data)
    .selected(idx)
    .on_select(cb)
    .build()
```

### TextArea
```rust
View::text_area()
    .value(text.get())
    .rows(10)
    .on_change(with!(text => move |s| text.set(s)))
    .build()
```

### Tabs
```rust
View::tabs()
    .tab("Home", home_view)
    .tab("Settings", settings_view)
    .active(tab.get())
    .on_change(with!(tab => move |idx| tab.set(idx)))
    .build()
```

## Dynamic Data

### Stream
```rust
let data = stream!(cx, || {
    (0..).map(|i| {
        std::thread::sleep(Duration::from_secs(1));
        i
    })
});
View::text(format!("{}", data.get()))
```

### Effect (one-time)
```rust
effect_once!(cx, || {
    println!("Component mounted!");
    || {}  // cleanup
});
```

### Effect (with dependency)
```rust
effect!(cx, count.get(), |&val| {
    println!("Count: {}", val);
    || {}  // cleanup
});
```

### Async
```rust
let data = async_data!(cx, || {
    fetch_from_api()
});

match &data {
    Async::Loading => View::text("Loading..."),
    Async::Ready(d) => View::text(d),
    Async::Error(e) => View::text(e),
}
```

### Channel
```rust
let ch = channel!(cx, String);
// ch.tx() gives WakingSender to pass to threads
// ch.get() returns this frame's messages
```

### Port (bidirectional)
```rust
let io = port!(cx, InMsg, OutMsg);
// io.rx.tx() / io.rx.get() for inbound
// io.tx() / io.take_outbound_rx() for outbound
```

### Interval
```rust
interval!(cx, Duration::from_secs(1), with!(count => move || {
    count.update(|n| *n += 1);
}));
```

### Reducer
```rust
let (state, dispatch) = reducer!(cx, initial, |s, a| {
    match a { /* ... */ }
});
```

### Slider
```rust
View::slider()
    .min(0.0).max(100.0).step(1.0)
    .value(val.get())
    .label("Volume")
    .on_change(with!(val => move |v: f64| val.set(v)))
    .build()
```

### Error Boundary
```rust
View::error_boundary()
    .child(risky_view)
    .fallback(View::text("Something went wrong"))
    .build()
```

### Custom Widget
```rust
View::custom(Rc::new(RefCell::new(my_widget)))
```

## Keyboard Commands

### Bind a key
```rust
cx.use_command(
    KeyBinding::key(KeyCode::Char('s')).ctrl(true),
    with!(data => move || save(data.get()))
);
```

### Multiple modifiers
```rust
KeyBinding::key(KeyCode::Char('q'))
    .ctrl(true)
    .shift(true)
```

## Common Patterns

### Toggle boolean
```rust
show.update(|v| *v = !*v)
```

### Conditional state (safe in if/else)
```rust
if condition {
    let data = state!(cx, || Vec::new());
}
```

### Add to list
```rust
items.update(|v| v.push(new_item))
```

### Remove from list
```rust
items.update(|v| {
    if idx < v.len() {
        v.remove(idx);
    }
})
```

### Clear list
```rust
items.update(|v| v.clear())
```

For more examples, see the full [CHEATSHEET.md](https://github.com/telex-tui/telex-tui/blob/main/CHEATSHEET.md) in the repository.
