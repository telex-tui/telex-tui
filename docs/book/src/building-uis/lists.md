# Lists and Selection

Lists are **controlled components** - you own the data and selection state, the widget handles display and keyboard navigation.

Two separate concerns:
- **Navigation** - Arrow keys move selection, fires `on_select` with the new index
- **Activation** - Enter key acts on the selected item (you handle this separately with `use_command`)

You provide the items and current selection. When the user navigates, you update your state. The widget never mutates your data directly.

---

```rust
let items = vec!["One".to_string(), "Two".to_string(), "Three".to_string()];
let selected = state!(cx, || 0usize);

View::list()
    .items(items)
    .selected(selected.get())
    .on_select(with!(selected => move |idx: usize| selected.set(idx)))
    .build()
```

Run with: `cargo run -p telex-tui --example 05_todo_list`

## Basic list

The simplest list just displays items:

```rust
let items = vec![
    "Learn Telex".to_string(),
    "Build something cool".to_string(),
];

View::list()
    .items(items)
    .build()
```

The list widget automatically handles:
- Scrolling when content exceeds viewport
- Arrow key navigation (↑/↓)
- Visual highlighting of the selected item

## Managing selection

Track which item is selected with state:

```rust
let items = state!(cx, || vec!["Apple".to_string(), "Banana".to_string()]);
let selected = state!(cx, || 0usize);

View::list()
    .items(items.get())
    .selected(selected.get())
    .on_select(with!(selected => move |idx: usize| selected.set(idx)))
    .build()
```

`on_select` fires when the user navigates with arrow keys. The callback receives the new index.

## Responding to activation

Use the `use_command` hook to bind keyboard shortcuts to actions. This lets you handle the Enter key on the selected item:

```rust
let selected_idx = selected.get();

cx.use_command(
    KeyBinding::key(KeyCode::Enter),
    with!(items, selected => move || {
        let idx = selected.get();
        let items_vec = items.get();
        if idx < items_vec.len() {
            // Do something with items_vec[idx]
            println!("Activated: {}", items_vec[idx]);
        }
    })
);
```

This pattern separates navigation (on_select) from activation (Enter key).

## Adding and removing items

Lists work with `Vec<T>` state. Use `.update()` to modify:

```rust
let items = state!(cx, || Vec::new());

// Add an item
items.update(|v| v.push("New item".to_string()));

// Remove at index
let idx = selected.get();
items.update(|v| {
    if idx < v.len() {
        v.remove(idx);
    }
});
```

When removing items, adjust the selection to stay in bounds:

```rust
items.update(|v| {
    if idx < v.len() {
        v.remove(idx);
        // Move selection up if we removed the last item
        if idx > 0 && idx >= v.len() {
            selected.set(idx - 1);
        }
    }
});
```

## Empty state

Show alternate content when the list is empty:

```rust
let items = items.get();

if items.is_empty() {
    View::styled_text("No items yet").dim().build()
} else {
    View::list()
        .items(items)
        .selected(selected.get())
        .on_select(on_select)
        .build()
}
```

This gives users context rather than showing an empty list.

## Working with structured data

Lists work with any type that implements `Display`:

```rust
struct Task {
    name: String,
    done: bool,
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let check = if self.done { "✓" } else { " " };
        write!(f, "[{}] {}", check, self.name)
    }
}

let tasks = state!(cx, || vec![
    Task { name: "First".into(), done: false },
    Task { name: "Second".into(), done: true },
]);

View::list()
    .items(tasks.get())
    .selected(selected.get())
    .on_select(on_select)
    .build()
```

## List vs Table vs Tree

**Use List when:**
- Displaying a simple collection
- Items fit on one line
- Order matters (chronological, priority)

**Use Table when:**
- Data has multiple columns
- You need sortable headers
- Alignment matters (numbers, dates)

**Use Tree when:**
- Data is hierarchical
- Users need to expand/collapse sections
- Showing file systems or nested structures

## Real-world example

The file browser (example 07) demonstrates:
- Directory navigation with Enter
- Parent directory (`..`) handling
- Distinguishing files from folders (trailing `/`)
- Showing file details in a modal

```rust
// In example 07_file_browser.rs
cx.use_command(
    KeyBinding::key(KeyCode::Enter),
    with!(current_path, selected => move || {
        let entry = &entries[selected.get()];
        if entry == ".." {
            // Go to parent directory
        } else if entry.ends_with('/') {
            // Enter directory
        } else {
            // Show file info
        }
    })
);
```

## Tips

**Selection bounds:** Always check `idx < items.len()` before accessing. Lists can be empty or shrink.

**Preserve selection:** When filtering or sorting, decide if selection should track the same item or stay at the same index.

**Initial selection:** Start at 0, or use `.selected(None)` if no default selection makes sense.

**Performance:** Lists render all items. For very large collections (10k+ items), consider virtualization or pagination.

Next: [Text Input](./text-input.md)
