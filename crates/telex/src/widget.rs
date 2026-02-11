//! Custom widget trait for user-defined character-cell rendering.
//!
//! The `Widget` trait allows users to create custom views that render
//! directly to the terminal buffer. This is an escape hatch for UIs
//! that can't be composed from built-in widgets.
//!
//! # Example
//! ```rust,ignore
//! use telex::prelude::*;
//! use telex::widget::Widget;
//! use telex::buffer::{Buffer, Rect};
//!
//! struct PianoRoll { notes: Vec<u8> }
//!
//! impl Widget for PianoRoll {
//!     fn render(&self, area: Rect, buf: &mut Buffer) {
//!         for (i, &note) in self.notes.iter().enumerate() {
//!             if i as u16 >= area.width { break; }
//!             let ch = if note > 0 { '█' } else { '░' };
//!             buf.set(area.x + i as u16, area.y, ch,
//!                 crossterm::style::Color::White,
//!                 crossterm::style::Color::Black);
//!         }
//!     }
//! }
//! ```

use crate::buffer::{Buffer, Rect};

/// Trait for custom character-cell widgets.
///
/// Implement this trait to create widgets that render directly to the
/// terminal buffer. Use `View::custom()` to wrap your widget in a View.
pub trait Widget {
    /// Render the widget into the given area of the buffer.
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// Whether this widget is focusable (receives keyboard input).
    fn focusable(&self) -> bool {
        false
    }

    /// Hint for the widget's preferred height given a width.
    /// Returns None if the widget has no preferred height.
    fn height_hint(&self, _width: u16) -> Option<u16> {
        None
    }

    /// Hint for the widget's preferred width.
    /// Returns None if the widget has no preferred width.
    fn width_hint(&self) -> Option<u16> {
        None
    }
}
