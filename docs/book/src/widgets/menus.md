# Menus

Dropdown menu bars with keyboard shortcuts and command handling.

```rust
let file_menu = Menu::new("File")
    .command_with_shortcut("file.save", "Save", "Ctrl+S")
    .command_with_shortcut("file.quit", "Quit", "Ctrl+Q");

View::menu_bar()
    .menu(file_menu)
    .on_select(|cmd| {
        match cmd {
            "file.save" => save(),
            "file.quit" => quit(),
            _ => {}
        }
    })
    .build()
```

Run with: `cargo run -p telex-tui --example 20_menu_bar`

## Limitations

**Note:** The current menu API doesn't support visually disabling menu items. You can handle disabled commands in your `on_select` handler (see [Disabled items](#disabled-items) section below), but disabled items will still appear clickable in the menu.

## Basic menu bar

Create a menu bar with dropdown menus:

```rust
let file_menu = Menu::new("File")
    .command("file.new", "New")
    .command("file.open", "Open")
    .command("file.quit", "Quit");

View::menu_bar()
    .menu(file_menu)
    .on_select(with!(state => move |cmd: &str| {
        handle_command(cmd, &state);
    }))
    .build()
```

## Menu structure

Each menu has a label and a list of commands:

```rust
Menu::new("Edit")
    .command("edit.undo", "Undo")
    .command("edit.redo", "Redo")
    .separator()
    .command("edit.cut", "Cut")
    .command("edit.copy", "Copy")
    .command("edit.paste", "Paste")
```

`.separator()` adds a visual divider between groups of commands.

## Command IDs

Commands have unique IDs (strings) that identify them when selected:

```rust
.command("namespace.action", "Display Label")
```

Use namespaced IDs like `"file.save"`, `"edit.copy"` to avoid collisions.

## Keyboard shortcuts

Show shortcuts next to menu items:

```rust
Menu::new("File")
    .command_with_shortcut("file.new", "New", "Ctrl+N")
    .command_with_shortcut("file.save", "Save", "Ctrl+S")
    .command_with_shortcut("file.quit", "Quit", "Ctrl+Q")
```

The shortcut is displayed but not automatically bound - you handle the actual keybinding separately.

## Handling commands

Respond when users select menu items:

```rust
let on_select = with!(count => move |cmd: &str| {
    match cmd {
        "counter.increment" => count.update(|n| *n += 1),
        "counter.decrement" => count.update(|n| *n -= 1),
        "counter.reset" => count.set(0),
        _ => {}
    }
});

View::menu_bar()
    .menu(counter_menu)
    .on_select(on_select)
    .build()
```

## Multiple menus

Add several menus to the bar:

```rust
View::menu_bar()
    .menu(file_menu)
    .menu(edit_menu)
    .menu(view_menu)
    .menu(help_menu)
    .on_select(on_select)
    .build()
```

Users navigate left/right between menus with arrow keys.

## Keyboard navigation

Menu bars support full keyboard control:
- **Tab** - Focus the menu bar
- **←/→** - Switch between menus
- **Enter** - Open the focused menu
- **↑/↓** - Navigate items within an open menu
- **Enter** - Execute the selected item
- **Escape** - Close the menu

## Menu state tracking

Track which menu and item are active:

```rust
let active_menu = state!(cx, || None::<usize>);
let highlighted_item = state!(cx, || 0usize);

View::menu_bar()
    .menu(file_menu)
    .menu(edit_menu)
    .active_menu(active_menu.get())
    .selected_item(highlighted_item.get())
    .on_menu_change(with!(active_menu => move |idx| {
        active_menu.set(idx);
    }))
    .on_item_change(with!(highlighted_item => move |idx| {
        highlighted_item.set(idx);
    }))
    .on_select(on_select)
    .build()
```

This lets you track and control menu state programmatically.

## Implementing actual shortcuts

The menu bar shows shortcuts but doesn't bind them. Use `use_command` to implement the actual keys:

```rust
cx.use_command(
    KeyBinding::key(KeyCode::Char('s')).ctrl(true),
    with!(data => move || save_file(&data))
);

cx.use_command(
    KeyBinding::key(KeyCode::Char('q')).ctrl(true),
    move || std::process::exit(0)
);
```

## Disabled items

You can't disable items directly in the current API, but you can handle disabled state in your command handler:

```rust
let on_select = with!(can_undo => move |cmd: &str| {
    match cmd {
        "edit.undo" if !can_undo.get() => {
            // Show "Nothing to undo" message
        }
        "edit.undo" => perform_undo(),
        _ => {}
    }
});
```

## Real-world example

Application menu from Example 20:

```rust
let file_menu = Menu::new("File")
    .command_with_shortcut("file.new", "New", "Ctrl+N")
    .command_with_shortcut("file.open", "Open", "Ctrl+O")
    .command_with_shortcut("file.save", "Save", "Ctrl+S")
    .separator()
    .command_with_shortcut("file.quit", "Quit", "Ctrl+Q");

let edit_menu = Menu::new("Edit")
    .command_with_shortcut("edit.undo", "Undo", "Ctrl+Z")
    .command_with_shortcut("edit.redo", "Redo", "Ctrl+Y")
    .separator()
    .command_with_shortcut("edit.cut", "Cut", "Ctrl+X")
    .command_with_shortcut("edit.copy", "Copy", "Ctrl+C")
    .command_with_shortcut("edit.paste", "Paste", "Ctrl+V");

let on_select = move |cmd: &str| {
    match cmd {
        "file.new" => create_new_file(),
        "file.open" => open_file_dialog(),
        "file.save" => save_current_file(),
        "file.quit" => std::process::exit(0),
        "edit.undo" => undo_last_action(),
        "edit.redo" => redo_last_action(),
        "edit.cut" => cut_selection(),
        "edit.copy" => copy_selection(),
        "edit.paste" => paste_from_clipboard(),
        _ => {}
    }
};

View::menu_bar()
    .menu(file_menu)
    .menu(edit_menu)
    .on_select(on_select)
    .build()
```

## Layout

The menu bar appears as the first row of your app. Put it at the top of a `vstack`:

```rust
View::vstack()
    .child(View::menu_bar().menu(...).build())
    .child(View::boxed().flex(1).child(main_content).build())
    .child(status_bar)
    .build()
```

## Tips

**Use namespaced IDs** - Prefix commands with their menu: `"file.save"`, `"edit.undo"`.

**Separate common groups** - Use `.separator()` to group related commands (New/Open/Save, then Quit).

**Show shortcuts** - Even if users don't use them, shortcuts communicate what's possible.

**Implement the shortcuts** - Don't just display them - actually bind the keys with `use_command`.

**Keep menus short** - 3-7 items per menu is ideal. More than 10 gets unwieldy.

**Standard order** - File, Edit, View, Help is convention. Users expect this.

**Command naming** - Use verbs: "Save", "Copy", "Quit", not nouns.

Next: [Toasts](./toasts.md)
