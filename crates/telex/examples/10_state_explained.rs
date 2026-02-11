//! Example 10: State Explained
//!
//! This example explains Telex's state model, which can be surprising
//! if you're coming from other languages or frameworks.
//!
//! Run with: cargo run -p telex --example 10_state_explained

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
        let count = state!(cx, || 0i32);
        let show_help = state!(cx, || false);

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        // Clone handles for closures (this is the pattern being explained)
        let count_for_increment = count.clone();
        let count_for_decrement = count.clone();
        let count_for_reset = count.clone();

        let increment = move || {
            let current = count_for_increment.get();
            count_for_increment.set(current + 1);
        };

        let decrement = move || {
            let current = count_for_decrement.get();
            count_for_decrement.set(current - 1);
        };

        let reset = move || {
            count_for_reset.set(0);
        };

        let current_value = count.get();

        // Hook ordering demo
        let _always_called_1 = state!(cx, || "hook 1");
        let _always_called_2 = state!(cx, || "hook 2");

        View::vstack()
            .spacing(1)
            .child(
                View::styled_text("State Explained")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .child(View::styled_text("The Mental Model:").bold().build())
                            .child(View::gap(1))
                            .child(View::text("  State<T> is a HANDLE, not data"))
                            .child(View::text("  clone() copies the handle, not the data"))
                            .child(View::text("  All handles point to ONE value"))
                            .child(View::gap(1))
                            .child(View::text("  count ──────┐"))
                            .child(View::text("              ├──► i32: 0  (shared!)"))
                            .child(View::text("  count2 ─────┘"))
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::hstack()
                    .spacing(1)
                    .child(View::text("Current value:"))
                    .child(
                        View::styled_text(format!("{}", current_value))
                            .color(Color::Yellow)
                            .bold()
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::hstack()
                    .spacing(1)
                    .child(View::button().label(" - ").on_press(decrement).build())
                    .child(View::button().label(" + ").on_press(increment).build())
                    .child(View::button().label("Reset").on_press(reset).build())
                    .build(),
            )
            .child(
                View::styled_text("All three buttons modify the SAME underlying i32")
                    .dim()
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::styled_text("Tab navigate • F1 help • Ctrl+Q quit")
                    .dim()
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 10: State Explained")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• State<T> as a handle/pointer concept"))
                            .child(View::text("• Multiple closures sharing one value"))
                            .child(View::text("• Visual diagram of the mental model"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• clone() copies handle, not data"))
                            .child(View::text("• All handles point to same underlying value"))
                            .child(View::text(
                                "• Hooks must be called in same order every render",
                            ))
                            .child(View::gap(1))
                            .child(View::styled_text("Important rule").bold().build())
                            .child(View::text("• NEVER put use_state inside an if block"))
                            .child(View::text("• Use with!() macro to simplify cloning"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 11_checkbox: toggle controls"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
