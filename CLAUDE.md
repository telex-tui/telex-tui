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
      scope.rs     # Hook storage (indexed and keyed state)
      text.rs      # Grapheme-aware text wrapping utilities
    examples/      # Numbered examples (01-32)
  telex-macro/     # view!, state!, effect!, effect_once!, with! proc macros
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
   - `cx.use_state(|| init)` - Legacy index-based (React-style), must maintain call order

4. **Effects via macros (preferred):**
   - `effect!(cx, deps, |&d| { ... || {} })` - Runs when deps change, order-independent
   - `effect_once!(cx, || { ... || {} })` - Runs once on first render, order-independent
   - Legacy `cx.use_effect_*` APIs exist but are index-based

5. **Callbacks as Rc<dyn Fn()>** - Cloneable function pointers for event handlers.

6. **Focus management** - Modal focus containment, initial focus support, cursor navigation.

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
- Single-threaded (Rc, not Arc) - TUI apps don't need threads

## New in This Version

- **Keyed state** - `state!` macro for order-independent state (examples 27, 28)
- **Keyed effects** - `effect!` and `effect_once!` macros for order-independent effects (example 32)
- **Effect cycle detection** - Automatic detection of infinite loops in effects
- **Modal focus containment** - Focus stays within visible modals, restores when closed
- **Initial focus** - TextInput supports `focused: true` for initial focus
- **Cursor navigation** - TextInput left/right, TextArea up/down/left/right with `on_cursor_change`
- **`with!` macro** - Cleaner callback state capture
- **Menu bar improvements** - Full keyboard navigation (Tab, Enter, arrows, Escape)
- **Focus visibility** - Focus indicators hidden until user starts keyboard navigation (`focus_visible` flag)
- **Overlay rendering** - Menu dropdowns render above content (z-order fix)

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

From external review - documented issues to address:

1. ~~**No `use_effect`**~~ - ✅ Implemented! See example 32_effects
2. **No escape hatch** - Can't do custom character-cell rendering (Canvas covers pixel graphics)
3. **No panic boundaries** - Callback panic crashes app
4. **Minimal devtools** - `TELEX_DEBUG=1` shows focus only, no state inspection
5. **No type-safe required props** - Builders don't enforce required fields at compile time
