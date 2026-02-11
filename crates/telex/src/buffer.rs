use crossterm::style::Color;
use std::fmt;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Parameters for cell styling.
#[derive(Debug, Clone, Copy)]
struct StyleParams {
    fg: Color,
    bg: Color,
    bold: bool,
    italic: bool,
    underline: bool,
    dim: bool,
}

/// A single cell in the terminal buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
    /// True if this cell is the second half of a wide character (e.g., emoji, CJK).
    /// The renderer should skip these cells since the wide char already occupies the space.
    pub wide_continuation: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
            wide_continuation: false,
        }
    }
}

impl Cell {
    /// Create a new cell with just character and colors.
    pub fn new(ch: char, fg: Color, bg: Color) -> Self {
        Self {
            ch,
            fg,
            bg,
            ..Default::default()
        }
    }

    /// Create a styled cell.
    pub fn styled(
        ch: char,
        fg: Color,
        bg: Color,
        bold: bool,
        italic: bool,
        underline: bool,
        dim: bool,
    ) -> Self {
        Self {
            ch,
            fg,
            bg,
            bold,
            italic,
            underline,
            dim,
            wide_continuation: false,
        }
    }

    /// Create a wide character continuation cell (second half of emoji/CJK).
    /// The renderer should skip these cells.
    pub fn wide_continuation(fg: Color, bg: Color) -> Self {
        Self {
            ch: ' ',
            fg,
            bg,
            wide_continuation: true,
            ..Default::default()
        }
    }

    /// Create a styled wide character continuation cell.
    pub fn wide_continuation_styled(
        fg: Color,
        bg: Color,
        bold: bool,
        italic: bool,
        underline: bool,
        dim: bool,
    ) -> Self {
        Self {
            ch: ' ',
            fg,
            bg,
            bold,
            italic,
            underline,
            dim,
            wide_continuation: true,
        }
    }
}

/// A rectangular region within the buffer.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// A 2D buffer of cells representing the terminal screen.
#[derive(Debug, Clone)]
pub struct Buffer {
    cells: Vec<Cell>,
    pub width: u16,
    pub height: u16,
}

impl Buffer {
    /// Create a new buffer filled with empty cells.
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            cells: vec![Cell::default(); size],
            width,
            height,
        }
    }

    /// Get the index into the cells vector for a given position.
    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y as usize) * (self.width as usize) + (x as usize))
        } else {
            None
        }
    }

    /// Get a cell at a position.
    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        self.index(x, y).map(|i| &self.cells[i])
    }

    /// Set a cell at a position.
    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = cell;
        }
    }

    /// Set a character at a position with colors.
    pub fn set(&mut self, x: u16, y: u16, ch: char, fg: Color, bg: Color) {
        self.set_cell(x, y, Cell::new(ch, fg, bg));
    }

    /// Write a string at a position, clipping at buffer boundaries.
    ///
    /// Handles grapheme clusters and wide characters (CJK, emoji) properly:
    /// - Iterates by grapheme clusters (user-perceived characters)
    /// - Wide characters (display width 2) advance the column by 2
    /// - Wide char continuations are marked so the renderer can skip them
    pub fn write_str(&mut self, x: u16, y: u16, s: &str, fg: Color, bg: Color) {
        let mut col = x;
        for grapheme in s.graphemes(true) {
            if col >= self.width {
                break;
            }
            let width = UnicodeWidthStr::width(grapheme);
            if width == 0 {
                continue; // Skip zero-width characters
            }

            // For wide characters, check if we have room for both columns
            if width == 2 && col + 1 >= self.width {
                // Wide char would overflow - write a space instead and stop
                self.set_cell(col, y, Cell::new(' ', fg, bg));
                break;
            }

            // Use first char of grapheme for the cell
            // (Full grapheme cluster support would require Cell to store String)
            let ch = grapheme.chars().next().unwrap_or(' ');
            self.set_cell(col, y, Cell::new(ch, fg, bg));

            // For wide characters, mark the next cell as a continuation
            if width == 2 {
                self.set_cell(col + 1, y, Cell::wide_continuation(fg, bg));
            }
            col += width as u16;
        }
    }

    /// Write a styled string at a position, clipping at buffer boundaries.
    ///
    /// Handles grapheme clusters and wide characters (CJK, emoji) properly:
    /// - Iterates by grapheme clusters (user-perceived characters)
    /// - Wide characters (display width 2) advance the column by 2
    /// - Wide char continuations are marked so the renderer can skip them
    #[allow(clippy::too_many_arguments)]
    pub fn write_str_styled(
        &mut self,
        x: u16,
        y: u16,
        s: &str,
        fg: Color,
        bg: Color,
        bold: bool,
        italic: bool,
        underline: bool,
        dim: bool,
    ) {
        let style = StyleParams {
            fg,
            bg,
            bold,
            italic,
            underline,
            dim,
        };
        self.write_str_styled_impl(x, y, s, style);
    }

    /// Internal implementation of write_str_styled using StyleParams.
    fn write_str_styled_impl(&mut self, x: u16, y: u16, s: &str, style: StyleParams) {
        let mut col = x;
        for grapheme in s.graphemes(true) {
            if col >= self.width {
                break;
            }
            let width = UnicodeWidthStr::width(grapheme);
            if width == 0 {
                continue; // Skip zero-width characters
            }

            // For wide characters, check if we have room for both columns
            if width == 2 && col + 1 >= self.width {
                // Wide char would overflow - write a space instead and stop
                self.set_cell(
                    col,
                    y,
                    Cell::styled(' ', style.fg, style.bg, style.bold, style.italic, style.underline, style.dim),
                );
                break;
            }

            // Use first char of grapheme for the cell
            // (Full grapheme cluster support would require Cell to store String)
            let ch = grapheme.chars().next().unwrap_or(' ');
            self.set_cell(
                col,
                y,
                Cell::styled(ch, style.fg, style.bg, style.bold, style.italic, style.underline, style.dim),
            );

            // For wide characters, mark the next cell as a continuation
            if width == 2 {
                self.set_cell(
                    col + 1,
                    y,
                    Cell::wide_continuation_styled(style.fg, style.bg, style.bold, style.italic, style.underline, style.dim),
                );
            }
            col += width as u16;
        }
    }

    /// Get the full rect of this buffer.
    pub fn rect(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    /// Clear the buffer to empty cells.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
    }

    /// Fill the buffer with a specific foreground and background color.
    pub fn fill(&mut self, fg: Color, bg: Color) {
        for cell in &mut self.cells {
            cell.ch = ' ';
            cell.fg = fg;
            cell.bg = bg;
            cell.bold = false;
            cell.italic = false;
            cell.underline = false;
            cell.dim = false;
        }
    }

    /// Convert the buffer to a string (for testing/snapshots).
    fn buffer_to_string(&self) -> String {
        let mut result = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(cell) = self.get(x, y) {
                    // Skip wide character continuations - the wide char already added its character
                    if cell.wide_continuation {
                        continue;
                    }
                    result.push(cell.ch);
                }
            }
            // Trim trailing spaces from each line
            let trimmed = result.trim_end_matches(' ');
            result.truncate(trimmed.len());
            result.push('\n');
        }
        // Remove trailing empty lines
        while result.ends_with("\n\n") {
            result.pop();
        }
        result
    }

    /// Compute the differences between this buffer and another.
    /// Returns a list of (x, y, cell) for cells that differ.
    pub fn diff<'a>(&'a self, other: &'a Buffer) -> Vec<(u16, u16, &'a Cell)> {
        let mut changes = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let self_cell = self.get(x, y);
                let other_cell = other.get(x, y);

                match (self_cell, other_cell) {
                    (Some(a), Some(b)) if a != b => {
                        changes.push((x, y, a));
                    }
                    (Some(a), None) => {
                        changes.push((x, y, a));
                    }
                    _ => {}
                }
            }
        }

        changes
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.buffer_to_string())
    }
}
