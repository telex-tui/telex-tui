# Blue Sky Ideas

Future directions and ideas for Telex.

---

## Current Widget Coverage

What we have and where it's showcased:

| Widget | Constructor | Examples |
|--------|-------------|----------|
| Text | `View::text()` | 01-19 (all) |
| StyledText | `View::styled_text()` | 01-19 (all) |
| VStack | `View::vstack()` | 01-19 (all) |
| HStack | `View::hstack()` | 02, 03, 04, 06, 08, 09, 10, 11, 12 |
| Button | `View::button()` | 02, 05, 09, 10 |
| Box | `View::boxed()` | 03, 06, 09, 10, 11, 13 |
| Spacer | `View::spacer()` | 01-19 (all) |
| List | `View::list()` | 03, 05, 07, 13 |
| TextInput | `View::text_input()` | 05 |
| TextArea | `View::text_area()` | 12 |
| Checkbox | `View::checkbox()` | 11, 14 |
| Modal | `View::modal()` | 07 |
| Split | `View::split()` | 13 |
| Tabs | `View::tabs()` | 14 |
| Markdown | `markdown::render()` | 15 |
| Tree | `View::tree()` | 16 |
| Table | `View::table()` | 17 |
| ProgressBar | `View::progress_bar()` | 18 |
| StatusBar | `View::status_bar()` | 19 |
| CommandPalette | `View::command_palette()` | 20 |
| MenuBar | `View::menu_bar()` | 20 |
| ToastContainer | `View::toast_container()` | 21 |
| Form | `View::form()` | 22 |
| FormField | `View::form_field()` | 22 |
| Modal | `View::modal()` | 23 (dedicated) |
| RadioGroup | `View::radio_group()` | 26 |
| Keyed State | `state!`, `use_state_keyed` | 27, 28 |
| Canvas ⚠️ | `View::canvas()` | 29 |
| Image ⚠️ | `View::image()` | 30 |
| AnimatedCanvas ⚠️ | `animated_canvas(cx)` | 31 |
| Effects | `effect!`, `effect_once!` | 32 |
| Terminal ⚠️ | `View::terminal()`, `cx.use_terminal()` | 33 |

⚠️ = Experimental feature (in active development with known limitations)

---

## Gap Analysis: Building Real Apps

What's missing to build apps like lazygit, btop, k9s, yazi?

### Per-App Requirements

**lazygit** (Git TUI):
- Split panes (resizable panels)
- Tabs (branches, stashes, etc.)
- Tree view (file tree, commit graph)
- Diff viewer (syntax highlighted)
- Search/filter in lists
- Keyboard shortcut hints overlay
- Status bar (bottom info line)
- Context menus / command palette

**btop** (System monitor):
- Graphs (line charts, sparklines)
- Progress bars
- Gauges
- Multi-panel grid layout
- Color gradients

**k9s** (Kubernetes TUI):
- Table (sortable columns, headers)
- Tabs
- Breadcrumbs
- Search/filter
- Tree view (resource hierarchy)

**yazi** (File manager):
- Miller columns (3-pane file browser)
- Tree view
- Tabs
- Preview pane (syntax highlighted)
- Status bar
- Breadcrumbs

### Priority Matrix

| Component | Needed By | Priority | Status |
|-----------|-----------|----------|--------|
| **Tabs** | lazygit, k9s, yazi | High | ✅ Done |
| **Table** | k9s, lazygit | High | ✅ Done |
| **Tree view** | lazygit, k9s, yazi | High | ✅ Done |
| **Split panes** | lazygit, yazi | High | ✅ Done |
| **Progress bar** | btop, any loading UI | Medium | ✅ Done |
| **Status bar** | All of them | Medium | ✅ Done |
| **Search/filter** | lazygit, k9s | Medium | 🔲 Next |
| **Canvas (pixels)** | btop, visualizations | Medium | ✅ Done |
| **Graphs/sparklines** | btop | Medium | 🔲 Planned (via Canvas) |
| **Image display** | yazi, media apps | Medium | ✅ Done |
| **Breadcrumbs** | k9s, yazi | Low | 🔲 Planned |
| **Syntax highlighting** | lazygit, yazi | Low | 🔲 Planned |
| **Command palette** | lazygit | Low | ✅ Done |

**The Big Four + essentials are done!** All showcase app types now have core widget coverage. Next: Search/filter or a showcase app.

---

## Missing Widgets

### The Big Four (High Priority)

These unlock most serious applications:

| Widget | Description | Enables | Status |
|--------|-------------|---------|--------|
| **Tabs** | Tabbed interface | Multi-view apps like lazygit, k9s | ✅ Done (example 14) |
| **Table** | Sortable columns, headers | Data-heavy apps like k9s | ✅ Done (example 17) |
| **Tree view** | Hierarchical navigation | File browsers, configs, git trees | ✅ Done (example 16) |
| **Split panes** | Resizable panels | lazygit-style layouts | ✅ Done (example 13) |

**All Big Four are complete!** The widget gap vs Ratatui is largely closed.

### Essential

| Widget | Description | Use Case | Status |
|--------|-------------|----------|--------|
| **Progress bar** | Visual progress indicator | Operations, loading, btop-style | ✅ Done |
| **Status bar** | Bottom info line | Most serious apps have one | ✅ Done |
| **Menu / MenuBar** | Dropdown menus, context menus | App navigation, actions | ✅ Done |
| **Command Palette** | Fuzzy search command execution | Power user workflows | ✅ Done |
| **Toast notifications** | Ephemeral messages | Feedback, alerts | ✅ Done |
| **Form validation** | Declarative field validation | Settings, data entry | ✅ Done |
| **Radio buttons** | Mutually exclusive options | Settings, forms | ✅ Done |
| **Keyed state** | Order-independent hooks | Conditional state | ✅ Done |
| **Canvas** | Pixel-level drawing (Kitty protocol) | Graphs, visualizations | ⚠️ Experimental |
| **Image** | Display PNG/GIF/JPEG (Kitty protocol) | Media, previews | ⚠️ Experimental |
| **AnimatedCanvas** | Frame-based animation | Games, visualizations | ⚠️ Experimental |
| **Terminal** | Interactive PTY emulator | Shell, vim, agent CLIs | ⚠️ Experimental |
| **Effects** | Side effects and cleanup | Timers, lifecycle | ⚠️ Experimental |
| **Select/Dropdown** | Collapsed list picker | Compact selection | 🔲 Planned |

### Nice to Have

- Search/filter (inline list filtering)
- ~~Toast notifications (temporary messages)~~ ✅ Done
- Breadcrumbs (navigation path)
- ~~Graphs/sparklines (btop-style)~~ → Use Canvas widget
- Spinner/loading indicator
- Syntax highlighting
- ~~Command palette~~ ✅ Done
- ~~Canvas (pixel drawing)~~ ✅ Done

---

## Demo App Ideas

Apps that could showcase Telex and "capture hearts":

### Arch Config Tool
- **Audience**: Passionate Arch Linux community
- **Why**: Would trend on r/archlinux, real utility
- **Exercises**: Menus, tabs, lists, checkboxes, progress bars
- **Risk**: Niche audience

### Dotfiles Manager
- **Audience**: All developers
- **Why**: Universal appeal, practical problem
- **Exercises**: File browser, text editing, diff view
- **Risk**: Less visual wow factor

### Git TUI
- **Audience**: Huge (lazygit has 56k+ stars)
- **Why**: Proven demand
- **Exercises**: Everything - complex UI
- **Risk**: Crowded space, significant effort

### SSH/Server Manager
- **Audience**: DevOps, sysadmins
- **Why**: TUIs are natural fit for terminal work
- **Exercises**: Lists, forms, real-time updates
- **Risk**: Requires networking

### AI Chat Interface
- **Audience**: Everyone (trendy)
- **Why**: Visual, real-time streaming, modern
- **Exercises**: TextArea, streaming, auto-scroll
- **Risk**: Many already exist

### System Bootstrap Tool
- **Audience**: All developers
- **Why**: "Configure your dev environment" - broad appeal
- **Exercises**: Same as Arch tool but universal
- **Risk**: Less passionate niche

---

## Developer Experience Ideas

### High Impact

1. **Hot Reload** - Change code, see it instantly
   - `cargo-watch` + fast restart
   - Scripting layer (Lua/Rhai) for hot-reloadable views
   - State persistence across restarts

2. **Component Inspector** - `Ctrl+D` overlay showing:
   - Component tree
   - Current state values
   - Focus chain
   - Render boundaries

3. **Time-Travel Debugging** - Record state changes, step through history

4. **Better Macro Errors** - Invest in proc-macro diagnostics

### Medium Effort

- Component gallery (`cargo run --example gallery`)
- `telex new` CLI scaffolding
- Web-based playground (via WASM)
- Recorded demos (asciinema in docs)

### Experimental

- AI describe-to-code
- ASCII art / Figma import
- Live REPL for runtime state manipulation

---

## Mouse Support

Currently keyboard-only. Adding mouse support is a tiered effort.

### Current State

- Mouse capture is **not enabled**
- Only `Event::Key` and `Event::Resize` handled
- No hit-testing infrastructure

### Implementation Tiers

#### Tier 1: Scroll Wheel (Easy - 1 hour)

```rust
// terminal.rs - enable mouse capture
use crossterm::event::{EnableMouseCapture, DisableMouseCapture};
execute!(stdout, EnterAlternateScreen, Hide, EnableMouseCapture)?;

// lib.rs - handle scroll events
Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollUp, .. }) => {
    focus.scroll_up(3);
}
```

No hit-testing needed. Scroll wheel affects the focused widget.

#### Tier 2: Click-to-Focus (Medium - half day)

Requires **hit-testing infrastructure**:

```rust
struct HitBox {
    x: u16, y: u16, width: u16, height: u16,
    focus_index: usize,
}

// Collected during render, searched on click
struct HitTestRegistry {
    boxes: Vec<HitBox>,
}
```

During render, each focusable widget records its bounding box. On click, find which box contains (x, y) and set focus.

#### Tier 3: Widget Interactions (Hard - 1-2 days)

Full mouse support for each widget type:

| Widget | Mouse Interaction |
|--------|-------------------|
| Button | Click to activate |
| Checkbox | Click to toggle |
| List/Tree/Table | Click row to select |
| TextInput/TextArea | Click to position cursor |
| Split | Drag divider to resize |
| Tabs | Click tab to switch |
| Scrollable Box | Click scrollbar / drag |

Each needs widget-specific click handling beyond just focus.

### The Core Challenge

Render is currently "fire and forget" - we don't track where widgets end up. Options:

1. **Record hitboxes during render** - Each focusable records its rect
2. **Re-traverse view tree on click** - Walk tree with layout info to find (x, y)
3. **Focus-based only** - Only handle mouse on currently focused widget (limited)

Option 1 is cleanest but requires threading a `HitTestRegistry` through render.

### Recommendation

Start with **Tier 1 (scroll wheel)** - immediate value, no infrastructure changes. Then tackle Tier 2 as a separate focused effort.

---

## Visual Layout Tool

Idea: A tool to visually design layouts and generate code.

**Potential value:**
- Lower barrier for new users
- Quick scaffolding of static layouts
- Learning tool

**Challenges:**
- Dynamic content (state, callbacks) is the hard part - tool only generates skeleton
- TUI layouts are simpler than GUI - less complexity to manage
- Terminal size variability
- Maintenance burden

**Options if pursued:**
- TUI tool (dogfooding)
- Web-based designer
- Interactive CLI wizard

**Verdict:** Nice-to-have, not essential. The API is already readable.

---

## Open Question: Widget Styling API

A fundamental design tension: how should widgets support styling?

### Current Pattern

```rust
View::text("simple")                           // Quick, no options
View::styled_text("fancy").color(...).build()  // Full control
```

This works for Text. But what about Button, List, Checkbox, etc.?

### The Options

| Approach | Pros | Cons |
|----------|------|------|
| **One widget, many options** | Single concept to learn | Bloated API, simple case gets noisy |
| **Simple + Styled variants** | Clean defaults, opt-in complexity | Proliferation: StyledButton, StyledList... |
| **Theme-based only** | Widgets stay simple | Less per-instance control |
| **Semantic variants** | `Button::primary()`, `Button::danger()` | Opinionated, may not fit all apps |
| **Style prop** | `.style(ButtonStyle { ... })` | Extra type, indirection |

### The Deeper Question

**Who is Telex for?**

- **Quick prototyping** → Keep widgets dead simple, theming handles aesthetics
- **Polished apps** → Need per-instance styling control
- **Both** → Simple/styled split works, but repeats across all widgets

### Current Stance

Wait and see. The current `[ button ]` works. Let real-world usage reveal what people actually need before committing to an API pattern.

### What Buttons Could Have (If Needed)

```rust
// Option A: Styled variant
View::styled_button()
    .label("Save")
    .color(Color::Green)
    .variant(ButtonVariant::Filled)
    .disabled(is_saving)
    .build()

// Option B: Style prop
View::button()
    .label("Save")
    .style(ButtonStyle::primary().disabled(is_saving))
    .build()

// Option C: Semantic constructors
View::button_primary("Save").on_press(save).build()
View::button_danger("Delete").on_press(delete).build()
```

No decision yet. Needs more real usage to inform.

---

## Competitive Analysis: Ratatui

Honest assessment of "Ratatui is better than Telex because..."

### Valid Criticisms

| Criticism | Reality |
|-----------|---------|
| **"More widgets"** | Ratatui has Sparkline, BarChart, Gauge, Canvas. Telex now has Table, Tabs, Tree, Split. Gap is narrowing. |
| **"Battle-tested"** | gitui, bottom, many production apps. Telex is new. |
| **"Bigger ecosystem"** | More examples, tutorials, third-party crates, StackOverflow answers |
| **"Better docs"** | Ratatui has a book, extensive examples, comprehensive API docs |
| **"Backend agnostic"** | Ratatui supports crossterm, termion, termwiz. Telex is crossterm-only. |

### Trade-offs (Not Strictly Better/Worse)

| Point | Ratatui | Telex |
|-------|---------|-------|
| **Architecture** | Immediate mode, no framework | Retained mode, React-style components |
| **Control** | Cell-by-cell when needed | Higher-level abstractions |
| **Flexibility** | No opinions, you decide everything | Opinionated component model |
| **State management** | DIY | Built-in hooks (`use_state`, `use_effect`, `use_stream`) |
| **Boilerplate** | More manual wiring | Less code for stateful UIs |

### Where Telex Could Counter

- Simpler mental model for UI (declarative, reactive)
- Less boilerplate for stateful, interactive UIs
- Familiar to React/web developers
- `share!` and `view!` macros reduce noise
- Auto-diffing (only re-render what changed)

### Where Telex is Genuinely Behind

1. ~~**Widget count** - The Big Four (Tabs, Table, Tree, Split panes) are missing~~ ✅ All done!
2. **Maturity** - No production apps yet
3. **Documentation** - Needs a proper guide/book
4. **Community** - No ecosystem yet
5. ~~**Missing widgets** - Gauge, Sparkline, Canvas (btop-style visualization)~~ Canvas ✅ Done (Kitty graphics)
6. **Mouse support** - Keyboard-only currently

### The Bet

Telex bets that the React model is worth the trade-off for many use cases - especially apps with complex state, real-time updates, and interactive UIs.

Progress on making that bet pay off:
- ✅ The Big Four widget gaps are closed (Tabs, Table, Tree, Split)
- ✅ Essential widgets done (Progress bar, Status bar, MenuBar, Command Palette)
- ✅ Nice-to-haves done (Toast notifications, Form validation)
- 🔲 Mouse support (scroll wheel is easy, click-to-focus needs hit-testing)
- 🔲 Documentation needs work (guide/book)
- 🔲 At least one "showcase" app to prove it out
- 🔲 Visualization widgets (Sparkline, Gauge) for btop-style apps
- ⚠️ Canvas widget - implemented (experimental, Kitty protocol only)
- ⚠️ Image widget - implemented (experimental, Kitty protocol only)
- ⚠️ Terminal widget - implemented (experimental, missing scrollback/resize/copy-paste)
- ⚠️ `use_effect` API - implemented (experimental, newly added)
