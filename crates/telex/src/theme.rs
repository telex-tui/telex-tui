//! Theming support for Telex applications.
//!
//! Provides customizable color schemes for UI elements.
//!
//! # Example
//! ```rust,ignore
//! use telex::theme::Theme;
//!
//! let theme = Theme::dark();
//! // or customize:
//! let custom = Theme::default()
//!     .with_primary(Color::Cyan)
//!     .with_background(Color::Black);
//! ```

use crossterm::style::Color;

/// Color scheme for the application.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary accent color (focused elements, highlights)
    pub primary: Color,
    /// Secondary accent color
    pub secondary: Color,
    /// Background color
    pub background: Color,
    /// Foreground/text color
    pub foreground: Color,
    /// Muted/dimmed text color
    pub muted: Color,
    /// Error color
    pub error: Color,
    /// Success color
    pub success: Color,
    /// Warning color
    pub warning: Color,
    /// Border color
    pub border: Color,
    /// Focused border color
    pub border_focused: Color,
    /// Button background
    pub button_bg: Color,
    /// Button foreground
    pub button_fg: Color,
    /// Button focused background
    pub button_focused_bg: Color,
    /// Button focused foreground
    pub button_focused_fg: Color,
    /// Selection/highlight background
    pub selection_bg: Color,
    /// Selection/highlight foreground
    pub selection_fg: Color,
    /// Input field background
    pub input_bg: Color,
    /// Input field foreground
    pub input_fg: Color,
    /// Placeholder text color
    pub placeholder: Color,
    /// Cursor color (foreground of cursor character/block)
    pub cursor: Color,
    /// Cursor text color (background behind cursor, text color when on a character)
    pub cursor_text: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Create a dark theme (default).
    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Magenta,
            background: Color::Reset,
            foreground: Color::Reset,
            muted: Color::DarkGrey,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            border: Color::Reset,
            border_focused: Color::Cyan,
            button_bg: Color::Reset,
            button_fg: Color::Grey,
            button_focused_bg: Color::White,
            button_focused_fg: Color::Black,
            selection_bg: Color::Grey,  // Softer than White
            selection_fg: Color::Black,
            input_bg: Color::Reset,
            input_fg: Color::Reset,
            placeholder: Color::DarkGrey,
            // Cursor must contrast with selection_bg (the "general highlight" when widget has focus).
            // Since selection_bg=Grey, cursor must NOT be Grey or it disappears.
            cursor: Color::Black,
            cursor_text: Color::Grey,
        }
    }

    /// Create a light theme.
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Magenta,
            background: Color::White,
            foreground: Color::Black,
            muted: Color::DarkGrey,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            border: Color::DarkGrey,
            border_focused: Color::Blue,
            button_bg: Color::DarkGrey,
            button_fg: Color::White,
            button_focused_bg: Color::Blue,
            button_focused_fg: Color::White,
            selection_bg: Color::Blue,
            selection_fg: Color::White,
            input_bg: Color::White,
            input_fg: Color::Black,
            placeholder: Color::DarkGrey,
            cursor: Color::Black,
            cursor_text: Color::White,
        }
    }

    /// Create a nord-inspired theme.
    pub fn nord() -> Self {
        Self {
            primary: Color::Rgb {
                r: 136,
                g: 192,
                b: 208,
            }, // Nord8 - frost
            secondary: Color::Rgb {
                r: 180,
                g: 142,
                b: 173,
            }, // Nord15 - aurora
            background: Color::Rgb {
                r: 46,
                g: 52,
                b: 64,
            }, // Nord0 - polar night
            foreground: Color::Rgb {
                r: 236,
                g: 239,
                b: 244,
            }, // Nord6 - snow storm
            muted: Color::Rgb {
                r: 76,
                g: 86,
                b: 106,
            }, // Nord3
            error: Color::Rgb {
                r: 191,
                g: 97,
                b: 106,
            }, // Nord11
            success: Color::Rgb {
                r: 163,
                g: 190,
                b: 140,
            }, // Nord14
            warning: Color::Rgb {
                r: 235,
                g: 203,
                b: 139,
            }, // Nord13
            border: Color::Rgb {
                r: 76,
                g: 86,
                b: 106,
            }, // Nord3
            border_focused: Color::Rgb {
                r: 136,
                g: 192,
                b: 208,
            }, // Nord8
            button_bg: Color::Rgb {
                r: 59,
                g: 66,
                b: 82,
            }, // Nord1
            button_fg: Color::Rgb {
                r: 236,
                g: 239,
                b: 244,
            }, // Nord6
            button_focused_bg: Color::Rgb {
                r: 136,
                g: 192,
                b: 208,
            }, // Nord8
            button_focused_fg: Color::Rgb {
                r: 46,
                g: 52,
                b: 64,
            }, // Nord0
            selection_bg: Color::Rgb {
                r: 136,
                g: 192,
                b: 208,
            }, // Nord8
            selection_fg: Color::Rgb {
                r: 46,
                g: 52,
                b: 64,
            }, // Nord0
            input_bg: Color::Rgb {
                r: 59,
                g: 66,
                b: 82,
            }, // Nord1
            input_fg: Color::Rgb {
                r: 236,
                g: 239,
                b: 244,
            }, // Nord6
            placeholder: Color::Rgb {
                r: 76,
                g: 86,
                b: 106,
            }, // Nord3
            cursor: Color::Rgb {
                r: 236,
                g: 239,
                b: 244,
            }, // Nord6 - snow storm (foreground)
            cursor_text: Color::Rgb {
                r: 46,
                g: 52,
                b: 64,
            }, // Nord0 - polar night
        }
    }

    /// Create a monokai-inspired theme.
    pub fn monokai() -> Self {
        Self {
            primary: Color::Rgb {
                r: 102,
                g: 217,
                b: 239,
            }, // Cyan
            secondary: Color::Rgb {
                r: 174,
                g: 129,
                b: 255,
            }, // Purple
            background: Color::Rgb {
                r: 39,
                g: 40,
                b: 34,
            }, // Dark bg
            foreground: Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            }, // Light fg
            muted: Color::Rgb {
                r: 117,
                g: 113,
                b: 94,
            }, // Comment grey
            error: Color::Rgb {
                r: 249,
                g: 38,
                b: 114,
            }, // Pink/red
            success: Color::Rgb {
                r: 166,
                g: 226,
                b: 46,
            }, // Green
            warning: Color::Rgb {
                r: 253,
                g: 151,
                b: 31,
            }, // Orange
            border: Color::Rgb {
                r: 117,
                g: 113,
                b: 94,
            },
            border_focused: Color::Rgb {
                r: 102,
                g: 217,
                b: 239,
            },
            button_bg: Color::Rgb {
                r: 73,
                g: 72,
                b: 62,
            },
            button_fg: Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            },
            button_focused_bg: Color::Rgb {
                r: 102,
                g: 217,
                b: 239,
            },
            button_focused_fg: Color::Rgb {
                r: 39,
                g: 40,
                b: 34,
            },
            selection_bg: Color::Rgb {
                r: 102,
                g: 217,
                b: 239,
            },
            selection_fg: Color::Rgb {
                r: 39,
                g: 40,
                b: 34,
            },
            input_bg: Color::Rgb {
                r: 73,
                g: 72,
                b: 62,
            },
            input_fg: Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            },
            placeholder: Color::Rgb {
                r: 117,
                g: 113,
                b: 94,
            },
            cursor: Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            }, // Light fg (foreground)
            cursor_text: Color::Rgb {
                r: 39,
                g: 40,
                b: 34,
            }, // Dark bg
        }
    }

    /// Create a Catppuccin Mocha theme (dark).
    pub fn catppuccin_mocha() -> Self {
        Self {
            primary: Color::Rgb {
                r: 137,
                g: 180,
                b: 250,
            }, // Blue
            secondary: Color::Rgb {
                r: 203,
                g: 166,
                b: 247,
            }, // Mauve
            background: Color::Rgb {
                r: 30,
                g: 30,
                b: 46,
            }, // Base
            foreground: Color::Rgb {
                r: 205,
                g: 214,
                b: 244,
            }, // Text
            muted: Color::Rgb {
                r: 108,
                g: 112,
                b: 134,
            }, // Overlay0
            error: Color::Rgb {
                r: 243,
                g: 139,
                b: 168,
            }, // Red
            success: Color::Rgb {
                r: 166,
                g: 227,
                b: 161,
            }, // Green
            warning: Color::Rgb {
                r: 249,
                g: 226,
                b: 175,
            }, // Yellow
            border: Color::Rgb {
                r: 69,
                g: 71,
                b: 90,
            }, // Surface1
            border_focused: Color::Rgb {
                r: 137,
                g: 180,
                b: 250,
            }, // Blue
            button_bg: Color::Rgb {
                r: 49,
                g: 50,
                b: 68,
            }, // Surface0
            button_fg: Color::Rgb {
                r: 205,
                g: 214,
                b: 244,
            }, // Text
            button_focused_bg: Color::Rgb {
                r: 137,
                g: 180,
                b: 250,
            }, // Blue
            button_focused_fg: Color::Rgb {
                r: 30,
                g: 30,
                b: 46,
            }, // Base
            selection_bg: Color::Rgb {
                r: 137,
                g: 180,
                b: 250,
            }, // Blue
            selection_fg: Color::Rgb {
                r: 30,
                g: 30,
                b: 46,
            }, // Base
            input_bg: Color::Rgb {
                r: 49,
                g: 50,
                b: 68,
            }, // Surface0
            input_fg: Color::Rgb {
                r: 205,
                g: 214,
                b: 244,
            }, // Text
            placeholder: Color::Rgb {
                r: 108,
                g: 112,
                b: 134,
            }, // Overlay0
            cursor: Color::Rgb {
                r: 205,
                g: 214,
                b: 244,
            }, // Text (foreground)
            cursor_text: Color::Rgb {
                r: 30,
                g: 30,
                b: 46,
            }, // Base
        }
    }

    /// Create a Catppuccin Latte theme (light).
    pub fn catppuccin_latte() -> Self {
        Self {
            primary: Color::Rgb {
                r: 30,
                g: 102,
                b: 245,
            }, // Blue
            secondary: Color::Rgb {
                r: 136,
                g: 57,
                b: 239,
            }, // Mauve
            background: Color::Rgb {
                r: 239,
                g: 241,
                b: 245,
            }, // Base
            foreground: Color::Rgb {
                r: 76,
                g: 79,
                b: 105,
            }, // Text
            muted: Color::Rgb {
                r: 156,
                g: 160,
                b: 176,
            }, // Overlay0
            error: Color::Rgb {
                r: 210,
                g: 15,
                b: 57,
            }, // Red
            success: Color::Rgb {
                r: 64,
                g: 160,
                b: 43,
            }, // Green
            warning: Color::Rgb {
                r: 223,
                g: 142,
                b: 29,
            }, // Yellow
            border: Color::Rgb {
                r: 188,
                g: 192,
                b: 204,
            }, // Surface1
            border_focused: Color::Rgb {
                r: 30,
                g: 102,
                b: 245,
            }, // Blue
            button_bg: Color::Rgb {
                r: 204,
                g: 208,
                b: 218,
            }, // Surface0
            button_fg: Color::Rgb {
                r: 76,
                g: 79,
                b: 105,
            }, // Text
            button_focused_bg: Color::Rgb {
                r: 30,
                g: 102,
                b: 245,
            }, // Blue
            button_focused_fg: Color::Rgb {
                r: 239,
                g: 241,
                b: 245,
            }, // Base
            selection_bg: Color::Rgb {
                r: 30,
                g: 102,
                b: 245,
            }, // Blue
            selection_fg: Color::Rgb {
                r: 239,
                g: 241,
                b: 245,
            }, // Base
            input_bg: Color::Rgb {
                r: 204,
                g: 208,
                b: 218,
            }, // Surface0
            input_fg: Color::Rgb {
                r: 76,
                g: 79,
                b: 105,
            }, // Text
            placeholder: Color::Rgb {
                r: 156,
                g: 160,
                b: 176,
            }, // Overlay0
            cursor: Color::Rgb {
                r: 76,
                g: 79,
                b: 105,
            }, // Text (foreground)
            cursor_text: Color::Rgb {
                r: 239,
                g: 241,
                b: 245,
            }, // Base
        }
    }

    /// Create a Dracula theme.
    pub fn dracula() -> Self {
        Self {
            primary: Color::Rgb {
                r: 139,
                g: 233,
                b: 253,
            }, // Cyan
            secondary: Color::Rgb {
                r: 189,
                g: 147,
                b: 249,
            }, // Purple
            background: Color::Rgb {
                r: 40,
                g: 42,
                b: 54,
            }, // Background
            foreground: Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            }, // Foreground
            muted: Color::Rgb {
                r: 98,
                g: 114,
                b: 164,
            }, // Comment
            error: Color::Rgb {
                r: 255,
                g: 85,
                b: 85,
            }, // Red
            success: Color::Rgb {
                r: 80,
                g: 250,
                b: 123,
            }, // Green
            warning: Color::Rgb {
                r: 255,
                g: 184,
                b: 108,
            }, // Orange
            border: Color::Rgb {
                r: 68,
                g: 71,
                b: 90,
            }, // Selection
            border_focused: Color::Rgb {
                r: 139,
                g: 233,
                b: 253,
            }, // Cyan
            button_bg: Color::Rgb {
                r: 68,
                g: 71,
                b: 90,
            }, // Selection
            button_fg: Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            }, // Foreground
            button_focused_bg: Color::Rgb {
                r: 139,
                g: 233,
                b: 253,
            }, // Cyan
            button_focused_fg: Color::Rgb {
                r: 40,
                g: 42,
                b: 54,
            }, // Background
            selection_bg: Color::Rgb {
                r: 68,
                g: 71,
                b: 90,
            }, // Selection
            selection_fg: Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            }, // Foreground
            input_bg: Color::Rgb {
                r: 68,
                g: 71,
                b: 90,
            }, // Selection
            input_fg: Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            }, // Foreground
            placeholder: Color::Rgb {
                r: 98,
                g: 114,
                b: 164,
            }, // Comment
            cursor: Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            }, // Foreground
            cursor_text: Color::Rgb {
                r: 40,
                g: 42,
                b: 54,
            }, // Background
        }
    }

    /// Create a Gruvbox Dark theme.
    pub fn gruvbox_dark() -> Self {
        Self {
            primary: Color::Rgb {
                r: 131,
                g: 165,
                b: 152,
            }, // Blue
            secondary: Color::Rgb {
                r: 211,
                g: 134,
                b: 155,
            }, // Purple
            background: Color::Rgb {
                r: 40,
                g: 40,
                b: 40,
            }, // bg0
            foreground: Color::Rgb {
                r: 235,
                g: 219,
                b: 178,
            }, // fg (light1)
            muted: Color::Rgb {
                r: 146,
                g: 131,
                b: 116,
            }, // gray
            error: Color::Rgb {
                r: 251,
                g: 73,
                b: 52,
            }, // Red bright
            success: Color::Rgb {
                r: 184,
                g: 187,
                b: 38,
            }, // Green bright
            warning: Color::Rgb {
                r: 250,
                g: 189,
                b: 47,
            }, // Yellow bright
            border: Color::Rgb {
                r: 80,
                g: 73,
                b: 69,
            }, // bg2
            border_focused: Color::Rgb {
                r: 131,
                g: 165,
                b: 152,
            }, // Blue
            button_bg: Color::Rgb {
                r: 60,
                g: 56,
                b: 54,
            }, // bg1
            button_fg: Color::Rgb {
                r: 235,
                g: 219,
                b: 178,
            }, // fg
            button_focused_bg: Color::Rgb {
                r: 131,
                g: 165,
                b: 152,
            }, // Blue
            button_focused_fg: Color::Rgb {
                r: 40,
                g: 40,
                b: 40,
            }, // bg0
            selection_bg: Color::Rgb {
                r: 131,
                g: 165,
                b: 152,
            }, // Blue
            selection_fg: Color::Rgb {
                r: 40,
                g: 40,
                b: 40,
            }, // bg0
            input_bg: Color::Rgb {
                r: 60,
                g: 56,
                b: 54,
            }, // bg1
            input_fg: Color::Rgb {
                r: 235,
                g: 219,
                b: 178,
            }, // fg
            placeholder: Color::Rgb {
                r: 146,
                g: 131,
                b: 116,
            }, // gray
            cursor: Color::Rgb {
                r: 235,
                g: 219,
                b: 178,
            }, // fg (foreground)
            cursor_text: Color::Rgb {
                r: 40,
                g: 40,
                b: 40,
            }, // bg0
        }
    }

    /// Create a Solarized Dark theme.
    pub fn solarized_dark() -> Self {
        Self {
            primary: Color::Rgb {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            secondary: Color::Rgb {
                r: 108,
                g: 113,
                b: 196,
            }, // Violet
            background: Color::Rgb { r: 0, g: 43, b: 54 }, // base03
            foreground: Color::Rgb {
                r: 131,
                g: 148,
                b: 150,
            }, // base0
            muted: Color::Rgb {
                r: 88,
                g: 110,
                b: 117,
            }, // base01
            error: Color::Rgb {
                r: 220,
                g: 50,
                b: 47,
            }, // Red
            success: Color::Rgb {
                r: 133,
                g: 153,
                b: 0,
            }, // Green
            warning: Color::Rgb {
                r: 181,
                g: 137,
                b: 0,
            }, // Yellow
            border: Color::Rgb { r: 7, g: 54, b: 66 },     // base02
            border_focused: Color::Rgb {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            button_bg: Color::Rgb { r: 7, g: 54, b: 66 },  // base02
            button_fg: Color::Rgb {
                r: 131,
                g: 148,
                b: 150,
            }, // base0
            button_focused_bg: Color::Rgb {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            button_focused_fg: Color::Rgb {
                r: 253,
                g: 246,
                b: 227,
            }, // base3
            selection_bg: Color::Rgb {
                r: 38,
                g: 139,
                b: 210,
            }, // Blue
            selection_fg: Color::Rgb {
                r: 253,
                g: 246,
                b: 227,
            }, // base3
            input_bg: Color::Rgb { r: 7, g: 54, b: 66 },   // base02
            input_fg: Color::Rgb {
                r: 131,
                g: 148,
                b: 150,
            }, // base0
            placeholder: Color::Rgb {
                r: 88,
                g: 110,
                b: 117,
            }, // base01
            cursor: Color::Rgb {
                r: 131,
                g: 148,
                b: 150,
            }, // base0 (foreground)
            cursor_text: Color::Rgb {
                r: 0,
                g: 43,
                b: 54,
            }, // base03
        }
    }

    /// Create a Rosé Pine theme (dark).
    pub fn rose_pine() -> Self {
        Self {
            primary: Color::Rgb {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            secondary: Color::Rgb {
                r: 196,
                g: 167,
                b: 231,
            }, // Iris
            background: Color::Rgb {
                r: 25,
                g: 23,
                b: 36,
            }, // Base
            foreground: Color::Rgb {
                r: 224,
                g: 222,
                b: 244,
            }, // Text
            muted: Color::Rgb {
                r: 110,
                g: 106,
                b: 134,
            }, // Muted
            error: Color::Rgb {
                r: 235,
                g: 111,
                b: 146,
            }, // Love
            success: Color::Rgb {
                r: 49,
                g: 116,
                b: 143,
            }, // Pine
            warning: Color::Rgb {
                r: 246,
                g: 193,
                b: 119,
            }, // Gold
            border: Color::Rgb {
                r: 31,
                g: 29,
                b: 46,
            }, // Surface
            border_focused: Color::Rgb {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            button_bg: Color::Rgb {
                r: 31,
                g: 29,
                b: 46,
            }, // Surface
            button_fg: Color::Rgb {
                r: 224,
                g: 222,
                b: 244,
            }, // Text
            button_focused_bg: Color::Rgb {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            button_focused_fg: Color::Rgb {
                r: 25,
                g: 23,
                b: 36,
            }, // Base
            selection_bg: Color::Rgb {
                r: 156,
                g: 207,
                b: 216,
            }, // Foam
            selection_fg: Color::Rgb {
                r: 25,
                g: 23,
                b: 36,
            }, // Base
            input_bg: Color::Rgb {
                r: 31,
                g: 29,
                b: 46,
            }, // Surface
            input_fg: Color::Rgb {
                r: 224,
                g: 222,
                b: 244,
            }, // Text
            placeholder: Color::Rgb {
                r: 110,
                g: 106,
                b: 134,
            }, // Muted
            cursor: Color::Rgb {
                r: 224,
                g: 222,
                b: 244,
            }, // Text (foreground)
            cursor_text: Color::Rgb {
                r: 25,
                g: 23,
                b: 36,
            }, // Base
        }
    }

    /// Create a HaX0R Blue theme (monochrome cyan/blue hacker style).
    pub fn hax0r_blue() -> Self {
        Self {
            primary: Color::Rgb {
                r: 16,
                g: 182,
                b: 255,
            }, // Bright cyan
            secondary: Color::Rgb {
                r: 0,
                g: 179,
                b: 247,
            }, // Cyan
            background: Color::Rgb { r: 1, g: 5, b: 21 }, // Dark navy
            foreground: Color::Rgb {
                r: 17,
                g: 183,
                b: 255,
            }, // Cyan text
            muted: Color::Rgb {
                r: 72,
                g: 65,
                b: 87,
            }, // Muted purple-gray
            error: Color::Rgb {
                r: 16,
                g: 182,
                b: 255,
            }, // Cyan (monochrome)
            success: Color::Rgb {
                r: 16,
                g: 182,
                b: 255,
            }, // Cyan (monochrome)
            warning: Color::Rgb {
                r: 16,
                g: 182,
                b: 255,
            }, // Cyan (monochrome)
            border: Color::Rgb {
                r: 16,
                g: 182,
                b: 255,
            }, // Cyan
            border_focused: Color::Rgb {
                r: 250,
                g: 250,
                b: 250,
            }, // White
            button_bg: Color::Rgb { r: 1, g: 9, b: 33 },  // Slightly lighter bg
            button_fg: Color::Rgb {
                r: 16,
                g: 182,
                b: 255,
            }, // Cyan
            button_focused_bg: Color::Rgb {
                r: 16,
                g: 182,
                b: 255,
            }, // Cyan
            button_focused_fg: Color::Rgb { r: 1, g: 5, b: 21 }, // Dark bg
            selection_bg: Color::Rgb {
                r: 193,
                g: 228,
                b: 255,
            }, // Light cyan
            selection_fg: Color::Rgb { r: 1, g: 5, b: 21 }, // Dark bg
            input_bg: Color::Rgb { r: 1, g: 9, b: 33 },   // Slightly lighter bg
            input_fg: Color::Rgb {
                r: 16,
                g: 182,
                b: 255,
            }, // Cyan
            placeholder: Color::Rgb {
                r: 72,
                g: 65,
                b: 87,
            }, // Muted
            cursor: Color::Rgb {
                r: 17,
                g: 183,
                b: 255,
            }, // Foreground cyan
            cursor_text: Color::Rgb { r: 1, g: 5, b: 21 }, // Dark navy
        }
    }

    /// Create a HaX0R Green theme (monochrome green hacker style).
    pub fn hax0r_green() -> Self {
        Self {
            primary: Color::Rgb {
                r: 21,
                g: 208,
                b: 13,
            }, // Bright green
            secondary: Color::Rgb {
                r: 25,
                g: 226,
                b: 14,
            }, // Green
            background: Color::Rgb { r: 2, g: 15, b: 1 }, // Dark green-black
            foreground: Color::Rgb {
                r: 22,
                g: 177,
                b: 14,
            }, // Green text
            muted: Color::Rgb {
                r: 51,
                g: 72,
                b: 67,
            }, // Muted green-gray
            error: Color::Rgb {
                r: 21,
                g: 208,
                b: 13,
            }, // Green (monochrome)
            success: Color::Rgb {
                r: 21,
                g: 208,
                b: 13,
            }, // Green (monochrome)
            warning: Color::Rgb {
                r: 21,
                g: 208,
                b: 13,
            }, // Green (monochrome)
            border: Color::Rgb {
                r: 21,
                g: 208,
                b: 13,
            }, // Green
            border_focused: Color::Rgb {
                r: 250,
                g: 250,
                b: 250,
            }, // White
            button_bg: Color::Rgb { r: 0, g: 31, b: 11 }, // Slightly lighter bg
            button_fg: Color::Rgb {
                r: 21,
                g: 208,
                b: 13,
            }, // Green
            button_focused_bg: Color::Rgb {
                r: 21,
                g: 208,
                b: 13,
            }, // Green
            button_focused_fg: Color::Rgb { r: 2, g: 15, b: 1 }, // Dark bg
            selection_bg: Color::Rgb {
                r: 212,
                g: 255,
                b: 193,
            }, // Light green
            selection_fg: Color::Rgb { r: 2, g: 15, b: 1 }, // Dark bg
            input_bg: Color::Rgb { r: 0, g: 31, b: 11 },  // Slightly lighter bg
            input_fg: Color::Rgb {
                r: 21,
                g: 208,
                b: 13,
            }, // Green
            placeholder: Color::Rgb {
                r: 51,
                g: 72,
                b: 67,
            }, // Muted
            cursor: Color::Rgb {
                r: 22,
                g: 177,
                b: 14,
            }, // Foreground green
            cursor_text: Color::Rgb { r: 2, g: 15, b: 1 }, // Dark bg
        }
    }

    /// Create a HaX0R Red theme (monochrome red hacker style).
    pub fn hax0r_red() -> Self {
        Self {
            primary: Color::Rgb {
                r: 176,
                g: 13,
                b: 13,
            }, // Dark red
            secondary: Color::Rgb {
                r: 255,
                g: 17,
                b: 17,
            }, // Bright red
            background: Color::Rgb { r: 32, g: 1, b: 1 }, // Dark red-black
            foreground: Color::Rgb {
                r: 177,
                g: 14,
                b: 14,
            }, // Red text
            muted: Color::Rgb {
                r: 85,
                g: 64,
                b: 64,
            }, // Muted red-gray
            error: Color::Rgb {
                r: 255,
                g: 17,
                b: 17,
            }, // Bright red
            success: Color::Rgb {
                r: 176,
                g: 13,
                b: 13,
            }, // Red (monochrome)
            warning: Color::Rgb {
                r: 176,
                g: 13,
                b: 13,
            }, // Red (monochrome)
            border: Color::Rgb {
                r: 176,
                g: 13,
                b: 13,
            }, // Red
            border_focused: Color::Rgb {
                r: 250,
                g: 250,
                b: 250,
            }, // White
            button_bg: Color::Rgb { r: 31, g: 0, b: 0 },  // Slightly lighter bg
            button_fg: Color::Rgb {
                r: 176,
                g: 13,
                b: 13,
            }, // Red
            button_focused_bg: Color::Rgb {
                r: 176,
                g: 13,
                b: 13,
            }, // Red
            button_focused_fg: Color::Rgb { r: 32, g: 1, b: 1 }, // Dark bg
            selection_bg: Color::Rgb {
                r: 235,
                g: 193,
                b: 255,
            }, // Light pink
            selection_fg: Color::Rgb { r: 32, g: 1, b: 1 }, // Dark bg
            input_bg: Color::Rgb { r: 31, g: 0, b: 0 },   // Slightly lighter bg
            input_fg: Color::Rgb {
                r: 176,
                g: 13,
                b: 13,
            }, // Red
            placeholder: Color::Rgb {
                r: 85,
                g: 64,
                b: 64,
            }, // Muted
            cursor: Color::Rgb {
                r: 177,
                g: 14,
                b: 14,
            }, // Foreground red
            cursor_text: Color::Rgb { r: 32, g: 1, b: 1 }, // Dark bg
        }
    }

    /// Create a Tokyo Night theme (dark).
    pub fn tokyo_night() -> Self {
        Self {
            primary: Color::Rgb {
                r: 122,
                g: 162,
                b: 247,
            }, // Blue
            secondary: Color::Rgb {
                r: 187,
                g: 154,
                b: 247,
            }, // Magenta
            background: Color::Rgb {
                r: 26,
                g: 27,
                b: 38,
            }, // bg
            foreground: Color::Rgb {
                r: 192,
                g: 202,
                b: 245,
            }, // fg
            muted: Color::Rgb {
                r: 65,
                g: 72,
                b: 104,
            }, // Bright black
            error: Color::Rgb {
                r: 247,
                g: 118,
                b: 142,
            }, // Red
            success: Color::Rgb {
                r: 158,
                g: 206,
                b: 106,
            }, // Green
            warning: Color::Rgb {
                r: 224,
                g: 175,
                b: 104,
            }, // Yellow
            border: Color::Rgb {
                r: 41,
                g: 46,
                b: 66,
            }, // Surface
            border_focused: Color::Rgb {
                r: 122,
                g: 162,
                b: 247,
            }, // Blue
            button_bg: Color::Rgb {
                r: 41,
                g: 46,
                b: 66,
            }, // Surface
            button_fg: Color::Rgb {
                r: 192,
                g: 202,
                b: 245,
            }, // fg
            button_focused_bg: Color::Rgb {
                r: 122,
                g: 162,
                b: 247,
            }, // Blue
            button_focused_fg: Color::Rgb {
                r: 26,
                g: 27,
                b: 38,
            }, // bg
            selection_bg: Color::Rgb {
                r: 122,
                g: 162,
                b: 247,
            }, // Blue
            selection_fg: Color::Rgb {
                r: 26,
                g: 27,
                b: 38,
            }, // bg
            input_bg: Color::Rgb {
                r: 41,
                g: 46,
                b: 66,
            }, // Surface
            input_fg: Color::Rgb {
                r: 192,
                g: 202,
                b: 245,
            }, // fg
            placeholder: Color::Rgb {
                r: 65,
                g: 72,
                b: 104,
            }, // Bright black
            cursor: Color::Rgb {
                r: 192,
                g: 202,
                b: 245,
            }, // fg (foreground)
            cursor_text: Color::Rgb {
                r: 26,
                g: 27,
                b: 38,
            }, // bg
        }
    }

    // Builder methods for customization

    /// Set the primary accent color.
    pub fn with_primary(mut self, color: Color) -> Self {
        self.primary = color;
        self
    }

    /// Set the secondary accent color.
    pub fn with_secondary(mut self, color: Color) -> Self {
        self.secondary = color;
        self
    }

    /// Set the background color.
    pub fn with_background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set the foreground/text color.
    pub fn with_foreground(mut self, color: Color) -> Self {
        self.foreground = color;
        self
    }

    /// Set the error color.
    pub fn with_error(mut self, color: Color) -> Self {
        self.error = color;
        self
    }

    /// Set the success color.
    pub fn with_success(mut self, color: Color) -> Self {
        self.success = color;
        self
    }

    /// Set the warning color.
    pub fn with_warning(mut self, color: Color) -> Self {
        self.warning = color;
        self
    }
}

/// Global theme storage (thread-local for now).
use std::cell::RefCell;

thread_local! {
    static CURRENT_THEME: RefCell<Theme> = RefCell::new(Theme::default());
}

/// Set the current theme.
pub fn set_theme(theme: Theme) {
    CURRENT_THEME.with(|t| {
        *t.borrow_mut() = theme;
    });
}

/// Get the current theme.
pub fn current_theme() -> Theme {
    CURRENT_THEME.with(|t| t.borrow().clone())
}

/// Get a specific color from the current theme.
pub fn themed_color<F>(f: F) -> Color
where
    F: FnOnce(&Theme) -> Color,
{
    CURRENT_THEME.with(|t| f(&t.borrow()))
}

/// Check if the terminal supports true color (24-bit RGB).
///
/// Returns `true` if the terminal likely supports RGB colors.
/// Returns `false` if running in a terminal known not to support true color
/// (e.g., Apple Terminal.app).
pub fn supports_true_color() -> bool {
    use std::env;

    // Apple Terminal doesn't support true color
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        if term_program == "Apple_Terminal" {
            return false;
        }
    }

    // Check for explicit true color support
    if let Ok(colorterm) = env::var("COLORTERM") {
        if colorterm == "truecolor" || colorterm == "24bit" {
            return true;
        }
    }

    // Known true color terminals
    if let Ok(term) = env::var("TERM") {
        if term.contains("256color") || term.contains("truecolor") || term.contains("24bit") {
            // 256color doesn't guarantee true color, but many modern terminals
            // that set this also support true color
        }
    }

    // Check for specific terminal programs known to support true color
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        let true_color_terminals = [
            "iTerm.app",
            "Hyper",
            "vscode",
            "Ghostty",
            "WezTerm",
            "Alacritty",
            "kitty",
        ];
        if true_color_terminals
            .iter()
            .any(|t| term_program.contains(t))
        {
            return true;
        }
    }

    // Default: assume true color is supported (most modern terminals do)
    // User can check the warning if colors look wrong
    true
}

/// Get the name of the current terminal, if detectable.
pub fn terminal_name() -> Option<String> {
    std::env::var("TERM_PROGRAM").ok()
}
