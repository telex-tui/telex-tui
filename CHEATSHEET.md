# Telex Cheatsheet

Quick reference for common patterns. Copy-paste friendly.

---

## Basic App

```rust
use telex::prelude::*;

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        View::text("Hello, Telex!")
    }
}

fn main() {
    telex::run(App).unwrap();
}
```

---

## State

```rust
// Create state (order-independent, safe in conditionals)
let count = state!(cx, || 0);

// Read
count.get()

// Write
count.set(5);

// Modify
count.update(|n| *n += 1);

// Clone for closures (verbose)
let count_clone = count.clone();
let on_click = move || count_clone.update(|n| *n += 1);

// Clone for closures (with! macro - preferred)
let on_click = with!(count => move || count.update(|n| *n += 1));
```

---

## Keyed State (order-independent)

```rust
// Safe in conditionals - each call site gets unique key
let count = state!(cx, || 0);

// Shared state - same key = same state everywhere
struct MyKey;
let shared = cx.use_state_keyed::<MyKey, _>(|| 0);
```

---

## Layouts

```rust
// Vertical stack
View::vstack()
    .spacing(1)
    .child(View::text("Top"))
    .child(View::text("Bottom"))
    .build()

// Horizontal stack
View::hstack()
    .spacing(2)
    .child(View::text("Left"))
    .child(View::text("Right"))
    .build()

// Boxed container
View::boxed()
    .border(true)
    .padding(1)
    .flex(1)           // fill available space
    .child(content)
    .build()

// Split panes
View::split()
    .horizontal()      // or .vertical()
    .ratio(0.3)        // 30% first, 70% second
    .first(left_content)
    .second(right_content)
    .build()

// Spacer (fills space)
View::spacer()

// Gap (fixed space)
View::gap(2)
```

---

## Text & Styling

```rust
// Plain text
View::text("Hello")

// Styled text
View::styled_text("Important")
    .bold()
    .color(Color::Red)
    .dim()
    .build()
```

---

## Buttons

```rust
View::button()
    .label("Click me")
    .on_press(with!(count => move || count.update(|n| *n += 1)))
    .build()
```

---

## Text Input

```rust
// Single line
View::text_input()
    .value(name.get())
    .placeholder("Enter name...")
    .on_change(with!(name => move |s: String| name.set(s)))
    .build()

// Multi-line
View::text_area()
    .value(content.get())
    .rows(10)
    .on_change(with!(content => move |s: String| content.set(s)))
    .build()
```

---

## Checkbox

```rust
View::checkbox()
    .checked(enabled.get())
    .label("Enable feature")
    .on_toggle(with!(enabled => move |checked: bool| enabled.set(checked)))
    .build()
```

---

## List

```rust
let items = vec!["One".to_string(), "Two".to_string(), "Three".to_string()];

View::list()
    .items(items)
    .selected(selected.get())
    .on_select(with!(selected => move |idx: usize| selected.set(idx)))
    .build()
```

---

## Modal

```rust
let show_modal = state!(cx, || false);

View::modal()
    .visible(show_modal.get())
    .title("My Modal")
    .on_dismiss(with!(show_modal => move || show_modal.set(false)))
    .child(View::text("Modal content"))
    .build()
```

---

## Tabs

```rust
View::tabs()
    .tab("First", View::text("Tab 1 content"))
    .tab("Second", View::text("Tab 2 content"))
    .active(active_tab.get())
    .on_change(with!(active_tab => move |idx: usize| active_tab.set(idx)))
    .build()
```

---

## Keyboard Commands

```rust
use crossterm::event::KeyCode;

// Global key binding
cx.use_command(
    KeyBinding::key(KeyCode::F(1)),
    with!(show_help => move || show_help.update(|v| *v = !*v)),
);

// Common keys
KeyCode::Enter
KeyCode::Esc
KeyCode::Tab
KeyCode::F(1)  // F1, F2, etc.
KeyCode::Char('q')
```

---

## Streams (background data)

```rust
// Continuous updates
let counter = stream!(cx, || {
    (0..).map(|i| {
        std::thread::sleep(Duration::from_secs(1));
        i
    })
});
let value = counter.get();

// Text accumulation (like streaming LLM output)
let output = text_stream!(cx, || {
    words.into_iter().map(|word| {
        std::thread::sleep(Duration::from_millis(100));
        format!("{} ", word)
    })
});
let text = output.get();
let is_loading = output.is_loading();
```

---

## Async Data

```rust
let data = async_data!(cx, || {
    std::thread::sleep(Duration::from_secs(2));
    Ok(fetch_data())  // or Err("message".to_string())
});

match &data {
    Async::Loading => View::text("Loading..."),
    Async::Ready(value) => View::text(format!("Got: {}", value)),
    Async::Error(e) => View::text(format!("Error: {}", e)),
}
```

---

## Effects (side effects)

```rust
// Run once on mount
effect_once!(cx, || {
    println!("Component mounted");
    || { println!("Cleanup on unmount"); }
});

// Run when dependency changes
effect!(cx, count.get(), |&val| {
    println!("Count changed to {}", val);
    || {}  // cleanup (optional)
});
```

---

## Context (global state)

```rust
// Provide (in parent)
cx.provide_context(MyConfig { ... });

// Consume (in child)
let config = cx.use_context::<MyConfig>();
if let Some(cfg) = config {
    // use cfg
}
```

---

## Channels (external events)

```rust
// Receive messages from external threads
let ch = channel!(cx, String);

effect_once!(cx, {
    let tx = ch.tx();
    move || {
        std::thread::spawn(move || {
            tx.send("hello from thread".to_string()).ok();
        });
        || {}
    }
});

// Read this frame's messages
for msg in ch.get() {
    // handle msg
}
```

---

## Ports (bidirectional)

```rust
// Bidirectional communication with external thread
let io = port!(cx, InMsg, OutMsg);

effect_once!(cx, {
    let tx_in = io.rx.tx();
    let rx_out = io.take_outbound_rx();
    move || {
        std::thread::spawn(move || {
            if let Some(rx) = rx_out {
                for msg in rx { /* handle outbound */ }
            }
        });
        || {}
    }
});

// Read inbound
for msg in io.rx.get() { /* ... */ }

// Send outbound
io.tx().send(OutMsg::Command("go".into())).ok();
```

---

## Interval (periodic timer)

```rust
let ticks = state!(cx, || 0u64);

interval!(cx, Duration::from_secs(1), with!(ticks => move || {
    ticks.update(|n| *n += 1);
}));
```

---

## Reducer (state machine)

```rust
let (state, dispatch) = reducer!(cx, AppState::Idle, |state, action| {
    match (state, action) {
        (_, Action::Reset) => AppState::Idle,
        (AppState::Idle, Action::Start) => AppState::Running,
        (s, _) => s,
    }
});

// Dispatch actions from callbacks
dispatch(Action::Start);
```

---

## Slider

```rust
let volume = state!(cx, || 50.0);

View::slider()
    .min(0.0)
    .max(100.0)
    .step(1.0)
    .value(volume.get())
    .label("Volume")
    .on_change(with!(volume => move |v: f64| volume.set(v)))
    .build()
```

---

## Error Boundary

```rust
View::error_boundary()
    .child(risky_view)
    .fallback(View::text("Something went wrong"))
    .build()
```

---

## Custom Widget

```rust
use telex::widget::Widget;
use telex::buffer::{Buffer, Rect};

struct MyWidget;

impl Widget for MyWidget {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        buf.set(area.x, area.y, '█', Color::Cyan, Color::Black);
    }
    fn focusable(&self) -> bool { false }
    fn height_hint(&self, _width: u16) -> Option<u16> { Some(1) }
}

View::custom(Rc::new(RefCell::new(MyWidget)))
```

---

## Common Patterns

### Toggle boolean
```rust
show.update(|v| *v = !*v)
```

### Conditional rendering
```rust
if show_details.get() {
    View::text("Details here")
} else {
    View::text("")
}
```

### Map over items
```rust
let items: Vec<View> = data.iter()
    .map(|item| View::text(item))
    .collect();

View::vstack()
    .children(items)
    .build()
```

---

## Run Examples

```bash
# Run specific example
cargo run -p telex-tui --example 02_counter

# Run with debug info
TELEX_DEBUG=1 cargo run -p telex-tui --example 02_counter

# Interactive example selector
./run-examples.sh
```

---

## Links

- [Examples](crates/telex/examples/) - 39 runnable examples
- [API Docs](https://docs.rs/telex-tui) - Full API reference
- [Architecture](docs/architecture.md) - Design decisions
