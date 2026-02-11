//! Example 29: Canvas (Kitty Graphics)
//!
//! Demonstrates the canvas widget for pixel-level drawing using the Kitty
//! graphics protocol.
//!
//! Features:
//! - Drawing primitives (lines, rectangles, circles)
//! - Bar chart visualization
//! - Animation support
//!
//! NOTE: Requires a Kitty-protocol compatible terminal:
//! - Kitty
//! - Ghostty
//! - WezTerm
//!
//! Run with: cargo run -p telex-tui --example 29_canvas

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
        let show_help = state!(cx, || false);

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        // Animated value for the bar chart using use_stream directly
        let frame_stream = cx.use_stream(|| {
            (0u32..).inspect(|&i| {
                if i > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            })
        });

        let current_frame = frame_stream.get();

        View::vstack()
            .spacing(1)
            .child(
                View::styled_text("Canvas Examples (Kitty Graphics)")
                    .bold()
                    .build(),
            )
            .child(View::text("Requires Kitty, Ghostty, or WezTerm terminal"))
            .child(View::text(""))
            // Basic shapes demo
            .child(View::text("Basic Shapes:"))
            .child(
                View::canvas()
                    .width(200)
                    .height(80)
                    .on_draw(|ctx| {
                        // Clear to dark background
                        ctx.clear(Color::Rgb {
                            r: 30,
                            g: 30,
                            b: 40,
                        });

                        // Draw some lines
                        ctx.line(10, 10, 190, 10, Color::Red);
                        ctx.line(10, 10, 10, 70, Color::Green);
                        ctx.line(10, 70, 190, 70, Color::Blue);
                        ctx.line(190, 10, 190, 70, Color::Yellow);

                        // Draw diagonal lines
                        ctx.line(10, 10, 190, 70, Color::Cyan);
                        ctx.line(10, 70, 190, 10, Color::Magenta);

                        // Draw filled rectangles
                        ctx.fill_rect(30, 25, 30, 20, Color::Red);
                        ctx.fill_rect(80, 25, 30, 20, Color::Green);
                        ctx.fill_rect(130, 25, 30, 20, Color::Blue);

                        // Draw stroked rectangles
                        ctx.stroke_rect(30, 50, 30, 15, Color::Yellow);
                        ctx.stroke_rect(80, 50, 30, 15, Color::Cyan);
                        ctx.stroke_rect(130, 50, 30, 15, Color::Magenta);
                    })
                    .build(),
            )
            .child(View::text(""))
            // Circles demo
            .child(View::text("Circles:"))
            .child(
                View::canvas()
                    .width(200)
                    .height(60)
                    .on_draw(|ctx| {
                        ctx.clear(Color::Rgb {
                            r: 20,
                            g: 25,
                            b: 35,
                        });

                        // Filled circles
                        ctx.fill_circle(30, 30, 20, Color::Red);
                        ctx.fill_circle(80, 30, 15, Color::Green);
                        ctx.fill_circle(120, 30, 10, Color::Blue);

                        // Stroked circles
                        ctx.circle(160, 30, 20, Color::Yellow);
                        ctx.circle(160, 30, 15, Color::Cyan);
                        ctx.circle(160, 30, 10, Color::Magenta);
                    })
                    .build(),
            )
            .child(View::text(""))
            // Animated bar chart
            .child(View::text("Animated Bar Chart:"))
            .child(
                View::canvas()
                    .width(200)
                    .height(80)
                    .on_draw({
                        move |ctx| {
                            ctx.clear(Color::Rgb {
                                r: 25,
                                g: 25,
                                b: 30,
                            });

                            // Generate animated data
                            let data: Vec<f32> = (0..8)
                                .map(|i| {
                                    let phase = (current_frame as f32 * 0.1) + (i as f32 * 0.5);
                                    0.3 + 0.7 * ((phase.sin() + 1.0) / 2.0)
                                })
                                .collect();

                            let bar_width = 20u16;
                            let gap = 5u16;
                            let max_height = 60u16;
                            let start_x = 10u16;
                            let baseline = 75u16;

                            // Draw baseline
                            ctx.line(5, baseline as i32, 195, baseline as i32, Color::Grey);

                            // Draw bars
                            let colors = [
                                Color::Red,
                                Color::Green,
                                Color::Blue,
                                Color::Yellow,
                                Color::Cyan,
                                Color::Magenta,
                                Color::Rgb {
                                    r: 255,
                                    g: 128,
                                    b: 0,
                                },
                                Color::Rgb {
                                    r: 128,
                                    g: 255,
                                    b: 128,
                                },
                            ];

                            for (i, &value) in data.iter().enumerate() {
                                let x = start_x + (i as u16) * (bar_width + gap);
                                let height = (value * max_height as f32) as u16;
                                let y = baseline - height;
                                ctx.fill_rect(x, y, bar_width, height, colors[i % colors.len()]);
                            }
                        }
                    })
                    .build(),
            )
            .child(View::text(""))
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 29: Canvas")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Pixel graphics via Kitty protocol"))
                            .child(View::text("• Lines, rectangles, circles"))
                            .child(View::text("• Animated bar chart"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::canvas() creates drawing area"))
                            .child(View::text("• .on_draw(|ctx| { ... }) draws pixels"))
                            .child(View::text("• ctx.line(), ctx.fill_rect(), etc."))
                            .child(View::text("• Works in Kitty/Ghostty/WezTerm"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Watch the animated bar chart"))
                            .child(View::text("• Run in compatible terminal"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 30_image: image display"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
