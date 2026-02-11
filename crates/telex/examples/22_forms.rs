//! Example 22: Form Validation
//!
//! Demonstrates declarative form validation with various validators
//! including required fields, email, min/max length, and custom validation.
//!
//! Run with: `cargo run -p telex-tui --example 22_forms`

use crossterm::event::KeyCode;
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

        // Create form state with validated fields
        let form = state!(cx, || {
            FormState::new()
                .field(
                    FieldBuilder::new("email")
                        .required()
                        .email()
                        .error_message("Please enter a valid email address")
                        .build(),
                )
                .field(
                    FieldBuilder::new("password")
                        .required()
                        .min_length(8)
                        .error_message("Password must be at least 8 characters")
                        .build(),
                )
                .field(
                    FieldBuilder::new("username")
                        .required()
                        .min_length(3)
                        .max_length(20)
                        .custom(|v| {
                            if v.contains(' ') {
                                Some("Username cannot contain spaces".to_string())
                            } else if !v.chars().all(|c| c.is_alphanumeric() || c == '_') {
                                Some(
                                    "Username can only contain letters, numbers, and underscores"
                                        .to_string(),
                                )
                            } else {
                                None
                            }
                        })
                        .build(),
                )
                .field(
                    FieldBuilder::new("age")
                        .integer()
                        .custom(|v| {
                            if v.is_empty() {
                                return None; // Optional field
                            }
                            match v.parse::<i32>() {
                                Ok(age) if age < 0 => Some("Age cannot be negative".to_string()),
                                Ok(age) if age > 150 => {
                                    Some("Please enter a valid age".to_string())
                                }
                                Ok(_) => None,
                                Err(_) => Some("Please enter a valid number".to_string()),
                            }
                        })
                        .build(),
                )
        });

        let submit_message = state!(cx, String::new);

        // Get current field values and errors
        let email = form.get().get_value("email");
        let password = form.get().get_value("password");
        let username = form.get().get_value("username");
        let age = form.get().get_value("age");

        let email_error = form.get().get_error("email");
        let password_error = form.get().get_error("password");
        let username_error = form.get().get_error("username");
        let age_error = form.get().get_error("age");

        // Handlers
        let on_submit = with!(form, submit_message => move || {
            if form.get().validate() {
                let values = form.get().values();
                submit_message.set(format!(
                    "Form submitted! Email: {}, Username: {}",
                    values.get("email").unwrap_or(&String::new()),
                    values.get("username").unwrap_or(&String::new())
                ));
            } else {
                submit_message.set("Please fix the errors above".to_string());
            }
        });

        let on_reset = with!(form, submit_message => move || {
            form.get().reset();
            submit_message.set(String::new());
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
                            .child(View::styled_text("Form Validation Demo").bold().build())
                            .child(
                                View::styled_text("Tab between fields, type to enter values")
                                    .dim()
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(
                // Form
                View::boxed()
                    .flex(1)
                    .border(true)
                    .padding(1)
                    .child(
                        View::form()
                            .spacing(1)
                            .child(
                                View::form_field("email")
                                    .label("Email Address *")
                                    .value(email.clone())
                                    .placeholder("you@example.com")
                                    .error(email_error)
                                    .on_change(with!(form => move |v: String| {
                                        form.get().set_value("email", v);
                                    }))
                                    .on_blur(with!(form => move || {
                                        form.get().touch("email");
                                    }))
                                    .build(),
                            )
                            .child(
                                View::form_field("username")
                                    .label("Username *")
                                    .value(username.clone())
                                    .placeholder("johndoe")
                                    .error(username_error)
                                    .on_change(with!(form => move |v: String| {
                                        form.get().set_value("username", v);
                                    }))
                                    .on_blur(with!(form => move || {
                                        form.get().touch("username");
                                    }))
                                    .build(),
                            )
                            .child(
                                View::form_field("password")
                                    .label("Password * (min 8 chars)")
                                    .value(password.clone())
                                    .placeholder("Enter password")
                                    .password(true)
                                    .error(password_error)
                                    .on_change(with!(form => move |v: String| {
                                        form.get().set_value("password", v);
                                    }))
                                    .on_blur(with!(form => move || {
                                        form.get().touch("password");
                                    }))
                                    .build(),
                            )
                            .child(
                                View::form_field("age")
                                    .label("Age (optional)")
                                    .value(age.clone())
                                    .placeholder("25")
                                    .error(age_error)
                                    .on_change(with!(form => move |v: String| {
                                        form.get().set_value("age", v);
                                    }))
                                    .on_blur(with!(form => move || {
                                        form.get().touch("age");
                                    }))
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(
                // Actions
                View::hstack()
                    .spacing(2)
                    .child(View::button().label("Submit").on_press(on_submit).build())
                    .child(View::button().label("Reset").on_press(on_reset).build())
                    .build(),
            )
            .child(
                // Status
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .child(View::text(if submit_message.get().is_empty() {
                                "Fill in the form and click Submit".to_string()
                            } else {
                                submit_message.get()
                            }))
                            .child(
                                View::styled_text(format!(
                                    "Form valid: {}",
                                    if form.get().is_valid() { "Yes" } else { "No" }
                                ))
                                .color(if form.get().is_valid() {
                                    Color::Green
                                } else {
                                    Color::Red
                                })
                                .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 22: Forms")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Declarative form validation"))
                            .child(View::text("• Required, email, length validators"))
                            .child(View::text("• Custom validation functions"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• FormState manages all fields"))
                            .child(View::text("• FieldBuilder defines validation"))
                            .child(View::text("• View::form_field() renders inputs"))
                            .child(View::text("• on_blur triggers validation"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Enter invalid email, see error"))
                            .child(View::text("• Try short password (<8 chars)"))
                            .child(View::text("• Username with spaces shows error"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 23_modal: modal dialogs"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
