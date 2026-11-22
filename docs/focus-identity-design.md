# Focus Identity Design

> **Status: ✅ Implemented**: release `0.4.0`. See `crates/telex/src/focus.rs` and
> `crates/telex/tests/focus_identity_tests.rs`

Stable identity for focusable widgets, so focus, focus requests, and per-widget state
(scroll, cursor) follow *the widget* rather than *its position in the focus list*. Also
settles which gets a key first: the focused widget or an app-registered command.

## Quick Start

```rust
fn render_review(cx: &Scope /* ... */) -> View {
    View::vstack()
        .child(
            View::boxed()
                .key("review-log")               // own scroll state, reset when it goes away
                .scroll(true)
                .child(View::text(log))
                .build(),
        )
        .child(
            View::button()
                .key("review-confirm")
                .label("Confirm")
                .focused(true)                   // focused (and shown focused) on entering this screen
                .on_press(confirm)
                .build(),
        )
        .build()
}
```

Unkeyed widgets behave exactly as they do today.

---

## Problem

`FocusManager` (`crates/telex/src/focus.rs`) rebuilds its focus list from the view tree
every frame and identifies each entry only by its **index** in that list. Everything
focus-related is keyed on that index:

1. **Focus requests can only say "index N".** `.focused(true)` exists only on
   `TextInput`, and `collect_focusables` honours it once per app lifetime
   (`initial_focus_applied`). An app that swaps whole screens (a multi-step
   wizard, say) can't focus a widget on any screen after the first, and has no way to
   focus a button at all.
2. **Scroll and cursor state leak between widgets.** `scroll_states` / `cursor_states`
   are `Vec`s indexed by focus position, grown but never reset. After a screen change, an
   offset left by one screen's scroll box applies to whatever sits at the same index on
   the next screen. The 0.3.4 clamp stops it overrunning the content, but the box can
   still open part-way down instead of at the top.
3. **Focus itself is an index.** After a screen change, or when a conditional widget
   appears above the focused one, focus lands on whatever now occupies that index.
4. **Commands beat the focused widget for every key.** In the run loop
   (`crates/telex/src/lib.rs`), `commands.execute` runs before any widget handling, and a
   match `continue`s. So a focused button never sees Enter if the app bound Enter, and a
   command bound to a plain letter fires while the user is typing in a `TextInput`.

`docs/architecture.md` already lists "Smarter focus tracking with stable IDs" as a future
consideration. This is that work.

## Goals

- A widget can be given a stable identity that survives frames, screen changes and
  re-ordering.
- Focus, scroll offset and cursor position follow that identity.
- Any focusable widget can request focus declaratively, and the request takes effect each
  time it newly appears (e.g. on entering a screen), not once per app.
- Focus moved by the app is visibly focused.
- A focused widget gets the keys it actually handles. Everything else still reaches the
  app's commands.
- Unkeyed widgets and existing apps keep working without changes.

## Non-goals

- **Imperative focus (`cx.focus("id")`).** The declarative request covers screen entry.
  It can be added later on top of the same ids if a real case needs it.
- **Custom widgets in focus.** `View::Custom` is not focusable today (`focus.rs`,
  `collect_recursive`), and stays that way here.
- **Mouse / click-to-focus.** Tracked separately in `docs/roadmap.md`.

---

## Design

### 1. Keys

Every focusable builder gains `.key(impl Into<String>)`: button, checkbox, text input,
text area, list, radio group, tabs, tree, table, slider, form field, menu bar, command
palette, terminal, and scrollable box.

Internally each collected focusable gets a `FocusId`:

```rust
enum FocusId {
    Key(String),     // from .key("...")
    Position(usize), // fallback: index in this frame's focus list (today's behaviour)
}
```

**Duplicate keys:** the first widget in tree order keeps the key. Later duplicates fall
back to `Position`, and `TELEX_DEBUG=1` reports the duplicate. A duplicate is a bug in the
app, but it shouldn't make two widgets share scroll state.

### 2. Focus follows the id

`FocusManager` stores the focused **id** alongside the index. After each
`collect_focusables`:

| Focused id this frame | Result |
|---|---|
| `Key` still present | focus moves to that widget's new index |
| `Key` gone | fall back to today's rule: same index, clamped to the list length |
| `Position` | today's rule |

Modal containment saves and restores a `FocusId` rather than an index, so closing a modal
returns focus to the same widget even if the tree shifted underneath it.

### 3. State lifetime

`scroll_states` and `cursor_states` become maps keyed by `FocusId`.

- **Keyed state lives as long as its widget.** An entry whose key is absent from a frame
  is dropped, so a keyed box that reappears later (re-entering a screen) starts at the top.
- **Positional state behaves as today.** It persists across frames by index. This is the
  case keys exist to fix.

`RenderContext` keeps its index-based vectors. `render.rs` walks the tree in the same order
`collect_focusables` does, so `FocusManager` builds per-index vectors from the maps before
each draw and folds the clamped results (and the 0.3.4 `ScrollLimit`s) back into the maps
after. Render code needs no changes beyond what 0.3.4 added.

### 4. Focus requests

`.focused(bool)` is available on every focusable builder, not just `TextInput`.

Each frame, `collect_focusables` records the set of ids requesting focus. **A request is
applied when its id was not requesting focus on the previous frame.** That covers:

- entering a screen whose button requests focus
- a widget appearing already requesting focus
- an app toggling `.focused(true)` on in response to state

A request that persists frame after frame is applied once, so Tab and Shift+Tab still move
focus away freely. If several ids newly request focus in the same frame, the first in tree
order wins.

`initial_focus_applied` goes away: the first frame is just the first time every request is
"new". Modal entry uses the same rule within the modal's focusables, falling back to the
first modal item as today.

Unkeyed widgets can request focus too, using their `Position` id. The known limit: two
consecutive screens requesting focus at the *same* position won't re-trigger. Giving the
widget a key removes that limit.

### 5. Visibility

Today focus styling stays hidden until the first Tab or Shift+Tab (`focus_visible`).

- **An applied focus request sets `focus_visible = true`.** The app moved focus on purpose,
  so it shows, the same as Tab.
- **Default focus (index 0 on first render, no request) stays hidden**, as today.

### 6. Key dispatch order

Replaces the current "commands first, then widgets" order in the run loop:

1. **Modal / command palette / menu bar handling.** Unchanged, as today.
2. **Ctrl or Alt chords go to commands first.** App shortcuts are never swallowed by the
   focused widget.
3. **The focused widget, only if it handles that key**, per the table below. If it does,
   dispatch stops.
4. **App-registered commands.**
5. **Built-in keys.** Ctrl+Q quit, Tab, Shift+Tab, as today.

What counts as "handles that key", taken from the current run loop and `FocusManager`
handlers:

| Focused widget | Keys it consumes |
|---|---|
| Button | Enter, Space, if `on_press` is set |
| Checkbox | Enter, Space, if `on_toggle` is set |
| TextInput | printable chars (incl. Space), Backspace, Left, Right. Up/Down only if `on_key_up`/`on_key_down` is set. Enter only if `on_submit` is set |
| TextArea | printable chars, Enter, Backspace, arrows |
| FormField | printable chars except Space, Backspace (Space goes to the Enter/Space activate arm, which doesn't type into form fields, a pre-existing quirk) |
| List | Up, Down |
| RadioGroup | Up, Down, `j`, `k` |
| Tabs | Left, Right, `[`, `]`, `1`–`9` |
| Tree | Up, Down, Left, Right, Home, End, `j`, `k`, `g`, `G`. Enter and Space only if `on_activate` is set |
| Table | Up, Down, `j`, `k`. Enter and Space only if `on_activate` is set |
| Slider | Left, Right |
| MenuBar | Left, Right, Enter, Space. Up/Down while a menu is open |
| Scrollable box | Up, Down, PageUp, PageDown, Home, End |
| Terminal | everything except Ctrl+Shift+`[` (escape) and chords claimed in step 2 |

Two consequences:

- **List and RadioGroup don't consume Enter.** Today `activate()` only handles buttons and
  checkboxes, so Enter on a focused list or radio falls through to the app's Enter command,
  same as now.
- **Typing is safe.** A command bound to a plain letter no longer fires while a text input,
  text area or form field has focus.

---

## Compatibility

New API is additive (`.key()`, `.focused()` on more builders). Behaviour changes, hence
`0.4.0`:

| Change | Who notices |
|---|---|
| Focused widget beats a plain-key command for keys it handles | apps that bind plain keys (letters, Enter, Space, arrows) as commands *and* have focusable widgets that handle them |
| `.focused(true)` on a `TextInput` re-applies whenever it newly appears, not once per app | apps that render a focused input on more than one screen |
| An applied `.focused(true)` request shows focus styling straight away, not after the first Tab | apps using `.focused(true)`, e.g. a text input focused on start |
| Positional scroll/cursor state is keyed by position only, no longer shared with a keyed widget at the same position | apps mixing keyed and unkeyed scroll boxes at the same position across screens |
| Keyed scroll/cursor state resets when its widget leaves the tree | apps that start using `.key()` (opt-in) |

These go in the changelog / release notes for 0.4.0.

## Testing

Written first, failing against the 0.3 code:

1. **Focus request on screen change:** two views swapped by state, each with a
   `.focused(true)` button. Focus lands on the second view's button after the swap.
2. **Scroll state doesn't leak:** view A has a scrolled keyed box at index 0, swap to view B
   with a different keyed box at index 0. B starts at the top. (Also a positional variant
   documenting today's behaviour.)
3. **Focus follows the key:** focus a keyed button, then render a new focusable above it.
   Focus stays on the same button.
4. **Persistent request doesn't trap focus:** a button with `.focused(true)` every frame,
   then Tab moves focus away and it stays away.
5. **Visibility:** an applied request makes focus visible. Default focus doesn't.
6. **Dispatch order:** a plain-letter command plus a focused text input. Typing the letter
   inserts it. With an unfocused/non-text widget, the command fires. Enter on a focused list
   reaches an Enter command. Enter on a focused button calls `on_press`, not the command.
7. **Duplicate keys:** two widgets with the same key keep separate scroll state.
8. **Modal restore by id:** open a modal, shift the tree, close it. Focus returns to the
   same keyed widget.

Dispatch, focus-request and modal tests drive the real run loop via `run_headless`
(`lib.rs`), so they exercise the actual key dispatch. Scroll-state tests use `TestApp`.
These tests live in `crates/telex/tests/focus_identity_tests.rs`.

## Adopting in an app

Not part of this release, listed to check the design covers a typical multi-screen app:

- Key scroll boxes that appear on more than one screen, so a box never opens scrolled from
  a previous screen.
- `.focused(true)` on each screen's primary button where wanted. With the dispatch order
  above, Enter then goes to that button. Apps that also bind Enter as a screen-wide command
  should check the button's `on_press` and the command agree.
- Moving from `telex-tui = "0.3"` to `"0.4"` is a deliberate opt-in.

## Open questions

- **Duplicate-key policy.** "First wins, rest positional, debug warning" is proposed above.
  The alternative is a panic in debug builds.
- **Ctrl/Alt rule and the terminal widget.** Step 2 means a command bound to e.g. Ctrl+C
  would beat a focused terminal. Today commands also run first, so this isn't a regression.
  But we could let a focused terminal claim all chords except Ctrl+Q and the escape chord.
