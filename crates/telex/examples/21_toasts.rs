//! Example 21: Toast Notifications
//!
//! Demonstrates ephemeral toast notifications that appear in the corner
//! of the screen and auto-dismiss after a duration.
//!
//! Run with: `cargo run -p telex-tui --example 21_toasts`

use crossterm::event::KeyCode;
use std::time::Duration;
use telex::prelude::*;
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

        // Create a toast queue with 3 second default duration
        let toasts = state!(cx, || ToastQueue::with_duration(Duration::from_secs(3)));
        let position = state!(cx, || ToastPosition::BottomRight);

        // Buttons to trigger different toast types
        let show_info = with!(toasts => move || {
            toasts.get().info("This is an informational message");
        });

        let show_success = with!(toasts => move || {
            toasts.get().success("Operation completed successfully!");
        });

        let show_warning = with!(toasts => move || {
            toasts.get().warning("Warning: This action cannot be undone");
        });

        let show_error = with!(toasts => move || {
            toasts.get().error("Error: Connection failed");
        });

        let show_long_error = with!(toasts => move || {
            toasts.get().error_long("Critical Error: Server not responding. Check your network.");
        });

        let clear_all = with!(toasts => move || {
            toasts.get().clear();
        });

        // Position cycling
        let cycle_position = with!(position => move || {
            let next = match position.get() {
                ToastPosition::TopRight => ToastPosition::TopLeft,
                ToastPosition::TopLeft => ToastPosition::BottomLeft,
                ToastPosition::BottomLeft => ToastPosition::BottomRight,
                ToastPosition::BottomRight => ToastPosition::TopRight,
            };
            position.set(next);
        });

        let position_name = match position.get() {
            ToastPosition::TopRight => "Top Right",
            ToastPosition::TopLeft => "Top Left",
            ToastPosition::BottomLeft => "Bottom Left",
            ToastPosition::BottomRight => "Bottom Right",
        };

        View::vstack()
            .spacing(1)
            .child(
                // Header
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .child(View::styled_text("Toast Notifications Demo").bold().build())
                            .child(
                                View::styled_text("Click buttons to show different toast types")
                                    .dim()
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(
                // Main content
                View::boxed()
                    .flex(1)
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .spacing(1)
                            .child(View::text("Toast Types:"))
                            .child(
                                View::hstack()
                                    .spacing(2)
                                    .child(View::button().label("Info").on_press(show_info).build())
                                    .child(
                                        View::button()
                                            .label("Success")
                                            .on_press(show_success)
                                            .build(),
                                    )
                                    .child(
                                        View::button()
                                            .label("Warning")
                                            .on_press(show_warning)
                                            .build(),
                                    )
                                    .child(
                                        View::button().label("Error").on_press(show_error).build(),
                                    )
                                    .build(),
                            )
                            .child(View::gap(1))
                            .child(View::text("Other Actions:"))
                            .child(
                                View::hstack()
                                    .spacing(2)
                                    .child(
                                        View::button()
                                            .label("Long Error")
                                            .on_press(show_long_error)
                                            .build(),
                                    )
                                    .child(
                                        View::button()
                                            .label("Clear All")
                                            .on_press(clear_all)
                                            .build(),
                                    )
                                    .build(),
                            )
                            .child(View::gap(1))
                            .child(View::text(format!("Position: {}", position_name)))
                            .child(
                                View::button()
                                    .label("Change Position")
                                    .on_press(cycle_position)
                                    .build(),
                            )
                            .child(View::spacer())
                            .child(
                                View::styled_text(format!("Active toasts: {}", toasts.get().len()))
                                    .color(Color::Yellow)
                                    .build(),
                            )
                            .child(
                                View::styled_text("Toasts auto-dismiss after 3 seconds")
                                    .dim()
                                    .build(),
                            )
                            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .child(
                // Toast container - renders the toast stack
                View::toast_container()
                    .from_queue(&toasts.get())
                    .position(position.get())
                    .max_visible(5)
                    .width(40)
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 21: Toasts")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Toast notifications in corner"))
                            .child(View::text("• Auto-dismiss after 3 seconds"))
                            .child(View::text(
                                "• Multiple toast types (info/success/warn/error)",
                            ))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• ToastQueue manages notifications"))
                            .child(View::text("• View::toast_container() renders them"))
                            .child(View::text("• .position() controls corner placement"))
                            .child(View::text("• .max_visible() limits shown toasts"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Click different toast type buttons"))
                            .child(View::text("• Change position to see toasts move"))
                            .child(View::text("• Spam buttons to stack toasts"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 22_forms: form validation"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
