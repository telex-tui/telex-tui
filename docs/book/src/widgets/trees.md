# Trees

Hierarchical data structures with expand/collapse functionality for navigating nested content.

```rust
let items = vec![
    TreeItem::new("src")
        .icon("📁")
        .expanded(true)
        .child(TreeItem::new("main.rs").icon("📄"))
        .child(TreeItem::new("lib.rs").icon("📄")),
    TreeItem::new("README.md").icon("📝"),
];

View::tree()
    .items(items)
    .selected(selected.get())
    .on_select(with!(selected => move |path| selected.set(path)))
    .build()
```

Run with: `cargo run -p telex-tui --example 16_tree`

## Basic tree

A tree displays hierarchical data with parent-child relationships:

```rust
let items = vec![
    TreeItem::new("Parent")
        .child(TreeItem::new("Child 1"))
        .child(TreeItem::new("Child 2")),
    TreeItem::new("Another Parent"),
];

View::tree()
    .items(items)
    .build()
```

Each `TreeItem` can have children, creating nested levels.

## TreeItem builder

Build tree items with the fluent API:

```rust
TreeItem::new("label")          // required: display text
    .icon("📁")                 // optional: icon before label
    .expanded(true)             // optional: show children (default false)
    .child(TreeItem::new("..")) // optional: add child nodes
```

## Icons

Add visual indicators for different item types:

```rust
TreeItem::new("src").icon("📁")           // folder
TreeItem::new("main.rs").icon("📄")       // file
TreeItem::new("Cargo.toml").icon("📦")    // package
TreeItem::new("README.md").icon("📝")     // document
```

Icons appear before the label. Use any string - emojis, Unicode symbols, or ASCII.

## Expand/collapse

Control which nodes show their children:

```rust
TreeItem::new("Folder")
    .expanded(true)   // children visible
    .child(TreeItem::new("File 1"))
    .child(TreeItem::new("File 2"))
```

When `.expanded(false)` or omitted, children are hidden. The tree shows a collapse indicator (▶/▼).

## Selection tracking

Trees use **paths** to identify items. A path is `Vec<usize>` - indices from root to the item:

```rust
let selected = state!(cx, || vec![0]);  // first root item

View::tree()
    .items(items)
    .selected(selected.get())
    .on_select(with!(selected => move |path: TreePath| {
        selected.set(path);
    }))
    .build()
```

Path examples:
- `[0]` - first root item
- `[0, 2]` - third child of first root item
- `[1, 0, 1]` - second grandchild of second root item

## Expand/collapse with activation

Handle Enter key to toggle expansion:

```rust
let expanded_paths = state!(cx, || vec![vec![0]]);  // src/ expanded

let on_activate = with!(expanded_paths => move |path: TreePath| {
    let mut paths = expanded_paths.get();

    if let Some(pos) = paths.iter().position(|p| *p == path) {
        // Already expanded, collapse it
        paths.remove(pos);
    } else {
        // Collapsed, expand it
        paths.push(path);
    }

    expanded_paths.set(paths);
});

View::tree()
    .items(build_tree_with_state(&expanded_paths.get()))
    .on_activate(on_activate)
    .build()
```

You manage which paths are expanded. Rebuild tree items based on that state.

## Dynamic tree building

Rebuild the tree based on expanded state:

```rust
fn build_tree(expanded_paths: &[TreePath]) -> Vec<TreeItem> {
    let is_expanded = |path: &[usize]| {
        expanded_paths.iter().any(|p| p == path)
    };

    vec![
        TreeItem::new("src")
            .icon("📁")
            .expanded(is_expanded(&[0]))
            .child(TreeItem::new("main.rs").icon("📄"))
            .child(TreeItem::new("lib.rs").icon("📄")),
        TreeItem::new("tests")
            .icon("📁")
            .expanded(is_expanded(&[1]))
            .child(TreeItem::new("test.rs").icon("📄")),
    ]
}

// In render:
let items = build_tree(&expanded_paths.get());
View::tree().items(items).build()
```

This pattern lets you control expansion state reactively.

## File system trees

Example from Example 16 - a project file browser:

```rust
vec![
    TreeItem::new("src")
        .icon("📁")
        .expanded(true)
        .child(
            TreeItem::new("components")
                .icon("📁")
                .child(TreeItem::new("button.rs").icon("📄"))
                .child(TreeItem::new("input.rs").icon("📄"))
        )
        .child(TreeItem::new("main.rs").icon("📄"))
        .child(TreeItem::new("lib.rs").icon("📄")),
    TreeItem::new("tests")
        .icon("📁")
        .child(TreeItem::new("integration.rs").icon("📄")),
    TreeItem::new("Cargo.toml").icon("📦"),
    TreeItem::new("README.md").icon("📝"),
]
```

## Getting item data by path

Navigate the tree to find the selected item:

```rust
fn get_item_at_path<'a>(
    items: &'a [TreeItem],
    path: &[usize]
) -> Option<&'a TreeItem> {
    if path.is_empty() {
        return None;
    }

    let mut current_items = items;
    let mut result = None;

    for &idx in path {
        if idx < current_items.len() {
            result = Some(&current_items[idx]);
            current_items = &current_items[idx].children;
        } else {
            return None;
        }
    }

    result
}

// Usage:
let selected_label = get_item_at_path(&items, &selected.get())
    .map(|item| item.label.clone())
    .unwrap_or_else(|| "Nothing".to_string());
```

This walks the path to retrieve the item.

## Nesting levels

Trees can be arbitrarily deep:

```rust
TreeItem::new("Level 1")
    .child(
        TreeItem::new("Level 2")
            .child(
                TreeItem::new("Level 3")
                    .child(TreeItem::new("Level 4"))
            )
    )
```

The tree widget handles indentation automatically - each level is indented further.

## Empty trees

Show a message when there's no data:

```rust
if items.is_empty() {
    View::styled_text("No items").dim().build()
} else {
    View::tree().items(items).build()
}
```

## Pattern: Conditional Children

For large trees, you can optimize performance by only building children when their parent is expanded. This isn't a built-in API - you implement it by conditionally adding children in your tree-building function:

```rust
fn build_tree(expanded_paths: &[TreePath]) -> Vec<TreeItem> {
    let is_expanded = |path: &[usize]| {
        expanded_paths.iter().any(|p| p == path)
    };

    vec![
        TreeItem::new("Folder")
            .expanded(is_expanded(&[0]))
            .children_if(is_expanded(&[0]), || {
                // Only build children if this folder is expanded
                vec![
                    TreeItem::new("File 1"),
                    TreeItem::new("File 2"),
                ]
            })
    ]
}
```

Replace `.children_if()` with manual conditionals:

```rust
let mut item = TreeItem::new("Folder").expanded(is_expanded(&[0]));

if is_expanded(&[0]) {
    item = item
        .child(TreeItem::new("File 1"))
        .child(TreeItem::new("File 2"));
}
```

This pattern avoids building large subtrees that aren't visible.

## Tree vs List vs Table

**Use Tree when:**
- Data is hierarchical (files, org charts, nested categories)
- Users need to expand/collapse sections
- Depth varies per branch
- Navigation follows parent-child relationships

**Use List when:**
- Flat data with no hierarchy
- Simple ordered collection
- No expand/collapse needed

**Use Table when:**
- Flat data with multiple columns
- Comparison across attributes
- Sorting is important

See [Lists](../building-uis/lists.md) and [Tables](./tables.md).

## Keyboard navigation

Tree widgets support:
- **↑/↓** - Navigate items (visible items only)
- **Enter** - Activate (typically expand/collapse)
- **Tab** - Focus next widget

Collapsed items are skipped during navigation - you only navigate visible items.

## Real-world patterns

**Project explorer:**
```rust
TreeItem::new("my-project")
    .icon("📦")
    .expanded(true)
    .child(TreeItem::new("src").icon("📁").child(...))
    .child(TreeItem::new("tests").icon("📁").child(...))
    .child(TreeItem::new("Cargo.toml").icon("📄"))
```

**Category navigation:**
```rust
TreeItem::new("Products")
    .child(
        TreeItem::new("Electronics")
            .child(TreeItem::new("Phones"))
            .child(TreeItem::new("Laptops"))
    )
    .child(
        TreeItem::new("Clothing")
            .child(TreeItem::new("Shirts"))
            .child(TreeItem::new("Pants"))
    )
```

**Organization chart:**
```rust
TreeItem::new("CEO")
    .child(
        TreeItem::new("VP Engineering")
            .child(TreeItem::new("Team Lead - Backend"))
            .child(TreeItem::new("Team Lead - Frontend"))
    )
    .child(TreeItem::new("VP Sales"))
```

## Tips

**Path is zero-indexed** - First root item is `[0]`, first child of first item is `[0, 0]`.

**Store expanded paths** - Keep a list of expanded paths in state. Check against this list when building the tree.

**Icons are optional** - Omit `.icon()` for plain text trees. Icons make it easier to distinguish file types.

**Children are Vec** - `TreeItem` has a public `.children` field. You can access it directly if needed.

**Rebuild on expansion** - When the user expands/collapses, rebuild the entire tree with updated `.expanded()` values.

**Selection persists** - Selected path stays valid even when you collapse ancestors. The widget handles this gracefully.

**Empty children** - A TreeItem with no children shows no expand/collapse indicator.

**Custom icons** - Use any Unicode: `"►"`, `"●"`, `"[D]"`, or emojis work fine.

Next: [Tabs](./tabs.md)
