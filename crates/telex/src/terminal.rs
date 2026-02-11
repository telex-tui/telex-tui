use std::io::{self, Stdout, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, poll, Event},
    execute, queue,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use crate::buffer::Buffer;
use crate::canvas::{encode_kitty_graphics, supports_kitty_graphics, PendingCanvas};
use crate::image::{encode_kitty_image, PendingImage};
use crate::render::{render_view, RenderContext};
use crate::theme::current_theme;
use crate::View;

/// Terminal wrapper that handles setup, rendering, and cleanup.
pub struct Terminal {
    stdout: Stdout,
    buffer: Buffer,
    prev_buffer: Buffer,
    headless: bool,
}

impl Terminal {
    /// Create a new terminal instance.
    /// Enters raw mode and the alternate screen.
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;

        let (width, height) = terminal::size()?;
        let buffer = Buffer::new(width, height);
        let prev_buffer = Buffer::new(width, height);

        Ok(Self {
            stdout,
            buffer,
            prev_buffer,
            headless: false,
        })
    }

    /// Create a headless terminal for testing.
    /// Does not enter raw mode or the alternate screen.
    pub fn new_headless(width: u16, height: u16) -> Self {
        let stdout = io::stdout();
        let buffer = Buffer::new(width, height);
        let prev_buffer = Buffer::new(width, height);

        Self {
            stdout,
            buffer,
            prev_buffer,
            headless: true,
        }
    }

    /// Draw a view to the terminal. Returns clamped scroll offsets to apply back to FocusManager.
    pub fn draw(
        &mut self,
        view: &View,
        focus_index: usize,
        focus_visible: bool,
        scroll_offsets: Vec<(u16, u16)>,
        cursor_offsets: Vec<usize>,
        modal_visible: bool,
    ) -> io::Result<Vec<(u16, u16)>> {
        if !self.headless {
            // Check for resize
            let (width, height) = terminal::size()?;
            if width != self.buffer.width || height != self.buffer.height {
                self.buffer = Buffer::new(width, height);
                self.prev_buffer = Buffer::new(width, height);
                // Force full redraw after resize
                execute!(self.stdout, Clear(ClearType::All))?;
            }
        }

        // Fill the buffer with the current theme's background color
        let theme = current_theme();
        self.buffer.fill(theme.foreground, theme.background);

        // Render the view into the buffer
        let area = self.buffer.rect();
        let mut ctx = RenderContext::new(focus_index, focus_visible, scroll_offsets, cursor_offsets, area);
        ctx.set_modal_visible(modal_visible);
        render_view(&mut self.buffer, view, area, &mut ctx);

        // Render overlays (menu dropdowns) after main content
        ctx.render_pending_dropdowns(&mut self.buffer);

        if !self.headless {
            // Get pending canvases and images before finishing with ctx
            let pending_canvases = ctx.take_pending_canvases();
            let pending_images = ctx.take_pending_images();

            // Compute diff and write changes (Pass 1: character buffer)
            self.flush_diff()?;

            // Pass 2: Render canvas graphics via Kitty protocol
            if !pending_canvases.is_empty() {
                self.flush_canvases(&pending_canvases)?;
            }

            // Pass 3: Render images via Kitty protocol
            if !pending_images.is_empty() {
                self.flush_images(&pending_images)?;
            }
        }

        // Swap buffers
        std::mem::swap(&mut self.buffer, &mut self.prev_buffer);

        // Return potentially clamped scroll offsets
        Ok(ctx.scroll_offsets().to_vec())
    }

    /// Get the terminal height (useful for page up/down calculations).
    pub fn height(&self) -> u16 {
        self.buffer.height
    }

    /// Get the terminal width.
    pub fn width(&self) -> u16 {
        self.buffer.width
    }

    /// Get the last rendered frame as a string (headless mode).
    /// The prev_buffer holds the last drawn frame (after swap in draw()).
    pub fn buffer_string(&self) -> String {
        self.prev_buffer.to_string()
    }

    /// Flush only the changed cells to the terminal.
    fn flush_diff(&mut self) -> io::Result<()> {
        let changes = self.buffer.diff(&self.prev_buffer);

        for (x, y, cell) in changes {
            // Skip wide character continuation cells - the wide char already occupies this space.
            // Printing here would overwrite the second half of the emoji/CJK character.
            if cell.wide_continuation {
                continue;
            }

            queue!(self.stdout, MoveTo(x, y))?;

            // Reset attributes first to avoid state leakage
            queue!(self.stdout, SetAttribute(Attribute::Reset))?;

            // Apply text styles
            if cell.bold {
                queue!(self.stdout, SetAttribute(Attribute::Bold))?;
            }
            if cell.italic {
                queue!(self.stdout, SetAttribute(Attribute::Italic))?;
            }
            if cell.underline {
                queue!(self.stdout, SetAttribute(Attribute::Underlined))?;
            }
            if cell.dim {
                queue!(self.stdout, SetAttribute(Attribute::Dim))?;
            }

            // Set colors
            queue!(self.stdout, SetForegroundColor(cell.fg))?;
            queue!(self.stdout, SetBackgroundColor(cell.bg))?;

            queue!(self.stdout, Print(cell.ch))?;
        }

        // Reset attributes at end
        queue!(self.stdout, SetAttribute(Attribute::Reset))?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Flush canvas graphics to terminal via Kitty protocol.
    fn flush_canvases(&mut self, canvases: &[PendingCanvas]) -> io::Result<()> {
        if !supports_kitty_graphics() {
            return Ok(());
        }

        for canvas in canvases {
            let escape_seq =
                encode_kitty_graphics(&canvas.pixels, canvas.cell_x, canvas.cell_y, canvas.id);
            self.stdout.write_all(escape_seq.as_bytes())?;
        }

        self.stdout.flush()?;
        Ok(())
    }

    /// Flush images to terminal via Kitty protocol.
    fn flush_images(&mut self, images: &[PendingImage]) -> io::Result<()> {
        if !supports_kitty_graphics() {
            return Ok(());
        }

        for image in images {
            let escape_seq = encode_kitty_image(&image.data, image.cell_x, image.cell_y, image.id);
            self.stdout.write_all(escape_seq.as_bytes())?;
        }

        self.stdout.flush()?;
        Ok(())
    }

    /// Poll for an input event with the given timeout.
    pub fn poll_event(&self, timeout: std::time::Duration) -> io::Result<Option<Event>> {
        if poll(timeout)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }

    /// Draw debug information overlay.
    pub fn draw_debug(
        &mut self,
        frame: u64,
        render_us: u64,
        focus_idx: usize,
        focusable_count: usize,
    ) -> io::Result<()> {
        let _ = (frame, render_us); // Suppress unused warnings
        let debug_text = format!(" Focus: {}/{} ", focus_idx, focusable_count);

        // Draw at bottom-right corner
        let x = self
            .buffer
            .width
            .saturating_sub(debug_text.len() as u16 + 1);
        let y = 0; // Top of screen

        queue!(
            self.stdout,
            MoveTo(x, y),
            SetForegroundColor(Color::Black),
            SetBackgroundColor(Color::Yellow),
            Print(&debug_text),
            SetForegroundColor(Color::Reset),
            SetBackgroundColor(Color::Reset)
        )?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Clean up the terminal state.
    pub fn cleanup(&mut self) -> io::Result<()> {
        if self.headless {
            return Ok(());
        }

        // Delete any Kitty graphics images
        if supports_kitty_graphics() {
            let delete_cmd = crate::canvas::delete_all_kitty_images();
            let _ = self.stdout.write_all(delete_cmd.as_bytes());
        }

        execute!(
            self.stdout,
            Clear(ClearType::All),
            SetForegroundColor(Color::Reset),
            SetBackgroundColor(Color::Reset),
            Show,
            LeaveAlternateScreen
        )?;
        disable_raw_mode()?;
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        if self.headless {
            return;
        }

        // Best-effort cleanup on drop
        // Delete any Kitty graphics images
        if supports_kitty_graphics() {
            let delete_cmd = crate::canvas::delete_all_kitty_images();
            let _ = self.stdout.write_all(delete_cmd.as_bytes());
        }

        let _ = execute!(
            self.stdout,
            Clear(ClearType::All),
            SetForegroundColor(Color::Reset),
            SetBackgroundColor(Color::Reset),
            Show,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
    }
}
