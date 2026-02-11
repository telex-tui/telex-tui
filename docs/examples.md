# Telex Examples

Learn Telex by example. Each example builds on the previous one.

⚠️ **Experimental Features:** Examples 29-33 demonstrate experimental features (Canvas, Image, Effects, Terminal) that are in active development with known limitations. See individual examples for details.

## Running Examples

```bash
cargo run -p telex-tui --example <name>
```

Press `Ctrl+Q` to quit any example.

---

## 01_hello_world

**The absolute minimum.** If this doesn't work, nothing will.

```bash
cargo run -p telex-tui --example 01_hello_world
```

**What it shows:**
- Basic app structure
- `Component` trait implementation
- `View::text()` for displaying text
- `View::styled_text()` with `.bold()` and `.dim()` modifiers

**Code:** 26 lines

---

## 02_counter

**State and interaction.** A counter with increment/decrement buttons.

```bash
cargo run -p telex-tui --example 02_counter
```

**What it shows:**
- `cx.use_state()` for reactive state
- `View::button()` with `on_press` callbacks
- `View::vstack()` and `View::hstack()` for layout

**Controls:**
- `Tab` - switch between buttons
- `Enter` - press the focused button

**Code:** 45 lines

---

## 03_theme_switcher

**Styling showcase.** The counter with colors and text formatting.

```bash
cargo run -p telex-tui --example 03_theme_switcher
```

**What it shows:**
- `View::styled_text()` with `.color()`, `.bold()`, `.italic()`, `.dim()`
- Conditional styling based on state (green for positive, red for negative)
- Using `crossterm::style::Color` for custom colors

**Code:** 80 lines

---

## 04_timer

**Streaming without interaction.** A timer that updates every second automatically.

```bash
cargo run -p telex-tui --example 04_timer
```

**What it shows:**
- `cx.use_stream()` for background streaming updates
- Creating an iterator with `std::thread::sleep` for timed updates
- `stream.is_loading()` to check if stream is still active
- Formatting time display (MM:SS)

**Code:** 63 lines

---

## 05_todo_list

**TextInput and List.** A simple todo app with add and delete functionality.

```bash
cargo run -p telex-tui --example 05_todo_list
```

**What it shows:**
- `View::text_input()` with `.value()`, `.placeholder()`, `.on_change()`, `.on_submit()`
- `View::list()` with `.items()`, `.selected()`, `.on_select()`
- Managing a `Vec<String>` with state
- Conditional rendering when list is empty

**Code:** 108 lines

---

## 06_log_viewer

**Streaming text.** Simulates tailing a log file with auto-updating content.

```bash
cargo run -p telex-tui --example 06_log_viewer
```

**What it shows:**
- `cx.use_text_stream()` for accumulating text over time
- `View::text_area()` for multi-line display
- Live/End status indicator based on `stream.is_loading()`

**Code:** 75 lines

---

## 07_file_browser

**Filesystem navigation.** Browse directories using a list view.

```bash
cargo run -p telex-tui --example 07_file_browser
```

**What it shows:**
- Reading filesystem with `std::fs::read_dir`
- Navigating directories on selection
- Displaying current path
- Parent directory navigation with `..`

**Code:** 102 lines

---

## 08_system_monitor

**Multiple streams.** Simulated CPU, memory, and network monitoring with progress bars.

```bash
cargo run -p telex-tui --example 08_system_monitor
```

**What it shows:**
- Multiple independent `cx.use_stream()` calls
- Color-coded progress bars based on thresholds
- Complex layout with `hstack` and `vstack`
- Pseudo-random number generation for simulation

**Code:** 130 lines

---

## 09_syntax_comparison

**Two syntaxes.** Shows builder style vs view! macro (when available).

```bash
cargo run -p telex-tui --example 09_syntax_comparison
```

**What it shows:**
- Builder pattern: `View::vstack().child(...).build()`
- Both approaches produce identical output

---

## 10_state_explained

**State deep dive.** Explains how `use_state` works with visual examples.

```bash
cargo run -p telex-tui --example 10_state_explained
```

**What it shows:**
- State initialization and updates
- How state persists across renders
- The `share!` macro for capturing state in callbacks

---

## 11_checkbox

**Toggle controls.** Demonstrates checkbox widgets with state binding.

```bash
cargo run -p telex-tui --example 11_checkbox
```

**What it shows:**
- `View::checkbox()` with `.checked()`, `.label()`, `.on_toggle()`
- Managing boolean state
- Multiple checkboxes with independent state

---

## 12_text_area

**Multi-line input.** Text area with cursor navigation.

```bash
cargo run -p telex-tui --example 12_text_area
```

**What it shows:**
- `View::text_area()` with `.value()`, `.placeholder()`, `.on_change()`
- Multi-line text editing
- Cursor line/column tracking

---

## 13_split_panes

**Resizable layouts.** Split panes for lazygit-style UIs.

```bash
cargo run -p telex-tui --example 13_split_panes
```

**What it shows:**
- `View::split()` with `.horizontal()` or `.vertical()`
- `.ratio()` for proportional sizing
- `.min_first()` / `.min_second()` for minimum sizes
- Nested splits for complex layouts
- `.show_divider()` to toggle divider visibility

**Code:** 95 lines

---

## 14_tabs

**Tabbed interfaces.** Multi-view apps with keyboard navigation.

```bash
cargo run -p telex-tui --example 14_tabs
```

**What it shows:**
- `View::tabs()` with `.tab(label, content)` API
- `.active()` and `.on_change()` for tab state
- Keyboard navigation: Left/Right arrows, `[`/`]` keys, number keys 1-9
- `.position(TabPosition::Top)` or `TabPosition::Bottom`

**Code:** 95 lines

---

## 15_markdown

**Rich text rendering.** Markdown to terminal UI.

```bash
cargo run -p telex-tui --example 15_markdown
```

**What it shows:**
- `markdown::render()` for markdown content
- Headers, lists, code blocks, emphasis
- Syntax highlighting in code blocks

---

## 16_tree

**Hierarchical navigation.** File-tree style navigation.

```bash
cargo run -p telex-tui --example 16_tree
```

**What it shows:**
- `View::tree()` with `TreeItem` structure
- Expand/collapse with arrow keys
- Selection callbacks

---

## 17_table

**Data tables.** Sortable columns with headers.

```bash
cargo run -p telex-tui --example 17_table
```

**What it shows:**
- `View::table()` with columns and rows
- Sortable headers
- Row selection

---

## 18_progress_bar

**Visual progress.** Progress indicators with labels.

```bash
cargo run -p telex-tui --example 18_progress_bar
```

**What it shows:**
- `View::progress_bar()` with `.progress()`, `.label()`
- Color theming based on progress value

---

## 19_status_bar

**Bottom info line.** Status bar with sections.

```bash
cargo run -p telex-tui --example 19_status_bar
```

**What it shows:**
- `View::status_bar()` with left/center/right sections
- Dynamic content updates

---

## 20_menu_bar

**Menu bar with keyboard navigation.** Traditional dropdown menus.

```bash
cargo run -p telex-tui --example 20_menu_bar
```

**What it shows:**
- `View::menu_bar()` with dropdowns
- Full keyboard navigation (Tab, Enter, arrows, Escape)
- Keyboard shortcut registration

---

## 21_toasts

**Notifications.** Ephemeral toast messages.

```bash
cargo run -p telex-tui --example 21_toasts
```

**What it shows:**
- `View::toast_container()` for displaying toasts
- Toast types: info, success, warning, error
- Auto-dismiss timing

---

## 22_forms

**Form validation.** Declarative field validation.

```bash
cargo run -p telex-tui --example 22_forms
```

**What it shows:**
- `View::form()` with `View::form_field()`
- Validation rules and error display
- Submit handling

---

## 23_modal

**Modal dialogs.** Overlay dialogs with focus trapping.

```bash
cargo run -p telex-tui --example 23_modal
```

**What it shows:**
- `View::modal()` with `.visible()`, `.title()`, `.child()`
- Focus containment within modal
- Close on Escape

---

## 24_async_data

**Async operations.** Loading states and async data fetching.

```bash
cargo run -p telex-tui --example 24_async_data
```

**What it shows:**
- `cx.use_async()` for async operations
- Loading/Ready/Error state handling

---

## 25_context

**Shared context.** Passing data through the component tree.

```bash
cargo run -p telex-tui --example 25_context
```

**What it shows:**
- `cx.use_context()` and `cx.provide_context()`
- Theme/config propagation without prop drilling

---

## 26_radio_buttons

**Mutually exclusive options.** Radio button groups.

```bash
cargo run -p telex-tui --example 26_radio_buttons
```

**What it shows:**
- `View::radio_group()` with options
- Single selection enforcement
- Change callbacks

---

## 27_keyed_state

**Order-independent state.** The `state!` macro for conditional hooks.

```bash
cargo run -p telex-tui --example 27_keyed_state
```

**What it shows:**
- `state!(cx, || init)` macro for order-independent state
- Safe use of state inside conditionals (unlike `use_state`)
- Each `state!` call gets its own unique state based on code location
- The `with!` macro for callback closure capture

**The problem with use_state:**
```rust
// WRONG - This panics if show_counter changes!
if show_counter {
    let count = cx.use_state(|| 0);  // Index shifts based on condition
}
```

**The solution with state!:**
```rust
// SAFE - This works!
if show_counter {
    let count = state!(cx, || 0);  // Key is baked into the code location
}
```

**Code:** 182 lines

---

## 28_shared_state

**Shared state across components.** Using explicit keys for state sharing.

```bash
cargo run -p telex-tui --example 28_shared_state
```

**What it shows:**
- `cx.use_state_keyed::<Key, _>(|| init)` for explicitly keyed state
- Same key type = same state (shared across call sites)
- Contrast with `state!` which creates independent state per call site

**How it works:**
```rust
// Define a named key type
struct SharedCounterKey;

// Pane A - gets shared counter
let count_a = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);

// Pane B - uses SAME key = SAME state!
let count_b = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);
// count_a and count_b point to the same underlying value
```

**Code:** 177 lines

---

## 29_canvas ⚠️ Experimental

**Pixel-level graphics.** Canvas widget using the Kitty graphics protocol.

```bash
cargo run -p telex-tui --example 29_canvas
```

**What it shows:**
- `View::canvas()` for pixel-level drawing
- `DrawContext` API: `clear()`, `line()`, `fill_rect()`, `stroke_rect()`, `circle()`, `fill_circle()`
- Kitty graphics protocol for actual pixel rendering
- Animated bar chart with streaming updates

**Requirements:**
- Kitty-protocol compatible terminal (Kitty, Ghostty, WezTerm)
- In unsupported terminals, shows a placeholder message

**Code:** 140 lines

---

## 30_image ⚠️ Experimental

**Image display.** Display images using the Kitty graphics protocol.

```bash
cargo run -p telex-tui --example 30_image
```

**What it shows:**
- `View::image()` for displaying image files
- `.file("path")` for runtime file loading
- `.data(include_bytes!(...))` for compile-time embedding
- PNG, JPEG, and GIF support (Kitty handles GIF animation natively)

**Requirements:**
- Kitty-protocol compatible terminal (Kitty, Ghostty, WezTerm)
- In unsupported terminals, shows empty space

**Code:** 45 lines

---

## 31_animated_canvas ⚠️ Experimental

**Frame-based animation.** Animated canvas with automatic frame timing.

```bash
cargo run -p telex-tui --example 31_animated_canvas
```

**What it shows:**
- `animated_canvas(cx)` helper for frame-based animations
- `.fps(30)` for configurable frame rate
- `.on_frame(|ctx, frame| {...})` callback with frame number
- Bouncing ball physics animation
- Sine wave visualization
- Particle fountain effect

**Requirements:**
- Kitty-protocol compatible terminal (Kitty, Ghostty, WezTerm)
- In unsupported terminals, shows empty space

**Code:** 165 lines

---

## 32_effects

**Side effects.** Demonstrates the effect macros for running code in response to state changes.

```bash
cargo run -p telex-tui --example 32_effects
```

**What it shows:**
- `effect_once!(cx, || ...)` - runs only on first render (initialization)
- `effect!(cx, deps, |d| ...)` - runs when dependencies change
- Live "Last effect" indicator showing which effect ran

**Code:** 91 lines

---

## 33_terminal ⚠️ Experimental

**Interactive PTY terminal emulator.** Run shell commands and CLI applications inside your TUI.

```bash
cargo run -p telex-tui --example 33_terminal
```

**What it shows:**
- `cx.use_terminal()` hook for terminal handle management
- `terminal.spawn(command, args, cols, rows)` to start a PTY process
- `View::terminal()` widget with border and title
- Full keyboard passthrough (all keys sent to PTY except Ctrl+Shift+[)
- ANSI escape sequence rendering (colors, styles, cursor movement)

**Use cases:**
- Running shell commands (bash, zsh)
- Text editors (vim, nano)
- System monitors (htop, top)
- AI agent CLIs
- Recursive TUI (run Telex inside Telex!)

**Keyboard shortcuts:**
- **Ctrl+Shift+[** - Escape terminal focus (return to TUI navigation)
- **Tab** - Navigate to next widget
- **All other keys** - Sent directly to the PTY

**Known limitations:**
- ❌ No scrollback buffer
- ❌ No terminal resize support
- ❌ No copy/paste
- ❌ No mouse input

**Code:** 30 lines

---

## Example Apps

Beyond the numbered examples, there are complete application examples:

### chat

AI chat application with streaming support for multiple LLM providers.

```bash
cargo run -p chat
```

**Supports:** Anthropic, OpenAI, Google Gemini, Ollama (local)

### layout-playground

Interactive layout experimentation tool.

```bash
cargo run -p layout-playground
```

### perf-test

Performance testing for render benchmarks.

```bash
cargo run -p perf-test
```
