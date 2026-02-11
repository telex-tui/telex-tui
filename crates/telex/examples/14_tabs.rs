//! Example 14: Tabs
//!
//! Demonstrates the Tabs widget for tabbed interfaces.
//!
//! Run with: cargo run -p telex-tui --example 14_tabs

use crossterm::event::KeyCode;
use crossterm::style::Color;
use telex::prelude::*;
use telex::theme::{set_theme, Theme};

telex::require_api!(0, 2);

fn main() {
    set_theme(Theme::dark());
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

        let active_tab = state!(cx, || 0usize);

        // Settings state
        let notifications = state!(cx, || true);
        let dark_mode = state!(cx, || true);
        let auto_save = state!(cx, || true);

        let on_change = with!(active_tab => move |idx: usize| {
            active_tab.set(idx);
        });

        // Checkbox handlers
        let on_notifications = with!(notifications => move |checked: bool| {
            notifications.set(checked);
        });

        let on_dark_mode = with!(dark_mode => move |checked: bool| {
            dark_mode.set(checked);
            if checked {
                set_theme(Theme::dark());
            } else {
                set_theme(Theme::light());
            }
        });

        let on_auto_save = with!(auto_save => move |checked: bool| {
            auto_save.set(checked);
        });

        View::vstack()
            .child(
                View::styled_text("Tabbed Interface Demo")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(
                View::boxed()
                    .flex(1)
                    .child(
                        View::tabs()
                            .tab(
                                "Overview",
                                View::vstack()
                                    .child(View::styled_text("Welcome!").bold().build())
                                    .child(View::text(
                                        "\nThis is the Overview tab.\n\n\
                                         Use the keyboard to switch tabs:\n\
                                         - Left/Right arrows\n\
                                         - [ and ] keys\n\
                                         - Number keys 1-3",
                                    ))
                                    .build(),
                            )
                            .tab(
                                "Settings",
                                View::vstack()
                                    .child(View::styled_text("Settings").bold().build())
                                    .child(View::text(""))
                                    .child(
                                        View::checkbox()
                                            .label("Enable notifications")
                                            .checked(notifications.get())
                                            .on_toggle(on_notifications)
                                            .build(),
                                    )
                                    .child(
                                        View::checkbox()
                                            .label("Dark mode")
                                            .checked(dark_mode.get())
                                            .on_toggle(on_dark_mode)
                                            .build(),
                                    )
                                    .child(
                                        View::checkbox()
                                            .label("Auto-save")
                                            .checked(auto_save.get())
                                            .on_toggle(on_auto_save)
                                            .build(),
                                    )
                                    .build(),
                            )
                            .tab(
                                "About",
                                View::vstack()
                                    .child(View::styled_text("About").bold().build())
                                    .child(View::text(""))
                                    .child(View::text("Telex TUI Framework"))
                                    .child(View::text("Version: 0.2.1"))
                                    .child(View::text(""))
                                    .child(
                                        View::styled_text("A React-style TUI framework for Rust")
                                            .dim()
                                            .build(),
                                    )
                                    .build(),
                            )
                            .active(active_tab.get())
                            .on_change(on_change)
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::styled_text("←→ or []: switch tabs | F1 help | Ctrl+Q: quit")
                    .dim()
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 14: Tabs")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Tabbed interface with three tabs"))
                            .child(View::text("• Settings tab with checkboxes"))
                            .child(View::text("• Keyboard navigation between tabs"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::tabs() creates tabbed container"))
                            .child(View::text("• .tab(\"Title\", content) adds each tab"))
                            .child(View::text("• .active() and .on_change() for state"))
                            .child(View::text("• Arrow keys, [ ], or 1-3 switch tabs"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Switch tabs with arrow keys"))
                            .child(View::text("• Toggle checkboxes in Settings"))
                            .child(View::text("• Try [ and ] keys for tab switching"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 15_markdown: markdown rendering"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
