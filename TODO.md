# TODO

## Bugs

### render.rs:1078 - usize underflow when input_width is 0
**Status:** Fixed locally, needs commit

```rust
// Before (panics when input_width is 0):
let start = cursor_pos.saturating_sub(input_width - 1);

// After:
let start = cursor_pos.saturating_sub(input_width.saturating_sub(1));
```

Zero-width text inputs cause panic. Discovered via telex-designer where a text input without flex wrapper got squeezed to zero width.

## API Improvements

### TextInputBuilder should have .flex()
Currently you must wrap in `View::boxed().flex(n)` to give a text input flex. Easy to accidentally create zero-width inputs.

Consider adding `.flex()` to all widget builders, or at least the common ones.

### Minimum width for inputs?
Text inputs with zero width silently fail to display. Could enforce a minimum, or at least not panic.
