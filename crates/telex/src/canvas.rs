//! Canvas widget for pixel-level drawing using the Kitty graphics protocol.
//!
//! This module provides:
//! - `PixelBuffer` - RGBA pixel storage
//! - `DrawContext` - Drawing API for the `on_draw` callback
//! - `animated_canvas` - Helper for frame-based animation
//! - Kitty protocol encoding for terminal output
//!
//! # Animated Canvas Example
//!
//! ```rust,ignore
//! use telex::prelude::*;
//! use telex::canvas::animated_canvas;
//!
//! fn App(cx: Scope) -> View {
//!     animated_canvas(cx)
//!         .width(200)
//!         .height(100)
//!         .fps(30)
//!         .on_frame(|ctx, frame| {
//!             ctx.clear(Color::Black);
//!             let x = (frame % 200) as u16;
//!             ctx.fill_circle(x, 50, 10, Color::Red);
//!         })
//!         .build()
//! }
//! ```

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use crossterm::style::Color;
use std::rc::Rc;
use std::time::Duration;

use crate::Scope;
use crate::View;

/// Type alias for the frame drawing callback.
type FrameCallback = Option<Rc<dyn Fn(&mut DrawContext, u64)>>;

/// RGBA pixel buffer for canvas rendering.
#[derive(Clone)]
pub struct PixelBuffer {
    width: u16,
    height: u16,
    /// RGBA data, row-major, 4 bytes per pixel
    data: Vec<u8>,
}

impl PixelBuffer {
    /// Create a new pixel buffer filled with transparent black.
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize) * 4;
        Self {
            width,
            height,
            data: vec![0; size],
        }
    }

    /// Get buffer dimensions.
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Get pixel at (x, y) as (r, g, b, a).
    pub fn get(&self, x: u16, y: u16) -> (u8, u8, u8, u8) {
        if x >= self.width || y >= self.height {
            return (0, 0, 0, 0);
        }
        let idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
        (
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        )
    }

    /// Set pixel at (x, y) with RGBA values.
    pub fn set(&mut self, x: u16, y: u16, r: u8, g: u8, b: u8, a: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
        self.data[idx] = r;
        self.data[idx + 1] = g;
        self.data[idx + 2] = b;
        self.data[idx + 3] = a;
    }

    /// Clear the buffer to a solid color.
    pub fn clear(&mut self, r: u8, g: u8, b: u8, a: u8) {
        for i in (0..self.data.len()).step_by(4) {
            self.data[i] = r;
            self.data[i + 1] = g;
            self.data[i + 2] = b;
            self.data[i + 3] = a;
        }
    }

    /// Get raw RGBA bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

/// Drawing context passed to the canvas `on_draw` callback.
///
/// Provides primitives for drawing pixels, lines, rectangles, and circles.
pub struct DrawContext<'a> {
    buffer: &'a mut PixelBuffer,
}

impl<'a> DrawContext<'a> {
    /// Create a new draw context wrapping a pixel buffer.
    pub fn new(buffer: &'a mut PixelBuffer) -> Self {
        Self { buffer }
    }

    /// Get canvas dimensions (width, height) in pixels.
    pub fn dimensions(&self) -> (u16, u16) {
        self.buffer.dimensions()
    }

    /// Clear the canvas to a solid color.
    pub fn clear(&mut self, color: Color) {
        let (r, g, b) = color_to_rgb(color);
        self.buffer.clear(r, g, b, 255);
    }

    /// Set a single pixel.
    pub fn pixel(&mut self, x: u16, y: u16, color: Color) {
        let (r, g, b) = color_to_rgb(color);
        self.buffer.set(x, y, r, g, b, 255);
    }

    /// Set a pixel with alpha.
    pub fn pixel_alpha(&mut self, x: u16, y: u16, color: Color, alpha: u8) {
        let (r, g, b) = color_to_rgb(color);
        self.buffer.set(x, y, r, g, b, alpha);
    }

    /// Draw a line using Bresenham's algorithm.
    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color) {
        let (r, g, b) = color_to_rgb(color);

        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x1;
        let mut y = y1;

        loop {
            if x >= 0 && y >= 0 {
                self.buffer.set(x as u16, y as u16, r, g, b, 255);
            }

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Draw a stroked rectangle (outline only).
    pub fn stroke_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Color) {
        if w == 0 || h == 0 {
            return;
        }
        let x2 = x.saturating_add(w).saturating_sub(1);
        let y2 = y.saturating_add(h).saturating_sub(1);

        // Top and bottom
        self.line(x as i32, y as i32, x2 as i32, y as i32, color);
        self.line(x as i32, y2 as i32, x2 as i32, y2 as i32, color);
        // Left and right
        self.line(x as i32, y as i32, x as i32, y2 as i32, color);
        self.line(x2 as i32, y as i32, x2 as i32, y2 as i32, color);
    }

    /// Draw a filled rectangle.
    pub fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Color) {
        let (r, g, b) = color_to_rgb(color);
        for dy in 0..h {
            for dx in 0..w {
                self.buffer
                    .set(x.saturating_add(dx), y.saturating_add(dy), r, g, b, 255);
            }
        }
    }

    /// Draw a stroked circle (outline only) using midpoint algorithm.
    pub fn circle(&mut self, cx: u16, cy: u16, radius: u16, color: Color) {
        if radius == 0 {
            self.pixel(cx, cy, color);
            return;
        }

        let (r, g, b) = color_to_rgb(color);
        let cx = cx as i32;
        let cy = cy as i32;
        let mut x = radius as i32;
        let mut y = 0i32;
        let mut err = 1 - x;

        while x >= y {
            // Draw 8 octants
            self.set_pixel_safe(cx + x, cy + y, r, g, b);
            self.set_pixel_safe(cx + y, cy + x, r, g, b);
            self.set_pixel_safe(cx - y, cy + x, r, g, b);
            self.set_pixel_safe(cx - x, cy + y, r, g, b);
            self.set_pixel_safe(cx - x, cy - y, r, g, b);
            self.set_pixel_safe(cx - y, cy - x, r, g, b);
            self.set_pixel_safe(cx + y, cy - x, r, g, b);
            self.set_pixel_safe(cx + x, cy - y, r, g, b);

            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x + 1);
            }
        }
    }

    /// Draw a filled circle.
    pub fn fill_circle(&mut self, cx: u16, cy: u16, radius: u16, color: Color) {
        let (r, g, b) = color_to_rgb(color);
        let cx = cx as i32;
        let cy = cy as i32;
        let radius = radius as i32;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    self.set_pixel_safe(cx + dx, cy + dy, r, g, b);
                }
            }
        }
    }

    /// Helper to set pixel with bounds checking for signed coords.
    fn set_pixel_safe(&mut self, x: i32, y: i32, r: u8, g: u8, b: u8) {
        if x >= 0 && y >= 0 {
            self.buffer.set(x as u16, y as u16, r, g, b, 255);
        }
    }
}

/// Convert crossterm Color to RGB tuple.
fn color_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb { r, g, b } => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::DarkGrey => (128, 128, 128),
        Color::Red => (255, 0, 0),
        Color::DarkRed => (139, 0, 0),
        Color::Green => (0, 255, 0),
        Color::DarkGreen => (0, 100, 0),
        Color::Yellow => (255, 255, 0),
        Color::DarkYellow => (128, 128, 0),
        Color::Blue => (0, 0, 255),
        Color::DarkBlue => (0, 0, 139),
        Color::Magenta => (255, 0, 255),
        Color::DarkMagenta => (139, 0, 139),
        Color::Cyan => (0, 255, 255),
        Color::DarkCyan => (0, 139, 139),
        Color::White => (255, 255, 255),
        Color::Grey => (192, 192, 192),
        Color::Reset => (0, 0, 0), // Default to black
        Color::AnsiValue(v) => ansi_to_rgb(v),
    }
}

/// Convert ANSI 256-color to RGB.
fn ansi_to_rgb(code: u8) -> (u8, u8, u8) {
    match code {
        0..=15 => {
            // Standard colors
            let colors = [
                (0, 0, 0),
                (128, 0, 0),
                (0, 128, 0),
                (128, 128, 0),
                (0, 0, 128),
                (128, 0, 128),
                (0, 128, 128),
                (192, 192, 192),
                (128, 128, 128),
                (255, 0, 0),
                (0, 255, 0),
                (255, 255, 0),
                (0, 0, 255),
                (255, 0, 255),
                (0, 255, 255),
                (255, 255, 255),
            ];
            colors[code as usize]
        }
        16..=231 => {
            // 216-color cube
            let n = code - 16;
            let r = (n / 36) % 6;
            let g = (n / 6) % 6;
            let b = n % 6;
            let to_rgb = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            (to_rgb(r), to_rgb(g), to_rgb(b))
        }
        232..=255 => {
            // Grayscale
            let gray = 8 + (code - 232) * 10;
            (gray, gray, gray)
        }
    }
}

// =============================================================================
// Kitty Graphics Protocol Encoding
// =============================================================================

/// A pending canvas graphic to be rendered after the character buffer.
#[derive(Clone)]
pub struct PendingCanvas {
    /// Cell column position
    pub cell_x: u16,
    /// Cell row position
    pub cell_y: u16,
    /// Pixel buffer to render
    pub pixels: PixelBuffer,
    /// Unique ID for this canvas (for caching/replacement)
    pub id: u32,
}

/// Check if the terminal supports Kitty graphics protocol.
pub fn supports_kitty_graphics() -> bool {
    if let Ok(term) = std::env::var("TERM") {
        let term_lower = term.to_lowercase();
        if term_lower.contains("kitty") || term_lower.contains("ghostty") {
            return true;
        }
    }

    // Also check TERM_PROGRAM for terminals that don't set TERM correctly
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        let program_lower = term_program.to_lowercase();
        if program_lower.contains("kitty")
            || program_lower.contains("ghostty")
            || program_lower.contains("wezterm")
        {
            return true;
        }
    }

    false
}

/// Encode a pixel buffer as Kitty graphics protocol escape sequences.
///
/// Returns the complete escape sequence string to write to the terminal.
pub fn encode_kitty_graphics(
    pixels: &PixelBuffer,
    cell_x: u16,
    cell_y: u16,
    image_id: u32,
) -> String {
    let (width, height) = pixels.dimensions();
    if width == 0 || height == 0 {
        return String::new();
    }

    // Encode RGBA data as base64
    let b64_data = BASE64.encode(pixels.as_bytes());

    // Kitty protocol uses chunked transmission for large images
    // Max chunk size is 4096 bytes of base64 data
    const CHUNK_SIZE: usize = 4096;

    let mut result = String::new();
    let chunks: Vec<&str> = b64_data
        .as_bytes()
        .chunks(CHUNK_SIZE)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect();

    let total_chunks = chunks.len();

    for (i, chunk) in chunks.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == total_chunks - 1;
        let more = if is_last { 0 } else { 1 };

        if is_first {
            // First chunk includes full header
            // a=T means transmit and display
            // f=32 means RGBA format
            // t=d means direct (data follows)
            // i=ID for image identification
            // p=1 means placement (display at cursor)
            // q=2 suppresses responses
            result.push_str(&format!(
                "\x1b_Ga=T,f=32,s={},v={},i={},t=d,m={},q=2;{}\x1b\\",
                width, height, image_id, more, chunk
            ));
        } else {
            // Continuation chunks
            result.push_str(&format!("\x1b_Gm={};{}\x1b\\", more, chunk));
        }
    }

    // Position the image at the specified cell
    // We use the cursor position before sending the image
    result.insert_str(0, &format!("\x1b[{};{}H", cell_y + 1, cell_x + 1));

    result
}

/// Delete a previously displayed image by ID.
pub fn delete_kitty_image(image_id: u32) -> String {
    // d=I means delete by image ID
    // q=2 suppresses responses
    format!("\x1b_Ga=d,d=I,i={},q=2\x1b\\", image_id)
}

/// Delete all Kitty graphics images.
pub fn delete_all_kitty_images() -> String {
    // d=a means delete all
    // q=2 suppresses responses
    "\x1b_Ga=d,d=a,q=2\x1b\\".to_string()
}

// =============================================================================
// Animated Canvas
// =============================================================================

/// Create an animated canvas with automatic frame management.
///
/// This is a convenience helper that manages frame timing internally using
/// a stream. The `on_frame` callback receives both the draw context and the
/// current frame number.
///
/// # Example
///
/// ```rust,ignore
/// use telex::canvas::animated_canvas;
///
/// animated_canvas(cx)
///     .width(200)
///     .height(100)
///     .fps(30)  // 30 frames per second
///     .on_frame(|ctx, frame| {
///         ctx.clear(Color::Black);
///         // Animate based on frame number
///         let x = (frame % 200) as u16;
///         ctx.fill_circle(x, 50, 10, Color::Red);
///     })
///     .build()
/// ```
///
/// # Note
///
/// The FPS value is set on first render and cannot be changed dynamically.
/// If you need dynamic FPS control, use `View::canvas()` with `cx.use_stream()`
/// directly.
pub fn animated_canvas(cx: Scope) -> AnimatedCanvasBuilder {
    AnimatedCanvasBuilder::new(cx)
}

/// Builder for animated canvas with automatic frame management.
pub struct AnimatedCanvasBuilder {
    cx: Scope,
    width: u16,
    height: u16,
    fps: u32,
    on_frame: FrameCallback,
}

impl AnimatedCanvasBuilder {
    /// Create a new animated canvas builder.
    pub fn new(cx: Scope) -> Self {
        Self {
            cx,
            width: 100,
            height: 50,
            fps: 30,
            on_frame: None,
        }
    }

    /// Set the canvas width in pixels.
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Set the canvas height in pixels.
    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    /// Set the target frames per second (default: 30).
    ///
    /// Note: This is set on first render. Changing it after the first
    /// render has no effect.
    pub fn fps(mut self, fps: u32) -> Self {
        self.fps = fps.max(1); // Ensure at least 1 FPS
        self
    }

    /// Set the frame drawing callback.
    ///
    /// The callback receives a `DrawContext` for drawing and the current
    /// frame number (starting from 0).
    pub fn on_frame<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut DrawContext, u64) + 'static,
    {
        self.on_frame = Some(Rc::new(f));
        self
    }

    /// Build the animated canvas View.
    ///
    /// This creates an internal stream that ticks at the specified FPS
    /// and returns a Canvas view that calls your `on_frame` callback
    /// with the current frame number.
    pub fn build(self) -> View {
        let delay_ms = 1000 / self.fps as u64;

        // Create a frame counter stream that ticks at the specified FPS
        struct AnimatedCanvasStreamKey;
        let frame_stream = self.cx.use_stream_keyed::<AnimatedCanvasStreamKey, _, _, _>(move || {
            (0u64..).inspect(move |&i| {
                if i > 0 {
                    std::thread::sleep(Duration::from_millis(delay_ms));
                }
            })
        });

        let current_frame = frame_stream.get();
        let on_frame = self.on_frame;
        let width = self.width;
        let height = self.height;

        // Return a regular Canvas that calls on_frame with the current frame number
        View::canvas()
            .width(width)
            .height(height)
            .on_draw(move |ctx| {
                if let Some(ref callback) = on_frame {
                    callback(ctx, current_frame);
                }
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_buffer_new() {
        let buf = PixelBuffer::new(10, 10);
        assert_eq!(buf.dimensions(), (10, 10));
        assert_eq!(buf.get(0, 0), (0, 0, 0, 0));
    }

    #[test]
    fn test_pixel_buffer_set_get() {
        let mut buf = PixelBuffer::new(10, 10);
        buf.set(5, 5, 255, 128, 64, 255);
        assert_eq!(buf.get(5, 5), (255, 128, 64, 255));
    }

    #[test]
    fn test_pixel_buffer_clear() {
        let mut buf = PixelBuffer::new(10, 10);
        buf.clear(100, 150, 200, 255);
        assert_eq!(buf.get(0, 0), (100, 150, 200, 255));
        assert_eq!(buf.get(9, 9), (100, 150, 200, 255));
    }

    #[test]
    fn test_draw_context_line() {
        let mut buf = PixelBuffer::new(10, 10);
        {
            let mut ctx = DrawContext::new(&mut buf);
            ctx.line(0, 0, 9, 0, Color::White);
        }
        // Check horizontal line was drawn
        assert_eq!(buf.get(0, 0), (255, 255, 255, 255));
        assert_eq!(buf.get(5, 0), (255, 255, 255, 255));
        assert_eq!(buf.get(9, 0), (255, 255, 255, 255));
    }

    #[test]
    fn test_draw_context_fill_rect() {
        let mut buf = PixelBuffer::new(10, 10);
        {
            let mut ctx = DrawContext::new(&mut buf);
            ctx.fill_rect(2, 2, 3, 3, Color::Red);
        }
        assert_eq!(buf.get(2, 2), (255, 0, 0, 255));
        assert_eq!(buf.get(4, 4), (255, 0, 0, 255));
        assert_eq!(buf.get(1, 1), (0, 0, 0, 0)); // Outside rect
    }

    #[test]
    fn test_color_to_rgb() {
        assert_eq!(color_to_rgb(Color::Red), (255, 0, 0));
        assert_eq!(color_to_rgb(Color::Green), (0, 255, 0));
        assert_eq!(color_to_rgb(Color::Blue), (0, 0, 255));
        assert_eq!(
            color_to_rgb(Color::Rgb {
                r: 100,
                g: 150,
                b: 200
            }),
            (100, 150, 200)
        );
    }
}
