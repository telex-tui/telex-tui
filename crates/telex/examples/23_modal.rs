//! Example 23: Modal Dialogs
//!
//! Demonstrates modal overlay dialogs including confirm dialogs,
//! alert dialogs, and custom modal content.
//!
//! Run with: `cargo run -p telex-tui --example 23_modal`

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

        // Modal visibility states
        let show_confirm = state!(cx, || false);
        let show_alert = state!(cx, || false);
        let show_custom = state!(cx, || false);

        // App state that modals can modify
        let deleted_count = state!(cx, || 0);
        let custom_input = state!(cx, String::new);
        let last_action = state!(cx, || "No action yet".to_string());

        // Handlers
        let open_confirm = with!(show_confirm => move || {
            show_confirm.set(true);
        });

        let open_alert = with!(show_alert => move || {
            show_alert.set(true);
        });

        let open_custom = with!(show_custom => move || {
            show_custom.set(true);
        });

        let on_confirm_yes = with!(show_confirm, deleted_count, last_action => move || {
            deleted_count.set(deleted_count.get() + 1);
            last_action.set(format!("Deleted item #{}", deleted_count.get()));
            show_confirm.set(false);
        });

        let on_confirm_no = with!(show_confirm, last_action => move || {
            last_action.set("Cancelled delete".to_string());
            show_confirm.set(false);
        });

        let on_alert_dismiss = with!(show_alert, last_action => move || {
            last_action.set("Dismissed alert".to_string());
            show_alert.set(false);
        });

        let on_custom_save = with!(show_custom, custom_input, last_action => move || {
            let value = custom_input.get();
            if !value.is_empty() {
                last_action.set(format!("Saved: {}", value));
                custom_input.set(String::new());
            }
            show_custom.set(false);
        });

        let on_custom_dismiss = with!(show_custom => move || {
            show_custom.set(false);
        });

        View::vstack()
            .spacing(1)
            .child(
                // Header
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .child(View::styled_text("Modal Dialogs Demo").bold().build())
                            .child(
                                View::styled_text("Click buttons to open different modal types")
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
                            .child(View::text("Modal Types:"))
                            .child(
                                View::hstack()
                                    .spacing(2)
                                    .child(
                                        View::button()
                                            .label("Confirm Dialog")
                                            .on_press(open_confirm)
                                            .build(),
                                    )
                                    .child(
                                        View::button()
                                            .label("Alert Dialog")
                                            .on_press(open_alert)
                                            .build(),
                                    )
                                    .child(
                                        View::button()
                                            .label("Custom Modal")
                                            .on_press(open_custom)
                                            .build(),
                                    )
                                    .build(),
                            )
                            .child(View::spacer())
                            .child(
                                View::styled_text(format!(
                                    "Deleted items: {}",
                                    deleted_count.get()
                                ))
                                .color(Color::Yellow)
                                .build(),
                            )
                            .child(
                                View::styled_text(format!("Last action: {}", last_action.get()))
                                    .dim()
                                    .build(),
                            )
                            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
                            .build(),
                    )
                    .build(),
            )
            // Help modal
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 23: Modal")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Confirm, alert, and custom modals"))
                            .child(View::text("• Modal focus containment"))
                            .child(View::text("• Escape to dismiss"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::modal() creates overlay"))
                            .child(View::text("• .visible() controls show/hide"))
                            .child(View::text("• .on_dismiss() handles Escape"))
                            .child(View::text("• Focus trapped in open modal"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Open confirm, click Yes/No"))
                            .child(View::text("• Custom modal has text input"))
                            .child(View::text("• Press Escape to close modals"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 24_async_data: async loading"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            // Confirm dialog modal
            .child(
                View::modal()
                    .visible(show_confirm.get())
                    .title("Confirm Delete")
                    .on_dismiss(on_confirm_no.clone())
                    .width(40)
                    .height(30)
                    .child(
                        View::vstack()
                            .spacing(1)
                            .child(View::text("Are you sure you want to delete this item?"))
                            .child(View::text("This action cannot be undone."))
                            .child(View::spacer())
                            .child(
                                View::hstack()
                                    .spacing(2)
                                    .child(
                                        View::button()
                                            .label("Yes, Delete")
                                            .on_press(on_confirm_yes)
                                            .build(),
                                    )
                                    .child(
                                        View::button()
                                            .label("Cancel")
                                            .on_press(on_confirm_no)
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            // Alert dialog modal
            .child(
                View::modal()
                    .visible(show_alert.get())
                    .title("Alert")
                    .on_dismiss(on_alert_dismiss.clone())
                    .width(50)
                    .height(25)
                    .child(
                        View::vstack()
                            .spacing(1)
                            .child(
                                View::styled_text("Operation completed successfully!")
                                    .color(Color::Green)
                                    .build(),
                            )
                            .child(View::text("Your changes have been saved."))
                            .child(View::spacer())
                            .child(
                                View::button()
                                    .label("OK")
                                    .on_press(on_alert_dismiss)
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            // Custom modal with input
            .child(
                View::modal()
                    .visible(show_custom.get())
                    .title("Enter Details")
                    .on_dismiss(on_custom_dismiss.clone())
                    .width(50)
                    .height(40)
                    .child(
                        View::vstack()
                            .spacing(1)
                            .child(View::text("Enter a value:"))
                            .child(
                                View::text_input()
                                    .value(custom_input.get())
                                    .placeholder("Type something...")
                                    .on_change(with!(custom_input => move |v: String| {
                                        custom_input.set(v);
                                    }))
                                    .build(),
                            )
                            .child(View::spacer())
                            .child(
                                View::hstack()
                                    .spacing(2)
                                    .child(
                                        View::button()
                                            .label("Save")
                                            .on_press(on_custom_save)
                                            .build(),
                                    )
                                    .child(
                                        View::button()
                                            .label("Cancel")
                                            .on_press(on_custom_dismiss)
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
