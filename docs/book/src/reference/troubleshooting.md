# Troubleshooting

Common issues and how to fix them.

## "My app freezes"

You might have a render loop - state being updated during render.

```rust
// WRONG - causes infinite loop
fn render(&self, cx: Scope) -> View {
    let count = state!(cx, || 0);
    count.update(|n| *n += 1);  // triggers re-render, which triggers update...
    View::text(format!("{}", count.get()))
}
```

**Fix:** Only update state in event handlers or effects, never during render.

## "State doesn't update"

Did you forget to call `.get()`?

```rust
// WRONG
View::text(format!("{}", count))  // prints State<i32>, not the value

// RIGHT
View::text(format!("{}", count.get()))
```

## "Hooks called in different order"

If you're using `use_state_keyed` with explicit keys, this won't happen. The `state!` macro handles ordering automatically — each call site gets a unique key based on its location in the source code.

```rust
// SAFE: state! is order-independent
if show_extra {
    let extra = state!(cx, || "");  // fine in conditionals
}
let main = state!(cx, || 0);  // always works
```

If you need shared state across call sites, use `cx.use_state_keyed::<Key, _>(|| init)` with an explicit key type.

## "Effect runs too often"

Check your dependency - effects run when the dependency changes.

```rust
// Runs every render (String clones are never equal by reference)
effect!(cx, name.get(), |n| { ... });

// Consider: is this the right dependency?
```

## "My component doesn't re-render"

If you're modifying data without going through `State::update()` or `State::set()`, the UI won't know to re-render.

```rust
// WRONG - modifies state but doesn't trigger re-render
let items = state!(cx, || vec![1, 2, 3]);
items.get().push(4);  // UI won't update!

// RIGHT - use .update() to modify
items.update(|v| v.push(4));  // triggers re-render
```

## "Stream values are stale"

Streams capture their initial context. If you need fresh state values inside a stream, pass them through the stream's data:

```rust
// WRONG - max captured once, won't update
let max = max_value.get();
let stream = stream!(cx, || {
    (0..).map(move |i| {
        i % max  // uses stale max
    })
});

// RIGHT - recalculate inside the stream
let stream = stream!(cx, || {
    (0..).map(|i| {
        let current_max = get_current_max();  // get fresh value
        i % current_max
    })
});
```

Or restart the stream when dependencies change using the refresh pattern from [Async Data](../dynamic-data/async.md#refreshing-data).

## "Terminal shows weird characters"

Unicode rendering issues can occur in terminals with poor Unicode support or incorrect locale settings.

**Check your locale:**
```bash
echo $LANG
# Should show something like: en_US.UTF-8
```

**If not set:**
```bash
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
```

**Terminal compatibility:**
- Use a modern terminal emulator (Kitty, Ghostty, WezTerm, Alacritty)
- Avoid older terminals that don't fully support Unicode

**For emojis and wide characters:**
- Some terminals render wide characters (emoji, CJK) incorrectly
- If you see overlapping or misaligned text, try a different terminal
- Telex uses grapheme clustering and width calculations, but terminal rendering varies
