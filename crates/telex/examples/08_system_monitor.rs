//! Example 08: System Monitor
//!
//! Demonstrates multiple streams and complex layout with simulated system stats.
//!
//! Run with: cargo run -p telex-tui --example 08_system_monitor

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

        // CPU usage stream (fluctuates between 10-90%)
        let cpu = cx.use_stream(|| {
            let mut rng_state = 42u64;
            (0..).map(move |_| {
                std::thread::sleep(Duration::from_millis(500));
                // Simple LCG pseudo-random
                rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
                
                ((rng_state >> 16) % 80) as u8 + 10
            })
        });

        // Memory usage stream (slowly increases then drops)
        let memory = cx.use_stream(|| {
            (0..).map(|i| {
                std::thread::sleep(Duration::from_millis(800));
                let cycle = i % 20;
                if cycle < 15 {
                    40 + (cycle * 3) as u8
                } else {
                    40 + ((20 - cycle) * 8) as u8
                }
            })
        });

        // Network stream (random-ish traffic)
        let network = cx.use_stream(|| {
            let mut rng_state = 123u64;
            (0..).map(move |_| {
                std::thread::sleep(Duration::from_millis(300));
                rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
                
                ((rng_state >> 16) % 1000) as u32 + 100
            })
        });

        let cpu_val = cpu.get();
        let mem_val = memory.get();
        let net_val = network.get();

        // Color based on value
        let cpu_color = if cpu_val > 80 {
            Color::Red
        } else if cpu_val > 50 {
            Color::Yellow
        } else {
            Color::Green
        };

        let mem_color = if mem_val > 80 {
            Color::Red
        } else if mem_val > 60 {
            Color::Yellow
        } else {
            Color::Green
        };

        // Create progress bar (ASCII to avoid multi-byte char issues)
        fn progress_bar(value: u8, width: usize) -> String {
            let filled = (value as usize * width) / 100;
            let empty = width - filled;
            format!("[{}{}]", "#".repeat(filled), "-".repeat(empty))
        }

        View::vstack()
            .child(
                View::styled_text("System Monitor")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::hstack()
                    .child(View::text("CPU:    "))
                    .child(
                        View::styled_text(progress_bar(cpu_val, 20))
                            .color(cpu_color)
                            .build(),
                    )
                    .child(
                        View::styled_text(format!(" {:>3}%", cpu_val))
                            .bold()
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::hstack()
                    .child(View::text("Memory: "))
                    .child(
                        View::styled_text(progress_bar(mem_val, 20))
                            .color(mem_color)
                            .build(),
                    )
                    .child(
                        View::styled_text(format!(" {:>3}%", mem_val))
                            .bold()
                            .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::hstack()
                    .child(View::text("Network: "))
                    .child(
                        View::styled_text(format!("{:>6} KB/s", net_val))
                            .color(Color::Magenta)
                            .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 08: System Monitor")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text(
                                "• Multiple independent streams running together",
                            ))
                            .child(View::text("• Color-coded thresholds (green/yellow/red)"))
                            .child(View::text("• ASCII progress bars"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• Each use_stream() runs in its own thread"))
                            .child(View::text(
                                "• Streams update independently at different rates",
                            ))
                            .child(View::text("• Conditional styling based on values"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Watch CPU fluctuate randomly"))
                            .child(View::text("• Memory climbs then drops cyclically"))
                            .child(View::text("• Network updates fastest (300ms)"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text(
                                "→ 09_syntax_comparison: builder vs macro syntax",
                            ))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
