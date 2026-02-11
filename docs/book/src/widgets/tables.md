# Tables

Multi-column data displays with sortable headers, row selection, and flexible layouts.

```rust
View::table()
    .column("Name")
    .column("Status")
    .column("Value")
    .rows(data)
    .selected(selected.get())
    .on_select(with!(selected => move |idx| selected.set(idx)))
    .build()
```

Run with: `cargo run -p telex-tui --example 17_table`

## Basic table

A table displays rows of data in aligned columns:

```rust
let rows = vec![
    vec!["Alice".to_string(), "Active".to_string(), "100".to_string()],
    vec!["Bob".to_string(), "Inactive".to_string(), "50".to_string()],
    vec!["Carol".to_string(), "Active".to_string(), "75".to_string()],
];

View::table()
    .column("Name")
    .column("Status")
    .column("Score")
    .rows(rows)
    .build()
```

Each row is a `Vec<String>`. The number of items in each row should match the number of columns.

## Column configuration

Use `.column()` for simple headers or `.column_with()` for advanced configuration:

```rust
View::table()
    .column("Name")  // flexible width, left-aligned
    .column_with(
        TableColumn::new("Status")
            .width(ColumnWidth::Fixed(12))
    )
    .column_with(
        TableColumn::new("Score")
            .width(ColumnWidth::Fixed(8))
            .align(TextAlign::Right)
    )
    .rows(data)
    .build()
```

### Column width

**Fixed width:** Exact number of characters

```rust
.column_with(TableColumn::new("ID").width(ColumnWidth::Fixed(8)))
```

**Flex width:** Proportional share of remaining space

```rust
.column_with(TableColumn::new("Description").width(ColumnWidth::Flex(2)))
```

If you don't specify width, columns default to flexible with equal weight.

### Text alignment

```rust
.column_with(
    TableColumn::new("Count")
        .align(TextAlign::Right)  // right-align numbers
)
```

Options: `TextAlign::Left` (default), `TextAlign::Right`, `TextAlign::Center`.

## Row selection

Track which row is selected:

```rust
let selected = state!(cx, || 0usize);

View::table()
    .column("Name")
    .column("Value")
    .rows(data)
    .selected(selected.get())
    .on_select(with!(selected => move |idx: usize| selected.set(idx)))
    .build()
```

Users navigate with arrow keys. `on_select` fires when selection changes.

## Row activation

Handle Enter key on a selected row:

```rust
let on_activate = with!(selected, data => move |idx: usize| {
    if let Some(row) = data.get(idx) {
        // Do something with the selected row
        show_details(row);
    }
});

View::table()
    .rows(data.get())
    .selected(selected.get())
    .on_select(on_select)
    .on_activate(on_activate)
    .build()
```

Common use: open a detail view or edit modal for the selected item.

## Sorting

Tables support sortable columns. You manage the sort state and re-sort data yourself:

```rust
// Track sort: (column_index, ascending)
let sort_state = state!(cx, || None::<(usize, bool)>);

// Sort your data based on state
let mut rows = base_data.clone();
if let Some((col, ascending)) = sort_state.get() {
    rows.sort_by(|a, b| {
        let a_val = a.get(col).unwrap_or(&String::new());
        let b_val = b.get(col).unwrap_or(&String::new());
        if ascending {
            a_val.cmp(b_val)
        } else {
            b_val.cmp(a_val)
        }
    });
}

let on_sort = with!(sort_state => move |col: usize, asc: bool| {
    sort_state.set(Some((col, asc)));
});

View::table()
    .rows(rows)
    .sort(sort_state.get())
    .on_sort(on_sort)
    .build()
```

The table widget displays sort indicators (▲/▼) in headers. You handle the actual sorting logic.

## Numeric sorting

For numeric columns, parse before comparing:

```rust
rows.sort_by(|a, b| {
    let a_num = a[col].parse::<i32>().unwrap_or(0);
    let b_num = b[col].parse::<i32>().unwrap_or(0);
    if ascending {
        a_num.cmp(&b_num)
    } else {
        b_num.cmp(&a_num)
    }
});
```

## Custom data types

Convert your structs to rows:

```rust
#[derive(Clone)]
struct Pod {
    name: String,
    status: String,
    cpu: String,
    memory: String,
}

let pods = vec![
    Pod { name: "nginx".into(), status: "Running".into(), cpu: "12%".into(), memory: "256Mi".into() },
    Pod { name: "redis".into(), status: "Running".into(), cpu: "8%".into(), memory: "128Mi".into() },
];

// Convert to table rows
let rows: Vec<Vec<String>> = pods.iter()
    .map(|p| vec![p.name.clone(), p.status.clone(), p.cpu.clone(), p.memory.clone()])
    .collect();

View::table()
    .column("Name")
    .column("Status")
    .column("CPU")
    .column("Memory")
    .rows(rows)
    .build()
```

## Column layout example

Mix fixed and flexible columns:

```rust
View::table()
    .column_with(TableColumn::new("ID").width(ColumnWidth::Fixed(6)))
    .column("Description")  // flex (fills remaining space)
    .column_with(TableColumn::new("Count").width(ColumnWidth::Fixed(8)).align(TextAlign::Right))
    .column_with(TableColumn::new("Status").width(ColumnWidth::Fixed(10)))
    .rows(data)
    .build()
```

Result: ID takes 6 chars, Count takes 8, Status takes 10, Description fills the rest.

## Empty state

Show a message when there's no data:

```rust
if data.is_empty() {
    View::styled_text("No data available").dim().build()
} else {
    View::table()
        .column("Name")
        .column("Value")
        .rows(data)
        .build()
}
```

## Real-world example: Kubernetes pods

From example 17:

```rust
let pods = vec![
    vec!["nginx-pod".into(), "Running".into(), "12%".into(), "256Mi".into(), "2h".into()],
    vec!["redis-cache".into(), "Running".into(), "8%".into(), "128Mi".into(), "5d".into()],
    vec!["api-server".into(), "Running".into(), "45%".into(), "512Mi".into(), "1h".into()],
];

View::table()
    .column("NAME")
    .column_with(TableColumn::new("STATUS").width(ColumnWidth::Fixed(12)))
    .column_with(TableColumn::new("CPU").width(ColumnWidth::Fixed(8)).align(TextAlign::Right))
    .column_with(TableColumn::new("MEMORY").width(ColumnWidth::Fixed(10)).align(TextAlign::Right))
    .column_with(TableColumn::new("AGE").width(ColumnWidth::Fixed(8)).align(TextAlign::Right))
    .rows(pods)
    .selected(selected.get())
    .sort(sort_state.get())
    .on_select(on_select)
    .on_sort(on_sort)
    .on_activate(on_activate)
    .build()
```

This creates a k9s-style pod dashboard with sortable columns and navigation.

## Table vs List

**Use Table when:**
- Data has multiple attributes (columns)
- Alignment matters (numbers, dates)
- Sorting by different fields is needed
- Comparing values across rows

**Use List when:**
- Single attribute per item
- Items are simple text
- Order is fixed or chronological
- No column alignment needed

See [Lists](../building-uis/lists.md) for simple collections.

## Scrolling

Tables automatically scroll when content exceeds viewport height. The header remains visible while scrolling through rows.

## Performance

Tables render all visible rows. For thousands of rows, consider:
- Pagination (show 50-100 rows at a time)
- Virtual scrolling (advanced)
- Filtering to reduce row count

For typical use (hundreds of rows), performance is fine.

## Common patterns

**Status indicators with color:**
```rust
let rows: Vec<Vec<String>> = data.iter().map(|item| {
    let status = if item.is_active {
        "✓ Active".to_string()
    } else {
        "✗ Inactive".to_string()
    };
    vec![item.name.clone(), status, item.count.to_string()]
}).collect();
```

**Showing selected row details:**
```rust
let selected_data = data.get(selected.get()).cloned();

View::vstack()
    .child(table_view)
    .child(if let Some(item) = selected_data {
        View::text(format!("Details: {:?}", item))
    } else {
        View::text("Select a row")
    })
    .build()
```

**Multi-column sorting (secondary sort):**
```rust
rows.sort_by(|a, b| {
    // Primary sort
    let primary = if ascending {
        a[col].cmp(&b[col])
    } else {
        b[col].cmp(&a[col])
    };

    // Secondary sort (always by first column)
    if primary == std::cmp::Ordering::Equal {
        a[0].cmp(&b[0])
    } else {
        primary
    }
});
```

**Formatting numbers:**
```rust
let rows: Vec<Vec<String>> = data.iter().map(|item| {
    vec![
        item.name.clone(),
        format!("{:>8}", item.count),  // right-aligned with padding
        format!("{:.2}%", item.percent),  // two decimal places
    ]
}).collect();
```

## Tips

**Column count must match** - Every row should have the same number of items as you have columns. Extra items are ignored, missing items show as empty.

**Right-align numbers** - Use `TextAlign::Right` for numeric columns. It's easier to compare values.

**Fixed width for predictable columns** - Status, dates, IDs usually have known max width. Use `ColumnWidth::Fixed`.

**Flex for variable content** - Names, descriptions, messages should use flexible width to utilize available space.

**Sort state is optional** - If you don't provide `.sort()`, the table won't show sort indicators. Sorting is purely visual feedback.

**Selection is zero-indexed** - First row is index 0. Check bounds when accessing data based on selection.

**Headers are always visible** - When scrolling, the header row stays at the top.

**Unicode in cells works** - Emojis, box drawing, CJK characters all work. The table handles grapheme cluster widths correctly. Note: Complex grapheme clusters (emoji with modifiers, combining characters) may impact rendering performance in very large tables (1000+ rows).

Next: [Trees](./trees.md)
