//! Example 01: Hello World
//!
//! The absolute minimum telex app. If this doesn't work, nothing will.
//!
//! Run with: cargo run --example 01_hello_world

use crossterm::event::KeyCode;
use telex::prelude::*;

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

        View::vstack()
            .child(View::styled_text("Hello World").bold().build())
            .child(View::gap(1))
            .child(View::text("Welcome to Telex!"))
            .child(View::gap(1))
            .child(
                View::styled_text("F1 for help • Ctrl+Q to quit")
                    .dim()
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 01: Hello World")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text(
                                "• Basic app structure with struct + Component trait",
                            ))
                            .child(View::text(
                                "• View::text() and View::styled_text() for display",
                            ))
                            .child(View::text("• View::vstack() for vertical layout"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• Every Telex app implements Component"))
                            .child(View::text("• render() returns a View tree"))
                            .child(View::text("• No state yet - this is purely static"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 02_counter: add state and interactivity"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
