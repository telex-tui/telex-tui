# Testing in Telex

This document explains how to write and run tests for Telex components.

---

## 1. Running Tests

```bash
# All workspace tests
cargo test

# Just the main telex crate
cargo test -p telex-tui

# With output shown
cargo test -- --nocapture

# Run a specific test
cargo test test_emoji_at_wrap_boundary
```

---

## 2. The TestApp Harness

Telex provides `TestApp` for testing components without a real terminal. It renders to an in-memory buffer and provides assertion helpers.

### Basic Usage

```rust
use telex::testing::TestApp;
use telex::prelude::*;

#[test]
fn test_button_renders() {
    let mut app = TestApp::new(|_cx| {
        View::button().label("Click Me").build()
    }).with_size(30, 5);

    app.assert_visible("Click Me");
}
```

### Setting Terminal Size

```rust
// Default is 80x24
let mut app = TestApp::new(component).with_size(40, 10);
```

Size matters for testing wrapping, overflow, and layout behavior.

### Getting Raw Output

```rust
let rendered = app.render_to_string();
println!("{}", rendered);  // See exactly what's rendered
```

---

## 3. Assertion Methods

### Visibility Assertions

```rust
// Panics with helpful message if not found
app.assert_visible("Expected text");

// Panics if text IS found
app.assert_not_visible("Should not appear");

// Non-panicking checks
if app.has_text("optional") {
    // ...
}
```

### Finding Content

```rust
// Find text containing substring
let found = app.find_text("partial");

// Get all text nodes
let all_text = app.find_all_text();

// Get all button labels
let buttons = app.find_all_buttons();

// Get rendered lines
let lines = app.rendered_lines();

// Find line number containing text
let line_num = app.find_line_containing("target");
```

---

## 4. Simulating Interaction

### Focus Navigation

```rust
app.focus_next();     // Tab
app.focus_prev();     // Shift+Tab
app.activate();       // Enter/Space on focused element
```

### Button Pressing

```rust
// Find button by label, focus it, activate it
app.press_button("Submit");
```

### Text Input

```rust
app.type_char('a');
app.type_str("hello");
app.backspace();
app.enter();  // For TextArea newlines
```

### List/Tree Navigation

```rust
app.list_up();
app.list_down();
```

### Scrolling

```rust
app.scroll_up(5);
app.scroll_down(5);
```

---

## 5. Testing Stateful Components

```rust
#[test]
fn test_counter_increments() {
    let mut app = TestApp::new(|cx| {
        let count = cx.use_state(|| 0);
        let c = count.clone();

        View::vstack()
            .child(View::text(format!("Count: {}", count.get())))
            .child(View::button()
                .label("+")
                .on_press(move || c.update(|n| *n += 1))
                .build())
            .build()
    });

    app.assert_visible("Count: 0");

    app.press_button("+");
    app.assert_visible("Count: 1");

    app.press_button("+");
    app.assert_visible("Count: 2");
}
```

---

## 6. Testing Unicode/Grapheme Rendering

This is critical for Telex. Wide characters (emoji, CJK) must render correctly.

### Counting Rendered Characters

```rust
#[test]
fn test_all_emojis_render() {
    let mut app = TestApp::new(|_cx| {
        View::text_area()
            .value("Hello 😊😊😊 World".to_string())
            .build()
    }).with_size(30, 5);

    let rendered = app.render_to_string();
    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(emoji_count, 3, "All emojis must render");
}
```

### Testing Wrap Boundaries

Wide characters (2 columns) at line boundaries are tricky:

```rust
#[test]
fn test_emoji_at_wrap_boundary() {
    let mut app = TestApp::new(|_cx| {
        View::text_area()
            // 26 chars + emoji (2 cols) = 28 display columns
            .value("abcdefghijklmnopqrstuvwxyz😊".to_string())
            .rows(3)
            .build()
    }).with_size(30, 6);  // content_width = 28

    let rendered = app.render_to_string();
    assert!(rendered.contains('😊'), "Emoji must survive wrapping");
}
```

### Testing Various Widths

The same content should render correctly at different terminal widths:

```rust
#[test]
fn test_content_at_multiple_widths() {
    let content = "Text with 😊 emoji";

    for width in [20, 30, 40, 50] {
        let mut app = TestApp::new(|_cx| {
            View::text_area().value(content.to_string()).build()
        }).with_size(width, 10);

        let rendered = app.render_to_string();
        assert!(rendered.contains('😊'),
            "Emoji must render at width {}", width);
    }
}
```

---

## 7. Test File Organization

```
crates/telex/tests/
  emoji_edge_cases.rs   # Unicode/grapheme comprehensive tests
  render_tests.rs       # Widget rendering (TextArea, Modal, Tabs, etc.)
  buffer_tests.rs       # Low-level buffer operations
  focus_tests.rs        # Tab navigation, modal containment
  component_tests.rs    # Component lifecycle
  keyed_state_tests.rs  # Order-independent state (state! macro)
  list_rendering_tests.rs
  theme_tests.rs
```

---

## 8. Design Philosophy

### Why Not Snapshot Tests?

Telex uses assertion-based tests rather than stored snapshots (like `insta`) because:

1. **Explicit assertions** - Tests document exactly what matters
2. **Resilient to cosmetic changes** - Border style changes don't break tests
3. **Better failure messages** - Know immediately what's wrong
4. **No snapshot maintenance** - No "update snapshots" ceremony

### What to Test

**DO test:**
- Content visibility at various sizes
- Unicode/emoji rendering (especially at boundaries)
- State changes after interaction
- Focus navigation order
- Widget-specific behavior (checkbox toggle, list selection)

**DON'T test:**
- Exact character positions (fragile)
- Specific border characters (theme-dependent)
- Pixel-perfect layout (terminals vary)

### The `render_to_string()` Pattern

When debugging, dump the rendered output:

```rust
#[test]
fn test_debugging() {
    let mut app = TestApp::new(my_component).with_size(40, 10);
    let rendered = app.render_to_string();

    println!("=== RENDERED OUTPUT ===");
    println!("{}", rendered);
    println!("=======================");

    // Now add your assertions
}
```

---

## 9. Common Patterns

### Testing Conditional Rendering

```rust
#[test]
fn test_modal_visibility() {
    // Hidden modal
    let mut app = TestApp::new(|_cx| {
        View::modal()
            .visible(false)
            .title("Hidden")
            .child(View::text("Content"))
            .build()
    });
    app.assert_not_visible("Hidden");
    app.assert_not_visible("Content");

    // Visible modal
    let mut app = TestApp::new(|_cx| {
        View::modal()
            .visible(true)
            .title("Shown")
            .child(View::text("Content"))
            .build()
    });
    app.assert_visible("Shown");
    app.assert_visible("Content");
}
```

### Testing Layout

```rust
#[test]
fn test_split_pane_renders_both() {
    let mut app = TestApp::new(|_cx| {
        View::split()
            .horizontal()
            .ratio(0.5)
            .first(View::text("Left"))
            .second(View::text("Right"))
            .build()
    }).with_size(40, 10);

    app.assert_visible("Left");
    app.assert_visible("Right");
}
```

### Testing User Workflows

```rust
#[test]
fn test_form_submission_workflow() {
    let mut app = TestApp::new(|cx| {
        let submitted = cx.use_state(|| false);
        let sub = submitted.clone();

        View::vstack()
            .child(View::text_input().placeholder("Name").build())
            .child(View::button()
                .label("Submit")
                .on_press(move || sub.set(true))
                .build())
            .child(if submitted.get() {
                View::text("Submitted!")
            } else {
                View::text("")
            })
            .build()
    });

    app.assert_not_visible("Submitted!");

    app.focus_next();  // Focus text input
    app.type_str("Alice");
    app.focus_next();  // Focus button
    app.activate();    // Press button

    app.assert_visible("Submitted!");
}
```

---

## 10. Example Compilation Check

The numbered examples in `crates/telex/examples/` (01-33) are the runnable code that the mdbook references. A test ensures they all compile when the API changes.

### Running it

```bash
cargo test -p telex-tui --test book_compiles
```

This runs `cargo build --examples` and fails if any example doesn't compile. It also runs as part of `cargo test -p telex-tui`.

### What it catches

- Renamed or removed methods, types, or traits
- Changed function signatures
- Broken imports

### Why examples, not book snippets?

The book contains ~300 Rust code blocks, but most are illustrative snippets that intentionally omit boilerplate for readability. The numbered examples are complete, runnable programs that readers actually copy — they're the right thing to protect.

## 11. Limitations

### TestApp State Sync

`TestApp` doesn't fully simulate the real event loop. Some limitations:

- `type_char`/`type_str` modify FocusManager state but may not trigger component re-renders in all cases
- For complex state interactions, test the component logic separately

### No Real Terminal

Tests run headless. You can't test:
- Actual terminal escape codes
- Color rendering
- Real keyboard input timing

For visual verification, use examples:

```bash
cargo run --example 12_text_area
```
