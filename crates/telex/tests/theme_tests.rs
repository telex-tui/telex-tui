//! Tests for the Theme module.
//!
//! Verifies theme creation, customization, and global theme management.

use crossterm::style::Color;
use telex::theme::{current_theme, set_theme, supports_true_color, terminal_name, Theme};

// ============================================================
// Theme Creation Tests
// ============================================================

#[test]
fn test_theme_dark_creation() {
    let theme = Theme::dark();

    // Dark theme should have Cyan primary
    assert_eq!(theme.primary, Color::Cyan);
    // Background should be Reset (terminal default)
    assert_eq!(theme.background, Color::Reset);
}

#[test]
fn test_theme_light_creation() {
    let theme = Theme::light();

    // Light theme should have Blue primary
    assert_eq!(theme.primary, Color::Blue);
    // Light theme has White background
    assert_eq!(theme.background, Color::White);
    assert_eq!(theme.foreground, Color::Black);
}

#[test]
fn test_theme_default_is_dark() {
    let theme = Theme::default();
    let dark = Theme::dark();

    // Default should be dark theme
    assert_eq!(theme.primary, dark.primary);
    assert_eq!(theme.background, dark.background);
}

#[test]
fn test_theme_nord_creation() {
    let theme = Theme::nord();

    // Nord theme uses RGB colors
    match theme.primary {
        Color::Rgb { r, g, b } => {
            // Nord8 - frost cyan
            assert_eq!(r, 136);
            assert_eq!(g, 192);
            assert_eq!(b, 208);
        }
        _ => panic!("Nord primary should be RGB"),
    }
}

#[test]
fn test_theme_monokai_creation() {
    let theme = Theme::monokai();

    // Monokai has distinctive cyan primary
    match theme.primary {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 102);
            assert_eq!(g, 217);
            assert_eq!(b, 239);
        }
        _ => panic!("Monokai primary should be RGB"),
    }
}

#[test]
fn test_theme_catppuccin_mocha_creation() {
    let theme = Theme::catppuccin_mocha();

    // Catppuccin Mocha has specific blue
    match theme.primary {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 137);
            assert_eq!(g, 180);
            assert_eq!(b, 250);
        }
        _ => panic!("Catppuccin Mocha primary should be RGB"),
    }
}

#[test]
fn test_theme_catppuccin_latte_creation() {
    let theme = Theme::catppuccin_latte();

    // Catppuccin Latte is light theme
    match theme.background {
        Color::Rgb { r, g, b } => {
            // Light background
            assert!(r > 200 && g > 200 && b > 200);
        }
        _ => panic!("Catppuccin Latte background should be RGB"),
    }
}

#[test]
fn test_theme_dracula_creation() {
    let theme = Theme::dracula();

    // Dracula has cyan primary
    match theme.primary {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 139);
            assert_eq!(g, 233);
            assert_eq!(b, 253);
        }
        _ => panic!("Dracula primary should be RGB"),
    }
}

#[test]
fn test_theme_gruvbox_dark_creation() {
    let theme = Theme::gruvbox_dark();

    // Gruvbox has distinctive warm colors
    match theme.background {
        Color::Rgb { r, g, b } => {
            // bg0 - dark with slight warmth
            assert_eq!(r, 40);
            assert_eq!(g, 40);
            assert_eq!(b, 40);
        }
        _ => panic!("Gruvbox background should be RGB"),
    }
}

#[test]
fn test_theme_solarized_dark_creation() {
    let theme = Theme::solarized_dark();

    // Solarized has distinctive blue primary
    match theme.primary {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 38);
            assert_eq!(g, 139);
            assert_eq!(b, 210);
        }
        _ => panic!("Solarized primary should be RGB"),
    }
}

#[test]
fn test_theme_rose_pine_creation() {
    let theme = Theme::rose_pine();

    // Rose Pine has "foam" cyan primary
    match theme.primary {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 156);
            assert_eq!(g, 207);
            assert_eq!(b, 216);
        }
        _ => panic!("Rose Pine primary should be RGB"),
    }
}

#[test]
fn test_theme_hax0r_blue_creation() {
    let theme = Theme::hax0r_blue();

    // Hax0r Blue is monochrome cyan
    match theme.primary {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 16);
            assert_eq!(g, 182);
            assert_eq!(b, 255);
        }
        _ => panic!("Hax0r Blue primary should be RGB"),
    }
}

#[test]
fn test_theme_hax0r_green_creation() {
    let theme = Theme::hax0r_green();

    // Hax0r Green is monochrome green
    match theme.primary {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 21);
            assert_eq!(g, 208);
            assert_eq!(b, 13);
        }
        _ => panic!("Hax0r Green primary should be RGB"),
    }
}

#[test]
fn test_theme_hax0r_red_creation() {
    let theme = Theme::hax0r_red();

    // Hax0r Red is monochrome red
    match theme.primary {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 176);
            assert_eq!(g, 13);
            assert_eq!(b, 13);
        }
        _ => panic!("Hax0r Red primary should be RGB"),
    }
}

#[test]
fn test_theme_tokyo_night_creation() {
    let theme = Theme::tokyo_night();

    // Tokyo Night has distinctive blue
    match theme.primary {
        Color::Rgb { r, g, b } => {
            assert_eq!(r, 122);
            assert_eq!(g, 162);
            assert_eq!(b, 247);
        }
        _ => panic!("Tokyo Night primary should be RGB"),
    }
}

// ============================================================
// Theme Customization Tests
// ============================================================

#[test]
fn test_theme_with_primary() {
    let theme = Theme::dark().with_primary(Color::Red);
    assert_eq!(theme.primary, Color::Red);
}

#[test]
fn test_theme_with_secondary() {
    let theme = Theme::dark().with_secondary(Color::Green);
    assert_eq!(theme.secondary, Color::Green);
}

#[test]
fn test_theme_with_background() {
    let theme = Theme::dark().with_background(Color::Blue);
    assert_eq!(theme.background, Color::Blue);
}

#[test]
fn test_theme_with_foreground() {
    let theme = Theme::dark().with_foreground(Color::Yellow);
    assert_eq!(theme.foreground, Color::Yellow);
}

#[test]
fn test_theme_with_error() {
    let theme = Theme::dark().with_error(Color::Magenta);
    assert_eq!(theme.error, Color::Magenta);
}

#[test]
fn test_theme_with_success() {
    let theme = Theme::dark().with_success(Color::Cyan);
    assert_eq!(theme.success, Color::Cyan);
}

#[test]
fn test_theme_with_warning() {
    let theme = Theme::dark().with_warning(Color::White);
    assert_eq!(theme.warning, Color::White);
}

#[test]
fn test_theme_builder_chain() {
    let theme = Theme::dark()
        .with_primary(Color::Red)
        .with_secondary(Color::Green)
        .with_background(Color::Blue)
        .with_foreground(Color::Yellow);

    assert_eq!(theme.primary, Color::Red);
    assert_eq!(theme.secondary, Color::Green);
    assert_eq!(theme.background, Color::Blue);
    assert_eq!(theme.foreground, Color::Yellow);
}

// ============================================================
// Global Theme Management Tests
// ============================================================

#[test]
fn test_set_and_get_theme() {
    // Set a known theme
    set_theme(Theme::light());
    let current = current_theme();

    // Should be light theme
    assert_eq!(current.primary, Color::Blue);
    assert_eq!(current.background, Color::White);

    // Reset to dark for other tests
    set_theme(Theme::dark());
}

#[test]
fn test_current_theme_default() {
    // Current theme should have a valid primary color
    let theme = current_theme();
    // Just check it's not crashing and returns something
    let _ = theme.primary;
}

// ============================================================
// Theme Clone and Debug Tests
// ============================================================

#[test]
fn test_theme_clone() {
    let theme1 = Theme::nord();
    let theme2 = theme1.clone();

    assert_eq!(theme1.primary, theme2.primary);
    assert_eq!(theme1.background, theme2.background);
}

#[test]
fn test_theme_debug() {
    let theme = Theme::dark();
    let debug_str = format!("{:?}", theme);

    // Debug should include theme fields
    assert!(debug_str.contains("primary"));
    assert!(debug_str.contains("background"));
}

// ============================================================
// Theme All Fields Accessible Tests
// ============================================================

#[test]
fn test_theme_all_fields_accessible() {
    let theme = Theme::dark();

    // All fields should be accessible
    let _ = theme.primary;
    let _ = theme.secondary;
    let _ = theme.background;
    let _ = theme.foreground;
    let _ = theme.muted;
    let _ = theme.error;
    let _ = theme.success;
    let _ = theme.warning;
    let _ = theme.border;
    let _ = theme.border_focused;
    let _ = theme.button_bg;
    let _ = theme.button_fg;
    let _ = theme.button_focused_bg;
    let _ = theme.button_focused_fg;
    let _ = theme.selection_bg;
    let _ = theme.selection_fg;
    let _ = theme.input_bg;
    let _ = theme.input_fg;
    let _ = theme.placeholder;
}

// ============================================================
// Terminal Detection Tests
// ============================================================

#[test]
fn test_supports_true_color_returns_bool() {
    // Just verify it returns a bool without panicking
    let _supports = supports_true_color();
}

#[test]
fn test_terminal_name_returns_option() {
    // Just verify it returns Option without panicking
    let _name = terminal_name();
}

// ============================================================
// Theme Consistency Tests
// ============================================================

#[test]
fn test_all_themes_have_distinct_colors() {
    let themes: Vec<(&str, Theme)> = vec![
        ("dark", Theme::dark()),
        ("light", Theme::light()),
        ("nord", Theme::nord()),
        ("monokai", Theme::monokai()),
        ("catppuccin_mocha", Theme::catppuccin_mocha()),
        ("catppuccin_latte", Theme::catppuccin_latte()),
        ("dracula", Theme::dracula()),
        ("gruvbox_dark", Theme::gruvbox_dark()),
        ("solarized_dark", Theme::solarized_dark()),
        ("rose_pine", Theme::rose_pine()),
        ("hax0r_blue", Theme::hax0r_blue()),
        ("hax0r_green", Theme::hax0r_green()),
        ("hax0r_red", Theme::hax0r_red()),
        ("tokyo_night", Theme::tokyo_night()),
    ];

    // Each theme should have selection colors that contrast
    for (name, theme) in &themes {
        // Selection background and foreground should be different
        assert_ne!(
            theme.selection_bg, theme.selection_fg,
            "{} should have different selection bg/fg",
            name
        );

        // Button focused colors should differ from unfocused
        // (except for dark theme which uses Reset)
        if *name != "dark" {
            assert_ne!(
                theme.button_bg, theme.button_focused_bg,
                "{} should have different button states",
                name
            );
        }
    }
}

#[test]
fn test_light_themes_have_light_backgrounds() {
    let light_themes = vec![
        ("light", Theme::light()),
        ("catppuccin_latte", Theme::catppuccin_latte()),
    ];

    for (name, theme) in light_themes {
        match theme.background {
            Color::White => {}
            Color::Rgb { r, g, b } => {
                // Light backgrounds should have high RGB values
                assert!(
                    r > 150 && g > 150 && b > 150,
                    "{} should have light background, got RGB({}, {}, {})",
                    name,
                    r,
                    g,
                    b
                );
            }
            _ => panic!("{} should have White or light RGB background", name),
        }
    }
}

#[test]
fn test_dark_themes_have_dark_backgrounds() {
    let dark_themes = vec![
        ("nord", Theme::nord()),
        ("monokai", Theme::monokai()),
        ("catppuccin_mocha", Theme::catppuccin_mocha()),
        ("dracula", Theme::dracula()),
        ("gruvbox_dark", Theme::gruvbox_dark()),
        ("solarized_dark", Theme::solarized_dark()),
        ("rose_pine", Theme::rose_pine()),
        ("tokyo_night", Theme::tokyo_night()),
    ];

    for (name, theme) in dark_themes {
        match theme.background {
            Color::Reset | Color::Black => {}
            Color::Rgb { r, g, b } => {
                // Dark backgrounds should have low RGB values
                assert!(
                    r < 100 && g < 100 && b < 100,
                    "{} should have dark background, got RGB({}, {}, {})",
                    name,
                    r,
                    g,
                    b
                );
            }
            _ => panic!("{} should have Reset, Black, or dark RGB background", name),
        }
    }
}
