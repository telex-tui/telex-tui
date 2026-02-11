//! Example 02: Counter
//!
//! Basic state management with use_state and button interaction.
//!
//! Run with: cargo run --example 02_counter

use crossterm::event::KeyCode;
use telex::prelude::*;

telex::require_api!(0, 2);

fn main() {
    telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let count = state!(cx, || 0i32);
        let show_help = state!(cx, || false);

        let increment = with!(count => move || count.update(|n| *n += 1));
        let decrement = with!(count => move || count.update(|n| *n -= 1));

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        View::vstack()
            .child(View::styled_text("Counter").bold().build())
            .child(View::gap(1))
            .child(View::text(format!("Count: {}", count.get())))
            .child(View::gap(1))
            .child(
                View::hstack()
                    .child(
                        View::button()
                            .label("Decrement")
                            .on_press(decrement)
                            .build(),
                    )
                    .child(View::text(" "))
                    .child(
                        View::button()
                            .label("Increment")
                            .on_press(increment)
                            .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::styled_text("Tab to switch • Enter to press • F1 for help • Ctrl+Q to quit")
                    .dim()
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 02: Counter")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• state!() macro for reactive state"))
                            .child(View::text("• View::button() with on_press callbacks"))
                            .child(View::text(
                                "• The with!() macro for capturing state in closures",
                            ))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• State persists across renders"))
                            .child(View::text("• Updating state triggers a re-render"))
                            .child(View::text("• Tab navigates between focusable elements"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Press +/- rapidly - notice instant updates"))
                            .child(View::text(
                                "• The UI stays in sync with state automatically",
                            ))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 03_theme_switcher: styling and colors"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
