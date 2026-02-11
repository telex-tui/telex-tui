# Canvas

> ⚠️ **Experimental Feature**
> Canvas requires the Kitty graphics protocol and only works in compatible terminals (Kitty, Ghostty, WezTerm).
> Other terminals will show a placeholder message.

Pixel-level drawing using the Kitty graphics protocol for charts, graphs, and visualizations.

```rust
View::canvas()
    .width(200)
    .height(100)
    .on_draw(|ctx| {
        ctx.clear(Color::Black);
        ctx.line(0, 0, 200, 100, Color::Red);
        ctx.fill_rect(50, 50, 100, 40, Color::Blue);
        ctx.circle(150, 50, 20, Color::Green);
    })
    .build()
```

Run with: `cargo run -p telex-tui --example 29_canvas`

## Terminal compatibility

**⚠️ Important:** Canvas requires terminals that support the Kitty graphics protocol. Check compatibility before using canvas widgets.

**✅ Supported terminals:**
- Kitty
- Ghostty
- WezTerm (with Kitty protocol enabled)

**❌ Not supported:**
- iTerm2 (uses different protocol)
- Alacritty (no inline graphics)
- Standard terminal.app

If your terminal doesn't support Kitty graphics, canvas widgets won't render. No fallback is provided - use a compatible terminal or stick to character-based visualizations.

## Basic canvas

Create a canvas with width and height in pixels:

```rust
View::canvas()
    .width(400)  // pixels wide
    .height(300)  // pixels tall
    .on_draw(|ctx| {
        // Drawing code here
    })
    .build()
```

The `on_draw` closure receives a drawing context with primitive operations.

## Drawing primitives

### Clear

Fill the entire canvas with a color:

```rust
ctx.clear(Color::Rgb { r: 30, g: 30, b: 40 });
```

### Lines

Draw lines between two points:

```rust
ctx.line(x1, y1, x2, y2, color);

// Example: horizontal line
ctx.line(0, 50, 400, 50, Color::Red);

// Example: diagonal
ctx.line(0, 0, 400, 300, Color::Blue);
```

### Rectangles

Draw filled rectangles:

```rust
ctx.fill_rect(x, y, width, height, color);

// Example: red square at (100, 100)
ctx.fill_rect(100, 100, 50, 50, Color::Red);
```

### Circles

Draw circles (filled):

```rust
ctx.circle(center_x, center_y, radius, color);

// Example: green circle
ctx.circle(200, 150, 30, Color::Green);
```

## Coordinate system

- Origin `(0, 0)` is **top-left**
- X increases to the right
- Y increases downward
- Units are pixels

## Colors

Use `Color` enum for drawing:

```rust
Color::Red
Color::Blue
Color::Green
Color::Cyan
Color::Magenta
Color::Yellow
Color::White
Color::Black
Color::Rgb { r: 100, g: 150, b: 200 }
```

## Animation

Combine canvas with streams for animated graphics:

```rust
let frame = cx.use_stream(|| {
    (0u32..).map(|i| {
        std::thread::sleep(Duration::from_millis(50));
        i
    })
});

View::canvas()
    .width(200)
    .height(100)
    .on_draw({
        let current_frame = frame.get();
        move |ctx| {
            ctx.clear(Color::Black);

            // Animate position
            let x = (current_frame % 200) as i32;
            ctx.circle(x, 50, 10, Color::Red);
        }
    })
    .build()
```

Each frame triggers a re-render with updated position.

## Bar charts

Visualize data with bar charts:

```rust
let data = vec![30, 50, 40, 80, 60];

View::canvas()
    .width(200)
    .height(100)
    .on_draw(move |ctx| {
        ctx.clear(Color::Black);

        let bar_width = 200 / data.len() as i32;

        for (i, &value) in data.iter().enumerate() {
            let x = i as i32 * bar_width;
            let height = value as i32;
            let y = 100 - height;

            ctx.fill_rect(x, y, bar_width - 2, height, Color::Cyan);
        }
    })
    .build()
```

## Line graphs

Plot data points with lines:

```rust
let points = vec![(0, 50), (50, 30), (100, 70), (150, 40), (200, 60)];

View::canvas()
    .width(200)
    .height(100)
    .on_draw(move |ctx| {
        ctx.clear(Color::Black);

        for i in 0..points.len() - 1 {
            let (x1, y1) = points[i];
            let (x2, y2) = points[i + 1];
            ctx.line(x1, y1, x2, y2, Color::Green);
        }
    })
    .build()
```

## Performance

Canvas operations are efficient - redrawing 60 times per second is feasible for simple graphics.

For complex scenes:
- Limit drawing operations
- Cache static elements
- Use lower frame rates

## Real-world uses

**System monitoring:**
```rust
let cpu = cpu_usage();  // 0-100

ctx.fill_rect(0, 0, cpu * 2, 20, Color::Green);
```

**Progress visualization:**
```rust
let progress = 0.7;  // 70%

let width = (400.0 * progress) as i32;
ctx.fill_rect(0, 0, width, 30, Color::Blue);
```

**Sparklines:**
```rust
let history = vec![10, 15, 13, 18, 22, 20, 25];

for (i, &v) in history.iter().enumerate() {
    let x = i as i32 * 5;
    let y = 30 - v;
    ctx.fill_rect(x, y, 4, v, Color::Cyan);
}
```

## Tips

**Check terminal compatibility** - Test in Kitty/Ghostty/WezTerm. Canvas won't work in all terminals.

**Pixel coordinates** - Canvas uses pixel units, not character cells.

**Clear first** - Always call `ctx.clear()` at the start of your draw function to avoid artifacts.

**Keep it simple** - Complex graphics can be slow. Test performance if animating.

**Alternative: ASCII art** - For terminal-agnostic apps, use character-based visualizations instead of canvas.

**Size matters** - Larger canvases use more resources. Don't make canvases bigger than needed.

**No text rendering** - Canvas doesn't support text. Use regular TUI widgets for labels.

Next: [Keyed State](../advanced/keyed-state.md)
