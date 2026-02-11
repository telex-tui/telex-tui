//! Example 03: Theme Switcher
//!
//! Demonstrates theming support with a selectable list of themes.
//!
//! Run with: cargo run --example 03_theme_switcher

use crossterm::event::KeyCode;
use telex::prelude::*;
use telex::theme::{current_theme, set_theme, supports_true_color, terminal_name, Theme};

telex::require_api!(0, 1);

fn main() {
    telex::run(App).unwrap();
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let selected = state!(cx, || 0usize);
        let show_help = state!(cx, || false);

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        let theme_names = vec![
            "Dark".to_string(),
            "Light".to_string(),
            "Nord".to_string(),
            "Monokai".to_string(),
            "Catppuccin Mocha".to_string(),
            "Catppuccin Latte".to_string(),
            "Dracula".to_string(),
            "Gruvbox Dark".to_string(),
            "Solarized Dark".to_string(),
            "Rosé Pine".to_string(),
            "Tokyo Night".to_string(),
            "HaX0R Blue".to_string(),
            "HaX0R Green".to_string(),
            "HaX0R Red".to_string(),
        ];

        // Apply theme when selection changes
        let on_select = with!(selected => move |idx: usize| {
            selected.set(idx);
            let theme = match idx {
                0 => Theme::dark(),
                1 => Theme::light(),
                2 => Theme::nord(),
                3 => Theme::monokai(),
                4 => Theme::catppuccin_mocha(),
                5 => Theme::catppuccin_latte(),
                6 => Theme::dracula(),
                7 => Theme::gruvbox_dark(),
                8 => Theme::solarized_dark(),
                9 => Theme::rose_pine(),
                10 => Theme::tokyo_night(),
                11 => Theme::hax0r_blue(),
                12 => Theme::hax0r_green(),
                _ => Theme::hax0r_red(),
            };
            set_theme(theme);
        });

        let theme = current_theme();
        let true_color = supports_true_color();

        let mut stack = View::vstack();

        // Show warning if true color isn't supported
        if !true_color {
            let term = terminal_name().unwrap_or_else(|| "Unknown".to_string());
            stack = stack
                .child(
                    View::styled_text(format!("Warning: {} doesn't support true color", term))
                        .color(theme.warning)
                        .bold()
                        .build(),
                )
                .child(
                    View::styled_text("Only 'Dark' and 'Light' themes will display correctly")
                        .color(theme.muted)
                        .build(),
                )
                .child(View::gap(1));
        }

        stack
            .child(
                View::styled_text("Theme Switcher")
                    .color(theme.primary)
                    .bold()
                    .build(),
            )
            .child(
                View::styled_text("Select a theme from the list")
                    .color(theme.muted)
                    .italic()
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::hstack()
                    .spacing(2)
                    .child(
                        View::boxed()
                            .border(true)
                            .min_width(25)
                            .child(
                                View::list()
                                    .items(theme_names)
                                    .selected(selected.get())
                                    .on_select(on_select)
                                    .build(),
                            )
                            .build(),
                    )
                    .child(
                        View::boxed()
                            .border(true)
                            .padding(1)
                            .child(
                                View::vstack()
                                    .child(View::styled_text("Preview").bold().build())
                                    .child(View::gap(1))
                                    .child(
                                        View::hstack()
                                            .child(
                                                View::styled_text("Primary")
                                                    .color(theme.primary)
                                                    .build(),
                                            )
                                            .child(View::text("  "))
                                            .child(
                                                View::styled_text("Secondary")
                                                    .color(theme.secondary)
                                                    .build(),
                                            )
                                            .build(),
                                    )
                                    .child(
                                        View::hstack()
                                            .child(
                                                View::styled_text("Muted")
                                                    .color(theme.muted)
                                                    .build(),
                                            )
                                            .child(View::text("  "))
                                            .child(
                                                View::styled_text("Success")
                                                    .color(theme.success)
                                                    .build(),
                                            )
                                            .build(),
                                    )
                                    .child(
                                        View::hstack()
                                            .child(
                                                View::styled_text("Warning")
                                                    .color(theme.warning)
                                                    .build(),
                                            )
                                            .child(View::text("  "))
                                            .child(
                                                View::styled_text("Error")
                                                    .color(theme.error)
                                                    .build(),
                                            )
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::styled_text("↑/↓ select • F1 help • Ctrl+Q quit")
                    .color(theme.muted)
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 03: Theme Switcher")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Built-in theme system with 14 themes"))
                            .child(View::text("• View::list() for selection UI"))
                            .child(View::text("• Live preview as you navigate"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• current_theme() gets active theme colors"))
                            .child(View::text("• set_theme() changes theme globally"))
                            .child(View::text(
                                "• Themes provide semantic colors (primary, error, etc.)",
                            ))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text(
                                "• Navigate with ↑/↓ to see themes change instantly",
                            ))
                            .child(View::text(
                                "• Notice the preview panel updates with theme colors",
                            ))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 04_timer: streaming data without interaction"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
