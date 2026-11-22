# Focus and Keys

Tab and Shift+Tab move focus through every focusable widget in tree order. Three tools
control focus beyond that: `.key()` gives a widget a stable identity, `.focused(true)`
requests focus, and the key dispatch order decides whether the focused widget or an app
command gets a key.

```rust
View::vstack()
    .child(
        View::boxed()
            .key("log")                // own scroll state, follows this box
            .scroll(true)
            .child(View::text(log))
            .build(),
    )
    .child(
        View::button()
            .key("confirm")
            .label("Confirm")
            .focused(true)             // focused on entering this screen
            .on_press(confirm)
            .build(),
    )
    .build()
```

## Stable identity with `.key()`

Every focusable builder accepts `.key("...")`. Without a key, a widget is identified by
its position in the focus order. That's fine for a fixed layout, but in a multi-screen app
or one with conditional widgets, positions shift.

With a key:

- **Focus follows the widget.** If something appears above the focused widget, focus stays
  on it rather than on whatever now sits at its old position.
- **Scroll offset and cursor position follow the widget**, and are dropped when the widget
  leaves the tree. A keyed scroll box on a new screen always starts at the top.
- **Closing a modal returns focus to the same widget**, even if the tree changed while it
  was open.

Keys only need to be unique among the widgets on screen at the same time. If two share a
key, the first in tree order keeps it and the others fall back to positional identity.
`TELEX_DEBUG=1` shows duplicates, and `TestApp::duplicate_focus_keys()` returns them.

## Requesting focus with `.focused(true)`

Any focusable widget accepts `.focused(true)`. A request is applied when it *newly*
appears:

- on the first render
- when a screen containing it is entered
- when the widget appears, or starts requesting, in response to state

A request that stays in place frame after frame doesn't trap focus. Tab still moves away.
If several widgets newly request focus at once, the first in tree order wins. Inside a
modal, a request picks the modal's initial focus.

A focus the app requests is shown straight away. Default focus stays unstyled until the
first Tab.

Give requesting widgets a `.key()` in multi-screen apps. Without one, two consecutive
screens requesting focus at the same position look like the same request, so the second
isn't re-applied.

## Who gets a key

1. **Ctrl and Alt chords** go to commands registered with `cx.use_command` first.
2. **Other keys** go to the focused widget first, *if it handles that key*. A text input
   takes printable characters, Backspace and Left/Right, and Enter only if it has
   `on_submit`. A button takes Enter and Space if it has `on_press`. A list takes Up/Down.
3. Otherwise the key goes to **app commands**.
4. Otherwise to Telex's built-in keys (Tab, Shift+Tab, Ctrl+Q).

So a command bound to a plain letter no longer fires while the user types in a text input,
and a screen-wide Enter command still runs when the focused widget (a list, a scroll box, a
radio group) has no use for Enter.

The full table of which widget handles which keys is in `docs/focus-identity-design.md`.
