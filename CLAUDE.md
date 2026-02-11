# Telex - Claude Code Project Guide

Telex is a DX-first TUI framework for Rust, inspired by React's component model.

## Project Structure

```
crates/
  telex/           # Main framework library
    src/
      lib.rs       # Entry point, run loop, event handling
      view.rs      # View enum and all widget types
      buffer.rs    # Terminal cell buffer with unicode handling
      render.rs    # View tree -> buffer rendering
      focus.rs     # Tab navigation, focus management, modal containment
      state.rs     # State<T> with Rc<RefCell<T>>
      scope.rs     # Hook storage (keyed state, channels, intervals)
      channel.rs   # Channel/port primitives for external events
      widget.rs    # Custom widget trait (escape hatch)
      text.rs      # Grapheme-aware text wrapping utilities
    examples/      # Numbered examples (01-33)
  telex-macro/     # view!, state!, effect!, effect_once!, with!, channel!, port!, interval!, reducer!, stream!, async_data!, terminal!, text_stream! proc macros
examples/
  chat/            # AI chat app (multi-provider: Anthropic, OpenAI, Gemini, Ollama)
  layout-playground/  # Interactive layout experimentation
  layout-showcase/    # Layout demonstrations
  perf-test/       # Performance testing
docs/              # Architecture documentation
  architecture.md    # Design rationale
  canvas-design.md   # Canvas widget design (Kitty protocol)
  use-effect-design.md # use_effect API design
  roadmap.md         # Blue sky ideas and gap analysis
  testing.md         # Testing guide
  examples.md        # Examples documentation
```

## Key Concepts

1. **View is an enum** - Not trait objects. Pattern matching, no vtable overhead.

2. **State via Rc<RefCell<T>>** - Cheap cloning for closures, interior mutability.

3. **State via macros (preferred):**
   - `state!(cx, || init)` - Keyed by code location, order-independent, safe in conditionals
   - `cx.use_state_keyed::<Key, _>(|| init)` - Explicit key for shared state across call sites

4. **Effects via macros (preferred):**
   - `effect!(cx, deps, |&d| { ... || {} })` - Runs when deps change, order-independent
   - `effect_once!(cx, || { ... || {} })` - Runs once on first render, order-independent

5. **External events:**
   - `channel!(cx, T)` - Typed inbound channel; returns `ChannelHandle<T>` with `tx()` (waking sender) and `get()` (this frame's messages)
   - `port!(cx, In, Out)` - Bidirectional port; inbound `ChannelHandle<In>` + outbound `Sender<Out>`
   - `interval!(cx, Duration, || callback)` - Periodic timer; callback runs on main thread each frame the timer fires
   - All channel senders use `WakingSender` which wakes the event loop for near-zero latency

6. **Reducer pattern:**
   - `reducer!(cx, initial, |state, action| { ... })` - Returns `(State<S>, Rc<dyn Fn(A)>)` dispatch pair

7. **Callbacks as Rc<dyn Fn()>** - Cloneable function pointers for event handlers.

8. **Focus management** - Modal focus containment, initial focus support, cursor navigation.

9. **Error boundaries** - `View::error_boundary().child(risky).fallback(safe).build()` catches panics in child views.

10. **Custom widgets** - `View::custom(widget)` escape hatch for user-defined character-cell rendering via the `Widget` trait.

## Unicode/Grapheme Handling

This is critical - terminals display characters in columns, but Unicode is complex:

- **Grapheme clusters**: User-perceived characters (e.g., emoji, combining chars)
- **Display width**: ASCII = 1 column, emoji/CJK = 2 columns
- **Continuation cells**: Wide chars occupy 2 cells; second cell marked `wide_continuation: true`

Key files:
- `buffer.rs`: `Cell.wide_continuation`, `write_str()` with grapheme iteration
- `text.rs`: `graphemes()`, `grapheme_width()`, `wrapped_height()`
- `render.rs`: TextArea cursor positioning in grapheme coordinates

Dependencies: `unicode-segmentation`, `unicode-width`

## Common Tasks

**Run a numbered example:**
```bash
cargo run -p telex-tui --example 02_counter
```

**Run an example app:**
```bash
cargo run -p chat          # AI chat app
cargo run -p layout-playground  # Layout experimentation
cargo run -p perf-test     # Performance testing
```

**Run all examples (interactive selector):**
```bash
./run-examples.sh
```

**Run tests:**
```bash
cargo test -p telex-tui
```

**Debug mode (shows render timing):**
```bash
TELEX_DEBUG=1 cargo run -p telex-tui --example 02_counter
```

## Testing

Tests use `TestApp` - a harness that renders to an in-memory buffer:

```rust
use telex::testing::TestApp;

let mut app = TestApp::new(|cx| {
    View::text_area().value("Hello 😊".to_string()).build()
}).with_size(30, 10);

app.assert_visible("Hello");
app.assert_visible("😊");
app.assert_not_visible("Goodbye");
```

Key test files:
- `tests/emoji_edge_cases.rs` - Comprehensive unicode/grapheme tests
- `tests/render_tests.rs` - Widget rendering tests
- `tests/focus_tests.rs` - Navigation tests

See `docs/testing.md` for the full testing guide.

## Architecture Deep Dives

See `docs/architecture.md` for detailed design rationale covering:
- Why View is an enum (not traits)
- State<T> and Rc<RefCell> trade-offs
- Hook storage and call-order dependency
- Focus management
- Double buffering and cell diffing

## Conventions

- Widgets use builder pattern: `View::button().label("Click").on_press(cb).build()`
- The `view!` macro provides JSX-like syntax as sugar over builders
- The `state!` macro provides order-independent state (preferred over `use_state`)
- The `effect!` macro provides order-independent effects: `effect!(cx, deps, |&d| { ... || {} })`
- The `effect_once!` macro runs once: `effect_once!(cx, || { ... || {} })`
- The `with!` macro captures state for callbacks: `with!(count => move || count.update(|n| *n += 1))`
- All state mutation goes through `State::update()` or `State::set()`
- Single-threaded render (Rc, not Arc) - external threads communicate via channels/ports
- The `channel!`, `port!`, `interval!`, `stream!`, `async_data!`, `text_stream!`, `terminal!`, `reducer!` macros all generate unique keys (order-independent, safe in conditionals)

## New in This Version (API 0.2)

- **Index-based hooks removed** - All hooks are now keyed via macros (`state!`, `effect!`, `channel!`, etc.). No more call-order dependency.
- **Channels & ports** - `channel!(cx, T)` and `port!(cx, In, Out)` for external event sources with frame-buffered message delivery
- **WakingSender** - Channel senders wake the event loop instantly (near-zero latency vs 16ms polling)
- **Error boundaries** - `View::error_boundary()` catches panics in child views and renders a fallback
- **Reducer** - `reducer!(cx, initial, |s, a| ...)` for action-dispatched state management
- **Custom widgets** - `View::custom(widget)` with the `Widget` trait for escape-hatch character-cell rendering
- **Slider widget** - `View::slider()` for bounded numeric values with left/right arrow key input
- **Interval timer** - `interval!(cx, duration, || callback)` sugar over channel + timer thread
- **Dirty render skip** - Run loop skips frames when nothing changed (no input, no channel data, no effects)
- **Component identity** - `Scope` carries an optional `TypeId` for future memoization support
- **Effect cycle detection** - Automatic detection of infinite loops in effects
- **Modal focus containment** - Focus stays within visible modals, restores when closed
- **Cursor navigation** - TextInput left/right, TextArea up/down/left/right with `on_cursor_change`
- **`with!` macro** - Cleaner callback state capture
- **Menu bar improvements** - Full keyboard navigation (Tab, Enter, arrows, Escape)
- **Focus visibility** - Focus indicators hidden until user starts keyboard navigation (`focus_visible` flag)

## Experimental Features

⚠️ **These features are in active development and marked as experimental:**

- **Terminal widget** *(example 33)* - Interactive PTY emulator for running shell commands, vim, htop, etc. Missing: scrollback, resize, copy/paste
- **Canvas** *(examples 29, 31)* - Pixel-level drawing using Kitty graphics protocol. Requires Kitty/Ghostty/WezTerm
- **Image** *(example 30)* - Display PNG/JPEG/GIF using Kitty graphics protocol. GIF animations supported

## Design Documents

- **`docs/canvas-design.md`** - Pixel-level drawing using Kitty graphics protocol. Supports Kitty, Ghostty, WezTerm. No fallback - use a modern terminal. *(⚠️ Implemented - Experimental)*
- **`docs/use-effect-design.md`** - Effect API design (`effect!`, `effect_once!` macros) for side effects, timers, cleanup. *(✅ Implemented)*
- **Terminal widget** *(example 33)* - Interactive PTY terminal emulator. *(⚠️ Implemented - Experimental)*

## Known Gaps

1. ~~**No `use_effect`**~~ - ✅ Implemented! See example 32_effects
2. ~~**No escape hatch**~~ - ✅ Implemented! `View::custom(widget)` with the `Widget` trait
3. ~~**No panic boundaries**~~ - ✅ Implemented! `View::error_boundary()` catches panics
4. **Minimal devtools** - `TELEX_DEBUG=1` shows focus only, no state inspection
5. **No type-safe required props** - Builders don't enforce required fields at compile time
