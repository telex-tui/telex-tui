//! Performance stress test for telex TUI framework.

use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, Instant};
use telex::prelude::*;
use telex::Color;

fn main() {
    telex::run_with_theme(
        |cx: Scope| {
            // Item count (powers of 10: 100, 1000, 10000, 100000)
            let item_power = state!(cx, || 3u32); // 10^3 = 1000
            let item_count = 10u32.pow(item_power.get()) as usize;

            // Selected item in list
            let selected = state!(cx, || 0usize);

            // Frame timing
            let frame_times = state!(cx, Vec::<Duration>::new);
            let last_frame = state!(cx, Instant::now);

            // Stress mode - auto-scroll
            let stress_mode = state!(cx, || false);

            // Record frame time
            let now = Instant::now();
            let elapsed = now.duration_since(last_frame.get());
            last_frame.set(now);

            frame_times.update(|times| {
                times.push(elapsed);
                // Keep last 60 frames
                if times.len() > 60 {
                    times.remove(0);
                }
            });

            // Calculate FPS and avg frame time
            let times = frame_times.get();
            let avg_frame_time = if times.is_empty() {
                Duration::ZERO
            } else {
                times.iter().sum::<Duration>() / times.len() as u32
            };
            let fps = if avg_frame_time.as_secs_f64() > 0.0 {
                (1.0 / avg_frame_time.as_secs_f64()) as u32
            } else {
                0
            };

            // Log metrics every 60 frames to file
            if times.len() == 60 {
                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("perf-test.log")
                {
                    let _ = writeln!(file, "items={} fps={} frame_ms={:.2} stress={}",
                        item_count, fps, avg_frame_time.as_secs_f64() * 1000.0, stress_mode.get());
                }
            }

            // Stress mode: auto-scroll (updates selection every render)
            if stress_mode.get() {
                selected.update(|s| *s = (*s + 1) % item_count);
            }

            // F1 help modal
            let show_help = state!(cx, || false);
            cx.use_command(
                KeyBinding::key(telex::KeyCode::F(1)),
                with!(show_help => move || show_help.update(|v| *v = !*v)),
            );

            // Key handlers
            let power_up = item_power.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::Right), move || {
                power_up.update(|p| *p = (*p + 1).min(5)); // max 100,000
            });

            let power_down = item_power.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::Left), move || {
                power_down.update(|p| *p = p.saturating_sub(1).max(2)); // min 100
            });

            let stress_toggle = stress_mode.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::Char('s')), move || {
                stress_toggle.update(|s| *s = !*s);
            });

            let sel_up = selected.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::Up), move || {
                sel_up.update(|s| *s = s.saturating_sub(1));
            });

            let sel_down = selected.clone();
            let count_down = item_count;
            cx.use_command(KeyBinding::key(telex::KeyCode::Down), move || {
                sel_down.update(|s| *s = (*s + 1).min(count_down - 1));
            });

            let sel_pgup = selected.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::PageUp), move || {
                sel_pgup.update(|s| *s = s.saturating_sub(100));
            });

            let sel_pgdn = selected.clone();
            let count_pgdn = item_count;
            cx.use_command(KeyBinding::key(telex::KeyCode::PageDown), move || {
                sel_pgdn.update(|s| *s = (*s + 100).min(count_pgdn - 1));
            });

            // Generate items
            let items: Vec<String> = (0..item_count)
                .map(|i| format!("Item {:06} - Lorem ipsum dolor sit amet, consectetur adipiscing elit", i))
                .collect();

            // FPS color coding
            let fps_color = if fps >= 30 {
                Color::Green
            } else if fps >= 15 {
                Color::Yellow
            } else {
                Color::Red
            };

            // Stats display
            let stats = View::hstack()
                .spacing(3)
                .child(
                    View::styled_text(format!("FPS: {:3}", fps))
                        .bold()
                        .color(fps_color)
                        .build(),
                )
                .child(View::text(format!("Frame: {:6.2}ms", avg_frame_time.as_secs_f64() * 1000.0)))
                .child(View::text(format!("Items: {:6}", item_count)))
                .child(View::text(format!("Selected: {:6}", selected.get())))
                .child(
                    View::styled_text(if stress_mode.get() { "STRESS: ON " } else { "STRESS: OFF" })
                        .color(if stress_mode.get() { Color::Red } else { Color::DarkGrey })
                        .bold()
                        .build(),
                )
                .build();

            // Controls footer
            let help = View::status_bar()
                .left("F1: help  s: stress  ←/→: items  ↑/↓: scroll  Ctrl+Q: quit")
                .build();

            // F1 help modal
            let help_modal = View::modal()
                .visible(show_help.get())
                .title("Performance Test")
                .on_dismiss(with!(show_help => move || show_help.set(false)))
                .child(
                    View::vstack()
                        .child(View::styled_text("What is this?").bold().build())
                        .child(View::text("A stress test for the Telex rendering pipeline."))
                        .child(View::text("Measures how fast the framework can render"))
                        .child(View::text("large lists and update the screen."))
                        .child(View::gap(1))
                        .child(View::styled_text("Controls").bold().build())
                        .child(View::text("  ←/→        Change item count (10^n)"))
                        .child(View::text("  ↑/↓        Scroll through list"))
                        .child(View::text("  PgUp/PgDn  Jump 100 items"))
                        .child(View::text("  s           Toggle stress mode"))
                        .child(View::text("  Ctrl+Q      Quit"))
                        .child(View::gap(1))
                        .child(View::styled_text("Stats bar").bold().build())
                        .child(View::text("  FPS         Frames per second (green=good)"))
                        .child(View::text("  Frame       Average ms per frame"))
                        .child(View::text("  Items       Current list size"))
                        .child(View::text("  Selected    Currently highlighted row"))
                        .child(View::gap(1))
                        .child(View::styled_text("Stress mode").bold().build())
                        .child(View::text("Auto-scrolls the list every frame, forcing"))
                        .child(View::text("continuous re-renders. Shows worst-case FPS."))
                        .child(View::text("Metrics are logged to perf-test.log."))
                        .child(View::gap(1))
                        .child(View::styled_text("Press Escape to close").dim().build())
                        .build(),
                )
                .build();

            // Main list
            let sel_clone = selected.clone();
            let list = View::list()
                .items(items)
                .selected(selected.get())
                .on_select(move |i| sel_clone.set(i))
                .build();

            View::vstack()
                .spacing(0)
                .child(
                    View::boxed()
                        .border(true)
                        .child(
                            View::styled_text("Telex Performance Test")
                                .bold()
                                .color(Color::Cyan)
                                .build(),
                        )
                        .build(),
                )
                .child(
                    View::boxed()
                        .border(true)
                        .child(stats)
                        .build(),
                )
                .child(
                    View::boxed()
                        .border(true)
                        .flex(1)
                        .min_height(3)
                        .child(list)
                        .build(),
                )
                .child(help)
                .child(help_modal)
                .build()
        },
        telex::theme::Theme::nord(),
    )
    .unwrap();
}
