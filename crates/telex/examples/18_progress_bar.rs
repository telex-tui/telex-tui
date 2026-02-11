//! Example 18: Progress Bar
//!
//! Demonstrates the progress bar widget with various configurations:
//! - Basic progress bar
//! - Progress bar with label
//! - Progress bar without percentage
//! - Custom characters
//! - Animated progress
//!
//! Run with: cargo run -p telex-tui --example 18_progress_bar

use crossterm::event::KeyCode;
use std::time::Duration;
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

        // Animated progress value using stream
        let progress = stream!(cx, || {
            (0u64..).map(|i| {
                if i > 0 {
                    std::thread::sleep(Duration::from_millis(50));
                }
                // Progress cycles from 0.0 to 1.0
                (i % 100) as f32 / 100.0
            })
        });

        let current_progress = progress.get();

        View::vstack()
            .spacing(1)
            .child(View::styled_text("Progress Bar Examples").bold().build())
            .child(View::text(""))
            // Basic progress bar
            .child(View::text("Basic (75%):"))
            .child(View::progress_bar().value(0.75).build())
            // With label
            .child(View::text("With label (50%):"))
            .child(View::progress_bar().value(0.5).label("Loading").build())
            // Without percentage
            .child(View::text("No percentage (33%):"))
            .child(
                View::progress_bar()
                    .value(0.33)
                    .show_percentage(false)
                    .build(),
            )
            // Fixed width
            .child(View::text("Fixed width (20 chars, 60%):"))
            .child(View::progress_bar().value(0.6).width(20).build())
            // Custom characters
            .child(View::text("Custom characters (80%):"))
            .child(
                View::progress_bar()
                    .value(0.8)
                    .filled_char('=')
                    .empty_char('-')
                    .width(20)
                    .build(),
            )
            // Another style
            .child(View::text("Block style (65%):"))
            .child(
                View::progress_bar()
                    .value(0.65)
                    .filled_char('#')
                    .empty_char('.')
                    .width(25)
                    .build(),
            )
            // Animated progress
            .child(View::text("Animated (loops 0-100%):"))
            .child(
                View::progress_bar()
                    .value(current_progress)
                    .label("Progress")
                    .build(),
            )
            .child(View::text(""))
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 18: Progress Bar")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Progress bars with various styles"))
                            .child(View::text("• Animated progress using stream!() macro"))
                            .child(View::text("• Custom fill and empty characters"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::progress_bar() creates bars"))
                            .child(View::text("• .value(0.0 to 1.0) sets progress"))
                            .child(View::text("• .label() adds text label"))
                            .child(View::text("• .filled_char() / .empty_char() customize"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Watch the animated bar loop"))
                            .child(View::text("• Compare different bar styles"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 19_status_bar: status bar widget"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
