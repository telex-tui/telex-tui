# Modals

Modals are **declarative** - they're always in your view tree, controlled by a `visible` flag. No imperative "show modal" calls.

When visible:
- The modal renders as a centered overlay
- **Focus is trapped** - Tab stays within the modal, can't reach elements behind it
- **Escape dismisses** - fires `on_dismiss`, you update your visibility state

When the modal closes, focus returns to where it was before.

---

```rust
let show_modal = state!(cx, || false);

View::modal()
    .visible(show_modal.get())
    .title("Confirm")
    .on_dismiss(with!(show_modal => move || show_modal.set(false)))
    .child(View::text("Are you sure?"))
    .build()
```

Run with: `cargo run -p telex-tui --example 23_modal`

## Basic modal

A modal appears as a centered overlay with a border and optional title:

```rust
let show_help = state!(cx, || false);

View::modal()
    .visible(show_help.get())
    .title("Help")
    .on_dismiss(with!(show_help => move || show_help.set(false)))
    .child(
        View::vstack()
            .child(View::text("Press F1 to toggle this help"))
            .child(View::text("Press Escape to close"))
            .build()
    )
    .build()
```

Modals are always present in the view tree but only rendered when `.visible(true)`.

## Opening and closing

Control visibility with state:

```rust
// Open
show_modal.set(true);

// Close
show_modal.set(false);

// Toggle
show_modal.update(|v| *v = !*v);
```

The `on_dismiss` callback fires when the user presses Escape:

```rust
.on_dismiss(with!(show_modal => move || show_modal.set(false)))
```

Most modals should close on Escape - it's the expected behavior.

## Sizing

Set width and height in columns/rows:

```rust
View::modal()
    .visible(visible.get())
    .title("Custom Size")
    .width(60)   // 60 columns wide
    .height(30)  // 30 rows tall
    .child(content)
    .build()
```

If you don't specify size, the modal sizes to its content. For large content, set explicit dimensions and let the content scroll.

## Confirm dialogs

Ask for confirmation before destructive actions:

```rust
let show_confirm = state!(cx, || false);

let on_yes = with!(show_confirm, items => move || {
    items.update(|v| v.clear());  // destructive action
    show_confirm.set(false);
});

let on_no = with!(show_confirm => move || {
    show_confirm.set(false);
});

View::modal()
    .visible(show_confirm.get())
    .title("Confirm Delete")
    .on_dismiss(on_no.clone())  // Escape = No
    .child(
        View::vstack()
            .child(View::text("Delete all items?"))
            .child(View::text("This cannot be undone."))
            .child(View::gap(1))
            .child(
                View::hstack()
                    .spacing(2)
                    .child(View::button().label("Yes").on_press(on_yes).build())
                    .child(View::button().label("No").on_press(on_no).build())
                    .build()
            )
            .build()
    )
    .build()
```

Common pattern: Escape key dismisses = same as clicking "No" or "Cancel".

## Alert dialogs

Show information that requires acknowledgment:

```rust
let show_alert = state!(cx, || false);

let on_ok = with!(show_alert => move || show_alert.set(false));

View::modal()
    .visible(show_alert.get())
    .title("Success")
    .on_dismiss(on_ok.clone())
    .child(
        View::vstack()
            .child(View::styled_text("Operation completed!").color(Color::Green).build())
            .child(View::gap(1))
            .child(View::button().label("OK").on_press(on_ok).build())
            .build()
    )
    .build()
```

Alerts typically have a single "OK" button. Escape and OK do the same thing.

## Modals with forms

Combine modals with text inputs for data entry:

```rust
let show_form = state!(cx, || false);
let name = state!(cx, String::new);

let on_save = with!(show_form, name => move || {
    let value = name.get();
    if !value.is_empty() {
        save_to_database(value);
        name.set(String::new());  // clear for next time
        show_form.set(false);
    }
});

let on_cancel = with!(show_form, name => move || {
    name.set(String::new());  // discard changes
    show_form.set(false);
});

View::modal()
    .visible(show_form.get())
    .title("Enter Name")
    .on_dismiss(on_cancel.clone())
    .child(
        View::vstack()
            .child(View::text("Name:"))
            .child(
                View::text_input()
                    .value(name.get())
                    .on_change(with!(name => move |s: String| name.set(s)))
                    .build()
            )
            .child(View::gap(1))
            .child(
                View::hstack()
                    .spacing(2)
                    .child(View::button().label("Save").on_press(on_save).build())
                    .child(View::button().label("Cancel").on_press(on_cancel).build())
                    .build()
            )
            .build()
    )
    .build()
```

When the user cancels, clear any partial input so the form is fresh next time.

## Focus containment

When a modal is visible, Tab navigation stays within the modal. You can't Tab to elements behind it.

Focus automatically returns to the main content when the modal closes.

This "modal focus trap" is automatic - you don't need to implement it. Telex's focus management system treats visible modals as focus boundaries, ensuring keyboard navigation stays within the topmost modal until it's dismissed.

## Multiple modals

You can have multiple modals in the view tree:

```rust
View::vstack()
    .child(main_content)
    .child(help_modal)
    .child(confirm_modal)
    .child(alert_modal)
    .build()
```

Only visible modals render. If multiple modals are visible simultaneously, the last one in the tree appears on top.

Common pattern: help modal always comes first, so action modals appear above it.

## Help screens with F1

Bind F1 to toggle a help modal:

```rust
let show_help = state!(cx, || false);

cx.use_command(
    KeyBinding::key(KeyCode::F(1)),
    with!(show_help => move || show_help.update(|v| *v = !*v))
);

View::modal()
    .visible(show_help.get())
    .title("Help")
    .on_dismiss(with!(show_help => move || show_help.set(false)))
    .child(help_content)
    .build()
```

F1 is the standard help key in Telex examples. Users expect it.

## Modal state management

**Clear form state when closing:**
```rust
let on_dismiss = with!(show_modal, form_data => move || {
    form_data.set(String::new());  // reset
    show_modal.set(false);
});
```

**Preserve state across opens:**
```rust
// Just close, don't clear
let on_dismiss = with!(show_modal => move || show_modal.set(false));
```

Choose based on use case. Forms usually clear on cancel, but editing modals might preserve drafts.

## Tips

**Title clarity** - The title should tell users what the modal is for. "Confirm Delete" not just "Confirm".

**Escape always works** - Never disable `on_dismiss`. Users expect Escape to close modals.

**Single action = Alert** - If there's only one button ("OK", "Got it"), it's an alert, not a dialog.

**Destructive actions = Confirm** - If the action can't be undone, ask for confirmation.

**Initial focus** - If the modal has a text input, use `.focused(true)` so the user can start typing immediately.

**Don't nest modals** - Showing a modal from within a modal is confusing. Close the first before opening the second.

**Size for content** - If your modal content is dynamic (lists, text areas), set explicit `.width()` and `.height()` to prevent the modal from jumping around.

**Overlay everything** - Modals appear above all other content. If something renders on top of your modal, check the order in your view tree.

## Common patterns

**Confirmation before quit:**
```rust
let show_quit_confirm = state!(cx, || false);

// On Ctrl+Q, show confirm instead of quitting
cx.use_command(
    KeyBinding::key(KeyCode::Char('q')).ctrl(true),
    with!(show_quit_confirm => move || show_quit_confirm.set(true))
);
```

**Loading modal:**
```rust
View::modal()
    .visible(is_loading.get())
    .title("Please wait...")
    .child(View::text("Loading data..."))
    .build()
```

Don't provide `on_dismiss` for loading modals - user can't cancel.

**Error modal:**
```rust
View::modal()
    .visible(error.get().is_some())
    .title("Error")
    .on_dismiss(with!(error => move || error.set(None)))
    .child(View::text(error.get().unwrap_or_default()))
    .build()
```

Next: [Streams](../dynamic-data/streams.md)
