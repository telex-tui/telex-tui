//! Example 39: Port — Bidirectional Background Task Runner
//!
//! Demonstrates port! for bidirectional communication between a
//! background worker thread and the UI. Send commands, receive progress.
//!
//! Run with: `cargo run -p telex-tui --example 39_port`

use crossterm::event::KeyCode;
use crossterm::style::Color;
use std::sync::mpsc;
use telex::prelude::*;

telex::require_api!(0, 2);

fn main() {
    telex::run(App).unwrap();
}

#[derive(Clone, Debug)]
enum TaskProgress {
    Started,
    Progress(u8),
    Done(String),
    Cancelled,
}

#[derive(Clone, Debug)]
enum TaskCommand {
    Start,
    Cancel,
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let show_help = state!(cx, || false);

        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        let status = state!(cx, || "Idle".to_string());
        let progress = state!(cx, || 0u8);
        let result: State<Option<String>> = state!(cx, || None);
        let running = state!(cx, || false);

        // Bidirectional port: inbound TaskProgress, outbound TaskCommand
        let port = port!(cx, TaskProgress, TaskCommand);

        // Process incoming progress messages
        for msg in port.rx.get() {
            match msg {
                TaskProgress::Started => {
                    status.set("Working...".to_string());
                    progress.set(0);
                    result.set(None);
                    running.set(true);
                }
                TaskProgress::Progress(pct) => {
                    status.set(format!("Progress: {}%", pct));
                    progress.set(pct);
                }
                TaskProgress::Done(data) => {
                    status.set("Done!".to_string());
                    progress.set(100);
                    result.set(Some(data));
                    running.set(false);
                }
                TaskProgress::Cancelled => {
                    status.set("Cancelled".to_string());
                    running.set(false);
                }
            }
        }

        // Spawn the worker thread on first render
        let worker_started = state!(cx, || false);
        if !worker_started.get() {
            worker_started.set(true);
            let tx_progress = port.rx.tx();
            if let Some(rx_commands) = port.take_outbound_rx() {
                std::thread::spawn(move || {
                    worker_loop(tx_progress, rx_commands);
                });
            }
        }

        let start_task = {
            let tx = port.tx();
            with!(running => move || {
                if !running.get() {
                    let _ = tx.send(TaskCommand::Start);
                }
            })
        };

        let cancel_task = {
            let tx = port.tx();
            with!(running => move || {
                if running.get() {
                    let _ = tx.send(TaskCommand::Cancel);
                }
            })
        };

        let pct = progress.get();
        let bar_width = 30usize;
        let filled = (pct as usize * bar_width) / 100;
        let bar = format!(
            "[{}{}] {}%",
            "█".repeat(filled),
            "░".repeat(bar_width - filled),
            pct
        );

        View::vstack()
            .spacing(1)
            .child(View::styled_text("Port: Background Task Runner").bold().build())
            .child(
                View::hstack()
                    .spacing(1)
                    .child(View::styled_text("Status:").dim().build())
                    .child(View::styled_text(status.get())
                        .color(if running.get() { Color::Yellow } else if pct == 100 { Color::Green } else { Color::Reset })
                        .bold()
                        .build())
                    .build(),
            )
            .child(View::styled_text(&bar).color(
                if pct == 100 { Color::Green }
                else if pct > 50 { Color::Yellow }
                else { Color::Cyan }
            ).build())
            .child(if let Some(data) = result.get() {
                View::vstack()
                    .child(View::styled_text("Result:").dim().build())
                    .child(View::styled_text(format!("  {}", data)).color(Color::Green).build())
                    .build()
            } else {
                View::empty()
            })
            .child(
                View::hstack()
                    .spacing(1)
                    .child(
                        View::button()
                            .label(if running.get() { "[ Running... ]" } else { "[ Start Task ]" })
                            .on_press(start_task)
                            .build(),
                    )
                    .child(
                        View::button()
                            .label("[ Cancel ]")
                            .on_press(cancel_task)
                            .build(),
                    )
                    .build(),
            )
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 39: Port")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Background task with progress"))
                            .child(View::text("• Bidirectional communication"))
                            .child(View::text("• Start and cancel controls"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• port!(cx, InType, OutType)"))
                            .child(View::text("• port.rx.tx() sends to UI"))
                            .child(View::text("• port.tx() sends to worker"))
                            .child(View::text("• port.take_outbound_rx() for worker"))
                            .child(View::text("• port.rx.get() reads this frame"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Start a task, watch progress"))
                            .child(View::text("• Cancel mid-way"))
                            .child(View::text("• Start another after completion"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}

fn worker_loop(tx: telex::WakingSender<TaskProgress>, rx: mpsc::Receiver<TaskCommand>) {
    loop {
        // Wait for a Start command
        match rx.recv() {
            Ok(TaskCommand::Start) => {}
            Ok(TaskCommand::Cancel) => continue,
            Err(_) => return, // UI dropped
        }

        tx.send(TaskProgress::Started).ok();

        let mut cancelled = false;
        for i in 1..=20 {
            std::thread::sleep(std::time::Duration::from_millis(150));

            // Check for cancel
            match rx.try_recv() {
                Ok(TaskCommand::Cancel) => {
                    tx.send(TaskProgress::Cancelled).ok();
                    cancelled = true;
                    break;
                }
                _ => {}
            }

            tx.send(TaskProgress::Progress((i * 5) as u8)).ok();
        }

        if !cancelled {
            tx.send(TaskProgress::Done("Computed 42 widgets successfully!".to_string())).ok();
        }
    }
}
