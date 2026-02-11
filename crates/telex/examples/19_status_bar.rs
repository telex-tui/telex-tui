//! Example 19: Status Bar
//!
//! Demonstrates the status bar widget with various configurations:
//! - Basic status bar with left section
//! - Status bar with left and right sections
//! - Status bar with all three sections (left, center, right)
//! - Custom colors
//!
//! Run with: cargo run -p telex-tui --example 19_status_bar

use crossterm::event::KeyCode;
use crossterm::style::Color;
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
            .child(View::styled_text("Status Bar Examples").bold().build())
            .child(View::text(""))
            .child(View::text("Basic status bar (left only):"))
            .child(View::status_bar().left("NORMAL").build())
            .child(View::text(""))
            .child(View::text("Left and right sections:"))
            .child(
                View::status_bar()
                    .left("INSERT")
                    .right("Ln 42, Col 8")
                    .build(),
            )
            .child(View::text(""))
            .child(View::text("All three sections:"))
            .child(
                View::status_bar()
                    .left("VISUAL")
                    .center("main.rs")
                    .right("UTF-8 | LF | Rust")
                    .build(),
            )
            .child(View::text(""))
            .child(View::text("Custom colors (green on dark):"))
            .child(
                View::status_bar()
                    .left("SUCCESS")
                    .center("All tests passed")
                    .right("100%")
                    .fg(Color::Green)
                    .bg(Color::DarkGreen)
                    .build(),
            )
            .child(View::text(""))
            .child(View::text("Editor-style status bar:"))
            .child(
                View::status_bar()
                    .left("-- INSERT --")
                    .center("~/projects/myapp/src/main.rs [+]")
                    .right("1/100 | 50%")
                    .build(),
            )
            .child(View::spacer())
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 19: Status Bar")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Status bars with left/center/right sections"))
                            .child(View::text("• Custom foreground and background colors"))
                            .child(View::text("• Editor-style status line example"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::status_bar() creates status lines"))
                            .child(View::text("• .left(), .center(), .right() for sections"))
                            .child(View::text("• .fg() and .bg() for custom colors"))
                            .child(View::text("• Great for showing mode, file info, etc."))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Compare different status bar styles"))
                            .child(View::text("• Notice how sections align"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 20_menu_bar: dropdown menus"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
