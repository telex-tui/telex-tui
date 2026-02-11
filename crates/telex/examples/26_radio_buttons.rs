//! Example 26: Radio Buttons
//!
//! Demonstrates radio button groups for mutually exclusive options.
//! Use arrow keys or j/k to navigate within a group.
//!
//! Run with: `cargo run -p telex-tui --example 26_radio_buttons`

use crossterm::event::KeyCode;
use telex::prelude::*;
use telex::Color;

telex::require_api!(0, 2);

fn main() {
    telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let show_help = state!(cx, || false);

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        // State for different radio groups
        let theme = state!(cx, || 0usize); // 0=Light, 1=Dark, 2=System
        let font_size = state!(cx, || 1usize); // 0=Small, 1=Medium, 2=Large
        let notification = state!(cx, || 0usize); // 0=All, 1=Important, 2=None

        let theme_options = vec!["Light", "Dark", "System"];
        let font_options = vec!["Small (12px)", "Medium (14px)", "Large (16px)"];
        let notification_options = vec!["All notifications", "Important only", "None"];

        View::vstack()
            .spacing(1)
            .child(
                // Header
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .child(View::styled_text("Radio Buttons Demo").bold().build())
                            .child(
                                View::styled_text(
                                    "Use Tab to switch groups, Up/Down or j/k to select",
                                )
                                .dim()
                                .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(
                // Main content - settings panel
                View::boxed()
                    .flex(1)
                    .border(true)
                    .padding(1)
                    .child(
                        View::hstack()
                            .spacing(4)
                            // Theme selection
                            .child(
                                View::vstack()
                                    .spacing(1)
                                    .child(
                                        View::styled_text("Theme")
                                            .bold()
                                            .color(Color::Cyan)
                                            .build(),
                                    )
                                    .child(
                                        View::radio_group()
                                            .options(theme_options)
                                            .selected(theme.get())
                                            .on_change(with!(theme => move |idx| {
                                                theme.set(idx);
                                            }))
                                            .build(),
                                    )
                                    .build(),
                            )
                            // Font size selection
                            .child(
                                View::vstack()
                                    .spacing(1)
                                    .child(
                                        View::styled_text("Font Size")
                                            .bold()
                                            .color(Color::Green)
                                            .build(),
                                    )
                                    .child(
                                        View::radio_group()
                                            .options(font_options)
                                            .selected(font_size.get())
                                            .on_change(with!(font_size => move |idx| {
                                                font_size.set(idx);
                                            }))
                                            .build(),
                                    )
                                    .build(),
                            )
                            // Notification selection
                            .child(
                                View::vstack()
                                    .spacing(1)
                                    .child(
                                        View::styled_text("Notifications")
                                            .bold()
                                            .color(Color::Yellow)
                                            .build(),
                                    )
                                    .child(
                                        View::radio_group()
                                            .options(notification_options)
                                            .selected(notification.get())
                                            .on_change(with!(notification => move |idx| {
                                                notification.set(idx);
                                            }))
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(
                // Current selections display
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .child(View::styled_text("Current Settings:").bold().build())
                            .child(View::text(format!(
                                "Theme: {} | Font: {} | Notifications: {}",
                                match theme.get() {
                                    0 => "Light",
                                    1 => "Dark",
                                    _ => "System",
                                },
                                match font_size.get() {
                                    0 => "Small",
                                    1 => "Medium",
                                    _ => "Large",
                                },
                                match notification.get() {
                                    0 => "All",
                                    1 => "Important",
                                    _ => "None",
                                }
                            )))
                            .build(),
                    )
                    .build(),
            )
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 26: Radio Buttons")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Radio groups for mutually exclusive options"))
                            .child(View::text("• Three independent groups"))
                            .child(View::text("• Current selection shown below"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::radio_group() creates groups"))
                            .child(View::text("• .options() takes Vec<&str>"))
                            .child(View::text("• .selected() binds to state (usize)"))
                            .child(View::text("• on_change receives new index"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Tab between groups"))
                            .child(View::text("• Up/Down or j/k to select"))
                            .child(View::text("• Watch current settings update"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 27_keyed_state: order-independent hooks"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
