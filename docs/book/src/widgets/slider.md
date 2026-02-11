# Slider

A bounded numeric input controlled with arrow keys.

```rust
let volume = state!(cx, || 50.0);

View::slider()
    .min(0.0)
    .max(100.0)
    .step(1.0)
    .value(volume.get())
    .label("Volume")
    .on_change(with!(volume => move |v: f64| volume.set(v)))
    .build()
```

## Builder API

```rust
View::slider()
    .min(0.0)           // minimum value (default: 0.0)
    .max(100.0)         // maximum value (default: 100.0)
    .value(50.0)        // current value
    .step(1.0)          // increment per arrow key press (default: 1.0)
    .label("Volume")    // optional label shown alongside the slider
    .on_change(cb)      // callback receives new f64 value
    .build()
```

## Interaction

- **Left arrow** — Decrease by step
- **Right arrow** — Increase by step
- Values are clamped to the `[min, max]` range automatically

## Examples

### Percentage

```rust
let brightness = state!(cx, || 75.0);

View::slider()
    .min(0.0)
    .max(100.0)
    .step(5.0)
    .value(brightness.get())
    .label("Brightness")
    .on_change(with!(brightness => move |v: f64| brightness.set(v)))
    .build()
```

### Fine-grained control

```rust
let opacity = state!(cx, || 1.0);

View::slider()
    .min(0.0)
    .max(1.0)
    .step(0.05)
    .value(opacity.get())
    .label("Opacity")
    .on_change(with!(opacity => move |v: f64| opacity.set(v)))
    .build()
```

### Integer range

```rust
let font_size = state!(cx, || 14.0);

View::slider()
    .min(8.0)
    .max(32.0)
    .step(1.0)
    .value(font_size.get())
    .label("Font Size")
    .on_change(with!(font_size => move |v: f64| font_size.set(v)))
    .build()
```

## Tips

**Step size matters** — A step of 1.0 with a range of 0-100 means 100 key presses to traverse. A step of 5.0 means 20. Choose a step that makes the slider usable.

**Float values** — The slider works with `f64`. If you need integers, round in your callback or when reading the value.

**Label is optional** — Omit `.label()` if the slider's purpose is clear from context.
