//! Example 11: Checkbox
//!
//! Demonstrates the Checkbox widget for boolean toggles.
//!
//! Run with: cargo run -p telex-tui --example 11_checkbox

use crossterm::event::KeyCode;
use telex::prelude::*;
use telex::theme::{set_theme, Theme};
use telex::Color;

telex::require_api!(0, 1);

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

        // Settings state
        let dark_mode = state!(cx, || true);
        let notifications = state!(cx, || true);
        let auto_save = state!(cx, || false);
        let telemetry = state!(cx, || false);

        View::vstack()
            .spacing(1)
            .child(
                View::styled_text("Settings")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(
                View::styled_text("Use Tab to navigate, Enter/Space to toggle")
                    .dim()
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .spacing(1)
                            .child(View::styled_text("Appearance").bold().build())
                            .child(
                                View::checkbox()
                                    .checked(dark_mode.get())
                                    .label("Dark mode")
                                    .on_toggle(with!(dark_mode => move |checked| {
                                        dark_mode.set(checked);
                                        if checked {
                                            set_theme(Theme::dark());
                                        } else {
                                            set_theme(Theme::light());
                                        }
                                    }))
                                    .build(),
                            )
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
                            .spacing(1)
                            .child(View::styled_text("Behavior").bold().build())
                            .child(
                                View::checkbox()
                                    .checked(notifications.get())
                                    .label("Enable notifications")
                                    .on_toggle(with!(notifications => move |checked| {
                                        notifications.set(checked);
                                    }))
                                    .build(),
                            )
                            .child(
                                View::checkbox()
                                    .checked(auto_save.get())
                                    .label("Auto-save documents")
                                    .on_toggle(with!(auto_save => move |checked| {
                                        auto_save.set(checked);
                                    }))
                                    .build(),
                            )
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
                            .spacing(1)
                            .child(View::styled_text("Privacy").bold().build())
                            .child(
                                View::checkbox()
                                    .checked(telemetry.get())
                                    .label("Send anonymous usage data")
                                    .on_toggle(with!(telemetry => move |checked| {
                                        telemetry.set(checked);
                                    }))
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::hstack()
                    .spacing(2)
                    .child(View::text("Current settings:"))
                    .child(
                        View::styled_text(format!(
                            "dark={} notify={} autosave={} telemetry={}",
                            dark_mode.get(),
                            notifications.get(),
                            auto_save.get(),
                            telemetry.get()
                        ))
                        .color(Color::Yellow)
                        .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 11: Checkbox")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Checkbox widget for boolean toggles"))
                            .child(View::text("• Grouped settings in boxed sections"))
                            .child(View::text("• Dark mode toggle that changes theme live"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::checkbox() with checked state"))
                            .child(View::text("• on_toggle callback receives new value"))
                            .child(View::text("• set_theme() for live theme switching"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Toggle Dark mode to see theme change"))
                            .child(View::text("• Watch the status line update"))
                            .child(View::text("• Tab between checkboxes"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 12_text_area: multi-line text editing"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
