//! Example 34: Channels and Intervals
//!
//! Demonstrates external event handling with channel! and interval!:
//! - `interval!` fires a callback every second (live tick counter)
//! - `channel!` receives messages from background threads
//! - A button spawns a worker thread that sleeps then sends a result
//!
//! Run with: `cargo run -p telex-tui --example 34_channels_and_intervals`

use crossterm::event::KeyCode;
use crossterm::style::Color;
use std::time::Duration;
use telex::prelude::*;

telex::require_api!(0, 2);

fn main() {
    telex::run(App).unwrap();
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let show_help = state!(cx, || false);

        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        let ticks = state!(cx, || 0u64);
        let messages: State<Vec<String>> = state!(cx, Vec::new);
        let tasks_running = state!(cx, || 0u32);

        // Tick every second
        interval!(cx, Duration::from_secs(1), with!(ticks => move || {
            ticks.update(|n| *n += 1);
        }));

        // Channel for worker thread results
        let ch = channel!(cx, String);

        // Process incoming messages
        for msg in ch.get() {
            messages.update(|v| v.push(msg));
            tasks_running.update(|n| *n = n.saturating_sub(1));
        }

        let spawn_task = {
            let tx = ch.tx();
            let task_num = messages.get().len() + tasks_running.get() as usize + 1;
            with!(tasks_running => move || {
                tasks_running.update(|n| *n += 1);
                let tx = tx.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(2));
                    tx.send(format!("Task {} complete!", task_num)).ok();
                });
            })
        };

        let elapsed = ticks.get();
        let mins = elapsed / 60;
        let secs = elapsed % 60;

        View::vstack()
            .spacing(1)
            .child(View::styled_text("Channels & Intervals").bold().build())
            .child(
                View::hstack()
                    .spacing(1)
                    .child(View::styled_text("Elapsed:").dim().build())
                    .child(View::styled_text(format!("{:02}:{:02}", mins, secs)).color(Color::Cyan).bold().build())
                    .child(View::styled_text(format!("({} ticks)", elapsed)).dim().build())
                    .build(),
            )
            .child(
                View::hstack()
                    .spacing(1)
                    .child(
                        View::button()
                            .label("[ Spawn Background Task ]")
                            .on_press(spawn_task)
                            .build(),
                    )
                    .child(if tasks_running.get() > 0 {
                        View::styled_text(format!("{} running...", tasks_running.get()))
                            .color(Color::Yellow)
                            .build()
                    } else {
                        View::styled_text("idle").dim().build()
                    })
                    .build(),
            )
            .child(View::styled_text("─── Messages ───").dim().build())
            .child({
                let msgs = messages.get();
                if msgs.is_empty() {
                    View::styled_text("(no messages yet — spawn a task!)").dim().build()
                } else {
                    let len = msgs.len();
                    msgs.iter()
                        .enumerate()
                        .fold(View::vstack(), |stack, (i, m)| {
                            stack.child(
                                View::styled_text(format!("  {} {}", if i == len - 1 { "→" } else { " " }, m))
                                    .color(if i == len - 1 { Color::Green } else { Color::Reset })
                                    .build(),
                            )
                        })
                        .build()
                }
            })
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 34: Channels & Intervals")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• interval! ticks every second"))
                            .child(View::text("• channel! receives from threads"))
                            .child(View::text("• Workers sleep 2s then send a message"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• interval!(cx, dur, || callback)"))
                            .child(View::text("• channel!(cx, Type) for inbound msgs"))
                            .child(View::text("• ch.tx() gives a WakingSender"))
                            .child(View::text("• ch.get() reads this frame's messages"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Spawn several tasks at once"))
                            .child(View::text("• Watch messages arrive after 2s"))
                            .child(View::text("• Notice the tick counter keeps going"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("-> 35_slider: bounded numeric input"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
