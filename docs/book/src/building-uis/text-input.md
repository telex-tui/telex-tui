# Text Input

Text inputs are **controlled** - you own the value, the widget owns the cursor and keyboard handling.

You provide the current text via `.value()`. Every keystroke fires `on_change` with the new text. You decide whether to accept it by updating your state. This gives you full control over validation, transformation, and character limits.

---

## Single-line input

Use `TextInput` for short text like names, search queries, or form fields:

```rust
let name = state!(cx, || String::new());

View::text_input()
    .value(name.get())
    .placeholder("Enter name...")
    .on_change(with!(name => move |s: String| name.set(s)))
    .build()
```

Run with: `cargo run -p telex-tui --example 05_todo_list`

The text input widget provides:
- Keyboard editing (type, backspace, delete)
- Left/right arrow navigation
- Cursor positioning
- Placeholder text when empty

## The controlled pattern

Text inputs are **controlled** - you provide the value and handle changes:

```rust
let input = state!(cx, || String::new());

View::text_input()
    .value(input.get())                    // current value
    .on_change(with!(input => move |text: String| {
        input.set(text);                   // update on change
    }))
    .build()
```

Every keystroke triggers `on_change` with the new text. You decide whether to accept it by updating state.

This pattern enables:
- Validation (reject invalid input)
- Transformation (uppercase, trimming)
- Character limits
- Real-time feedback

## Handling submit

Use `on_submit` to respond when the user presses Enter:

```rust
View::text_input()
    .value(input.get())
    .placeholder("Type something...")
    .on_change(with!(input => move |text: String| input.set(text)))
    .on_submit(with!(input, items => move || {
        let text = input.get();
        if !text.is_empty() {
            items.update(|v| v.push(text));
            input.set(String::new());  // clear after submit
        }
    }))
    .build()
```

Common pattern: validate on submit, clear the input if successful.

## Arrow key callbacks

TextInput supports `on_key_up` and `on_key_down` for command history or autocomplete:

```rust
let history = state!(cx, || vec!["ls", "cd ..", "git status"]);
let history_idx = state!(cx, || None::<usize>);

View::text_input()
    .value(input.get())
    .on_change(on_change)
    .on_key_up(with!(history, history_idx, input => move || {
        let h = history.get();
        if h.is_empty() { return; }
        let idx = history_idx.get().map(|i| i.saturating_sub(1)).unwrap_or(h.len() - 1);
        history_idx.set(Some(idx));
        input.set(h[idx].to_string());
    }))
    .on_key_down(with!(history, history_idx, input => move || {
        let h = history.get();
        if let Some(idx) = history_idx.get() {
            if idx + 1 < h.len() {
                history_idx.set(Some(idx + 1));
                input.set(h[idx + 1].to_string());
            }
        }
    }))
    .build()
```

These fire when the input is focused and up/down arrows are pressed.

## Validation example

Only allow numeric input:

```rust
let number = state!(cx, || String::new());

View::text_input()
    .value(number.get())
    .placeholder("Numbers only...")
    .on_change(with!(number => move |text: String| {
        // Only update if all chars are digits
        if text.chars().all(|c| c.is_ascii_digit()) {
            number.set(text);
        }
    }))
    .build()
```

Since you control what goes into state, invalid input is simply ignored.

## Character limits

Enforce a maximum length:

```rust
View::text_input()
    .value(username.get())
    .on_change(with!(username => move |text: String| {
        if text.chars().count() <= 20 {
            username.set(text);
        }
    }))
    .build()
```

**Important:** Use `.chars().count()` not `.len()` for character limits. `.len()` counts bytes, not user-perceived characters. This matters for Unicode: `"café"` has 4 characters but 5 bytes, and emoji like "👍" are multiple bytes. Using `.len()` would incorrectly limit Unicode input.

## Multi-line input

Use `TextArea` for longer content like notes, messages, or code:

```rust
let content = state!(cx, || String::new());

View::text_area()
    .value(content.get())
    .placeholder("Start typing...")
    .rows(10)  // height in lines
    .on_change(with!(content => move |text: String| content.set(text)))
    .build()
```

Run with: `cargo run -p telex-tui --example 12_text_area`

TextArea supports:
- Multi-line editing with Enter
- Up/down/left/right navigation
- Scrolling when content exceeds height
- Line wrapping

## Tracking cursor position

TextArea can report cursor location:

```rust
let cursor_line = state!(cx, || 0usize);
let cursor_col = state!(cx, || 0usize);

View::text_area()
    .value(content.get())
    .rows(12)
    .on_change(with!(content => move |text: String| content.set(text)))
    .on_cursor_change(with!(cursor_line, cursor_col => move |line: usize, col: usize| {
        cursor_line.set(line);
        cursor_col.set(col);
    }))
    .build()
```

Use this to show cursor position, implement syntax highlighting, or track editing location.

## Real-time stats

Calculate line/word/char counts as the user types:

```rust
let text = content.get();
let line_count = if text.is_empty() { 0 } else { text.lines().count() };
let word_count = text.split_whitespace().count();
let char_count = text.chars().count();

View::text(format!("Lines: {} | Words: {} | Chars: {}",
    line_count, word_count, char_count))
```

This demonstrates the reactive model - state changes automatically trigger re-render with updated stats.

## Initial focus

Make an input focused when the component appears:

```rust
View::text_input()
    .value(input.get())
    .focused(true)  // focus on mount
    .on_change(on_change)
    .build()
```

Useful for dialogs, forms, or any UI where the user's next action is typing.

## TextInput vs TextArea

**Use TextInput when:**
- Single line of text
- Short responses (names, URLs, search)
- Submit on Enter makes sense
- Forms with multiple fields

**Use TextArea when:**
- Multiple lines expected
- Long-form content (notes, descriptions, messages)
- Editing structured text (JSON, code, logs)
- Height can be predetermined

## Placeholder text

Show hints when the input is empty:

```rust
View::text_input()
    .placeholder("Search files...")
    .value(search.get())
    .on_change(on_change)
    .build()
```

Placeholders disappear when the user starts typing. Keep them short and actionable.

## Common patterns

**Clear on submit:**
```rust
.on_submit(with!(input => move || {
    process_input(input.get());
    input.set(String::new());
}))
```

**Trim whitespace:**
```rust
.on_submit(with!(input => move || {
    let trimmed = input.get().trim().to_string();
    if !trimmed.is_empty() {
        process_input(trimmed);
    }
    input.set(String::new());
}))
```

**Character counter:**
```rust
let remaining = 280 - input.get().len();
View::text(format!("{} characters remaining", remaining))
```

## Tips

**Don't validate on every keystroke** - Let users type freely, validate on submit or blur.

**Show errors clearly** - If validation fails, show why. Don't silently reject input.

**Preserve cursor position** - If you transform input, the cursor may jump. Consider whether transformations should happen on change or on submit.

**TextArea scrolling** - Content longer than `rows` automatically scrolls. Don't try to fit everything by making rows huge.

Next: [Modals](./modals.md)
