//! Example 04: Timer
//!
//! Demonstrates streaming updates without user interaction.
//! The timer updates automatically every second.
//!
//! Run with: cargo run -p telex-tui --example 04_timer

use crossterm::event::KeyCode;
use crossterm::style::Color;
use std::time::Duration;
use telex::prelude::*;

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

        // Stream that yields elapsed seconds
        let elapsed = cx.use_stream(|| {
            (0u64..).inspect(|&s| {
                if s > 0 {
                    std::thread::sleep(Duration::from_secs(1));
                }
            })
        });

        let seconds = elapsed.get();
        let is_running = elapsed.is_loading();

        // Format as MM:SS
        let minutes = seconds / 60;
        let secs = seconds % 60;
        let time_display = format!("{:02}:{:02}", minutes, secs);

        View::vstack()
            .child(View::styled_text("Timer").color(Color::Cyan).bold().build())
            .child(View::gap(1))
            .child(
                View::hstack()
                    .child(View::styled_text(&time_display).bold().build())
                    .child(if is_running {
                        View::styled_text(" ●").color(Color::Green).build()
                    } else {
                        View::styled_text(" ○").dim().build()
                    })
                    .build(),
            )
            .child(View::gap(1))
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 04: Timer")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• cx.use_stream() for background data"))
                            .child(View::text("• Auto-updating UI without user input"))
                            .child(View::text("• Green dot = stream is running"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• Streams run in background threads"))
                            .child(View::text("• Each yielded value triggers a re-render"))
                            .child(View::text("• is_loading() tells you if stream is active"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Just watch - the timer ticks automatically"))
                            .child(View::text("• No button presses needed for updates"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 05_todo_list: text input and list management"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
