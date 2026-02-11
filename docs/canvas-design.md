# Canvas Widget Design

A pixel-level drawing primitive for Telex using the Kitty graphics protocol.

## Goal

Enable visualizations (charts, graphs, sparklines, heatmaps, images) with actual pixel rendering. Requires a Kitty-protocol-compatible terminal.

---

## Supported Terminals

- **Kitty** — native support
- **Ghostty** — Kitty protocol compatible
- **WezTerm** — Kitty protocol support
- **Konsole** — Kitty protocol support (partial)

Not supported: Apple Terminal, GNOME Terminal, xterm, iTerm2, Windows Terminal

**Philosophy:** This is 2025. Use a modern terminal.

---

## Kitty Graphics Protocol

The Kitty graphics protocol transmits images as base64-encoded data via escape sequences. Supports:

- PNG, RGB, RGBA formats
- Placement at specific cells
- Image caching (transmit once, display many times)
- Z-index layering
- Animations (optional)

Reference: https://sw.kovidgoyal.net/kitty/graphics-protocol/

---

## API Design

### Basic Usage

```rust
View::canvas()
    .width(200)   // pixels
    .height(100)  // pixels
    .on_draw(|ctx| {
        ctx.line(0, 0, 200, 100, Color::Red);
        ctx.fill_rect(10, 10, 50, 30, Color::Blue);
        ctx.circle(150, 50, 20, Color::Cyan);
    })
    .build()
```

### DrawContext Primitives

```rust
pub struct DrawContext<'a> {
    buffer: &'a mut PixelBuffer,
}

impl DrawContext<'_> {
    // Basic
    fn pixel(&mut self, x: u16, y: u16, color: Color);
    fn clear(&mut self, color: Color);
    fn dimensions(&self) -> (u16, u16);

    // Shapes
    fn line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: Color);
    fn stroke_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Color);
    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Color);
    fn circle(&mut self, cx: u16, cy: u16, r: u16, color: Color);
    fn fill_circle(&mut self, cx: u16, cy: u16, r: u16, color: Color);

    // Paths
    fn begin_path(&mut self);
    fn move_to(&mut self, x: u16, y: u16);
    fn line_to(&mut self, x: u16, y: u16);
    fn stroke(&mut self, color: Color);
    fn fill(&mut self, color: Color);

    // Text (rasterized into pixels)
    fn text(&mut self, x: u16, y: u16, s: &str, color: Color);

    // Images
    fn image(&mut self, x: u16, y: u16, data: &[u8], format: ImageFormat);
}
```

### High-Level Widgets (Future)

```rust
View::sparkline()
    .data(&[1.0, 2.5, 3.0, 2.0, 4.0])
    .color(Color::Green)
    .build()

View::line_chart()
    .series("cpu", &cpu_data, Color::Red)
    .series("mem", &mem_data, Color::Blue)
    .build()

View::bar_chart()
    .data(&[("A", 10), ("B", 25), ("C", 15)])
    .build()
```

---

## Internal Architecture

```
┌─────────────────────────────────────┐
│          View::Canvas               │
│      (width, height, on_draw)       │
└─────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│           PixelBuffer               │
│    Vec<u8> RGBA at logical res      │
└─────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│        Kitty Encoder                │
│   PixelBuffer → base64 → escapes    │
└─────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│       Terminal Output               │
│   (bypasses Buffer, direct write)   │
└─────────────────────────────────────┘
```

### PixelBuffer

```rust
pub struct PixelBuffer {
    width: u16,
    height: u16,
    data: Vec<u8>,  // RGBA, row-major, 4 bytes per pixel
}

impl PixelBuffer {
    pub fn new(width: u16, height: u16) -> Self;
    pub fn get(&self, x: u16, y: u16) -> (u8, u8, u8, u8);
    pub fn set(&mut self, x: u16, y: u16, r: u8, g: u8, b: u8, a: u8);
    pub fn clear(&mut self, r: u8, g: u8, b: u8, a: u8);
    pub fn as_bytes(&self) -> &[u8];
}
```

### Kitty Protocol Encoding

```rust
pub fn encode_kitty_graphics(
    pixels: &PixelBuffer,
    x: u16,      // cell column
    y: u16,      // cell row
    z: i32,      // z-index (layering)
) -> String {
    // 1. Encode RGBA data as base64
    // 2. Chunk into 4096-byte segments
    // 3. Wrap in escape sequences:
    //    \x1b_Gf=32,s=<width>,v=<height>,a=T,t=d,m=1;<base64chunk>\x1b\\
    //    \x1b_Gm=1;<next chunk>\x1b\\
    //    \x1b_Gm=0;<final chunk>\x1b\\
}
```

### Rendering Integration

Canvas bypasses the character Buffer. Rendering happens in two passes:

1. **Pass 1:** Normal Buffer rendering (all other widgets)
2. **Pass 2:** Canvas widgets write Kitty escape sequences directly

```rust
// In terminal.rs
pub fn draw(&mut self, view: &View, ...) -> io::Result<...> {
    // Pass 1: Character buffer
    render_view(&mut self.buffer, view, area, &mut ctx);
    self.flush_diff()?;

    // Pass 2: Canvas graphics (direct terminal writes)
    ctx.flush_canvas_graphics(&mut self.stdout)?;

    Ok(...)
}
```

---

## View Integration

### View Enum

```rust
pub enum View {
    // ... existing variants ...
    Canvas(CanvasNode),
}

pub struct CanvasNode {
    pub width: u16,
    pub height: u16,
    pub on_draw: Rc<dyn Fn(&mut DrawContext)>,
}
```

### Builder

```rust
impl View {
    pub fn canvas() -> CanvasBuilder {
        CanvasBuilder::new()
    }
}

pub struct CanvasBuilder {
    width: u16,
    height: u16,
    on_draw: Option<Rc<dyn Fn(&mut DrawContext)>>,
}

impl CanvasBuilder {
    pub fn width(mut self, w: u16) -> Self;
    pub fn height(mut self, h: u16) -> Self;
    pub fn on_draw<F: Fn(&mut DrawContext) + 'static>(mut self, f: F) -> Self;
    pub fn build(self) -> View;
}
```

---

## Capability Detection

Simple check at startup:

```rust
pub fn supports_kitty_graphics() -> bool {
    // Option 1: Check $TERM
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("kitty") || term.contains("ghostty") {
            return true;
        }
    }

    // Option 2: Query terminal (send graphics query, check response)
    // More reliable but adds startup latency

    false
}
```

If `supports_kitty_graphics()` is false, Canvas renders as empty space (or a placeholder message). No fallback rendering — just nothing.

---

## Caching

Kitty protocol supports image caching. Transmit once, display by ID:

```rust
View::canvas()
    .cache_key("my_chart_v1")  // Only re-transmit if key changes
    .on_draw(|ctx| { ... })
    .build()
```

Implementation:
1. Hash the PixelBuffer content
2. If hash matches cached image ID, just send placement command
3. If new, transmit image and store ID

---

## Implementation Phases

### Phase 1: Core ✅ DONE
- [x] `PixelBuffer` struct
- [x] `DrawContext` with pixel, line, rect
- [x] Kitty encoder (RGBA → base64 → escape sequences)
- [x] `View::Canvas` node and builder
- [x] Two-pass rendering in terminal.rs
- [x] Basic capability detection
- [x] Example: colored rectangles (29_canvas)

### Phase 2: Drawing Primitives ✅ DONE
- [x] Circle, fill_circle
- [ ] Path API (begin_path, move_to, line_to, stroke, fill)
- [ ] Anti-aliased lines (Xiaolin Wu's algorithm)
- [ ] Example: line graph

### Phase 3: Caching & Performance
- [ ] Image caching by content hash
- [ ] Dirty region tracking
- [ ] Placement reuse (move image without retransmit)

### Phase 4: High-Level Widgets
- [ ] `View::sparkline()`
- [ ] `View::line_chart()`
- [ ] `View::bar_chart()`
- [ ] `View::gauge()`

### Phase 5: Image Widget ✅ DONE

Display image files (PNG, GIF, JPEG) with Kitty handling format detection and GIF animation.

```rust
// Display static image (embed at compile time)
View::image()
    .data(include_bytes!("logo.png"))
    .build()

// Display animated GIF (Kitty handles animation natively)
View::image()
    .file("assets/loading.gif")
    .build()
```

Implementation:
- [x] `ImageNode` and `ImageBuilder`
- [x] Kitty encoding with `f=100` (PNG format, auto-detected by Kitty)
- [x] File loading option (lazy load at render time)
- [x] Dimension detection from PNG/GIF/JPEG headers
- [x] Example: 30_image

### Phase 6: Animated Canvas ✅ DONE

Frame-based animations with automatic timing management.

```rust
use telex::canvas::animated_canvas;

animated_canvas(cx)
    .width(200)
    .height(100)
    .fps(30)
    .on_frame(|ctx, frame| {
        ctx.clear(Color::Black);
        let x = (frame % 200) as u16;
        ctx.fill_circle(x, 50, 10, Color::Red);
    })
    .build()
```

Implementation:
- [x] Frame timing management via internal stream
- [x] Callback-based frame rendering with frame number
- [x] Configurable FPS (default: 30)
- [x] Example: 31_animated_canvas (bouncing ball, sine waves, particles)

---

## Open Questions

1. **Text rendering**: Rasterize text into pixels? Requires font handling. Maybe punt and say "use regular View::text() outside the canvas".

2. **Transparency**: Kitty supports alpha. Do we expose it fully?

3. **Animation**: Kitty supports animated GIFs. Worth exposing?

4. **Mouse**: Kitty can report mouse clicks on images. Future feature?

---

## Dependencies

```toml
[dependencies]
base64 = "0.21"  # For Kitty protocol encoding
```

Minimal. No image crate needed unless we add image loading.

---

## Example

```rust
use telex::prelude::*;

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let data = cx.use_state(|| vec![10.0, 25.0, 15.0, 30.0, 20.0, 35.0, 25.0]);

        View::vstack()
            .child(View::text("Sales Chart"))
            .child(
                View::canvas()
                    .width(200)
                    .height(100)
                    .on_draw({
                        let data = data.get();
                        move |ctx| {
                            ctx.clear(Color::Black);
                            let max = data.iter().cloned().fold(0.0_f64, f64::max);
                            let bar_width = 200 / data.len() as u16;

                            for (i, &val) in data.iter().enumerate() {
                                let x = i as u16 * bar_width;
                                let h = ((val / max) * 100.0) as u16;
                                ctx.fill_rect(x, 100 - h, bar_width - 2, h, Color::Green);
                            }
                        }
                    })
                    .build(),
            )
            .build()
    }
}
```

---

## Success Criteria

Run `cargo run -p telex --example canvas_demo` in Kitty/Ghostty/WezTerm and see actual pixel graphics. Run in Apple Terminal and see empty space (expected — use a real terminal).
