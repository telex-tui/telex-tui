//! Example 31: Animated Canvas
//!
//! Demonstrates the animated_canvas helper for frame-based animations
//! using the Kitty graphics protocol.
//!
//! Features:
//! - Automatic frame timing management
//! - Bouncing ball animation
//! - Sine wave visualization
//! - Particle effects
//!
//! NOTE: Requires a Kitty-protocol compatible terminal:
//! - Kitty
//! - Ghostty
//! - WezTerm
//!
//! Run with: cargo run -p telex-tui --example 31_animated_canvas

use crossterm::event::KeyCode;
use telex::canvas::animated_canvas;
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
        View::vstack()
            .spacing(1)
            .child(
                View::styled_text("Animated Canvas Demo (Kitty Graphics)")
                    .bold()
                    .build(),
            )
            .child(View::text("Requires Kitty, Ghostty, or WezTerm terminal"))
            .child(View::text(""))
            // Bouncing ball animation
            .child(View::text("Bouncing Ball (60 FPS):"))
            .child(
                animated_canvas(cx.clone())
                    .width(200)
                    .height(60)
                    .fps(60)
                    .on_frame(|ctx, frame| {
                        ctx.clear(Color::Rgb {
                            r: 20,
                            g: 20,
                            b: 30,
                        });

                        // Ball physics
                        let t = frame as f32 * 0.05;
                        let x = 100.0 + 80.0 * t.cos();
                        let y = 30.0 + 20.0 * (t * 1.5).sin().abs();

                        // Shadow
                        ctx.fill_circle(
                            x as u16,
                            55,
                            8,
                            Color::Rgb {
                                r: 40,
                                g: 40,
                                b: 50,
                            },
                        );

                        // Ball with gradient effect (outer to inner)
                        ctx.fill_circle(
                            x as u16,
                            y as u16,
                            12,
                            Color::Rgb {
                                r: 200,
                                g: 50,
                                b: 50,
                            },
                        );
                        ctx.fill_circle(
                            x as u16,
                            y as u16,
                            8,
                            Color::Rgb {
                                r: 255,
                                g: 80,
                                b: 80,
                            },
                        );
                        ctx.fill_circle(
                            x as u16 - 2,
                            y as u16 - 2,
                            3,
                            Color::Rgb {
                                r: 255,
                                g: 200,
                                b: 200,
                            },
                        );

                        // Ground line
                        ctx.line(10, 58, 190, 58, Color::Grey);
                    })
                    .build(),
            )
            .child(View::text(""))
            // Sine wave animation
            .child(View::text("Sine Waves (30 FPS):"))
            .child(
                animated_canvas(cx.clone())
                    .width(200)
                    .height(50)
                    .fps(30)
                    .on_frame(|ctx, frame| {
                        ctx.clear(Color::Rgb {
                            r: 15,
                            g: 25,
                            b: 35,
                        });

                        let phase = frame as f32 * 0.1;

                        // Draw multiple sine waves
                        let colors = [
                            Color::Rgb {
                                r: 255,
                                g: 100,
                                b: 100,
                            },
                            Color::Rgb {
                                r: 100,
                                g: 255,
                                b: 100,
                            },
                            Color::Rgb {
                                r: 100,
                                g: 100,
                                b: 255,
                            },
                        ];

                        for (wave_idx, color) in colors.iter().enumerate() {
                            let offset = wave_idx as f32 * 0.5;
                            let amplitude = 15.0 - wave_idx as f32 * 3.0;

                            for x in 0..200 {
                                let t = x as f32 * 0.05 + phase + offset;
                                let y = 25.0 + amplitude * t.sin();
                                ctx.pixel(x, y as u16, *color);

                                // Make line thicker
                                if y as u16 > 0 {
                                    ctx.pixel(x, y as u16 - 1, *color);
                                }
                            }
                        }

                        // Center line
                        ctx.line(0, 25, 199, 25, Color::DarkGrey);
                    })
                    .build(),
            )
            .child(View::text(""))
            // Particle effect
            .child(View::text("Particle Fountain (45 FPS):"))
            .child(
                animated_canvas(cx.clone())
                    .width(200)
                    .height(60)
                    .fps(45)
                    .on_frame(|ctx, frame| {
                        ctx.clear(Color::Rgb {
                            r: 10,
                            g: 10,
                            b: 20,
                        });

                        // Simple particle system using deterministic "random" based on frame
                        let num_particles = 30;
                        for i in 0..num_particles {
                            // Each particle has a lifecycle based on frame offset
                            let particle_frame = (frame + i * 7) % 60;
                            let life = particle_frame as f32 / 60.0;

                            // Starting position (center bottom)
                            let start_x = 100.0;
                            let start_y = 55.0;

                            // Velocity varies per particle (deterministic "random")
                            let angle = -1.57 + (i as f32 * 0.21).sin() * 0.8; // Around -90 degrees
                            let speed = 40.0 + (i as f32 * 0.37).cos() * 20.0;

                            // Physics: position = start + velocity * time + 0.5 * gravity * time^2
                            let vx = angle.cos() * speed;
                            let vy = angle.sin() * speed;
                            let gravity = 80.0;

                            let x = start_x + vx * life;
                            let y = start_y + vy * life + 0.5 * gravity * life * life;

                            // Fade out as particle ages
                            let alpha = (1.0 - life) * 255.0;

                            // Color based on age (yellow -> orange -> red)
                            let r = 255;
                            let g = ((1.0 - life * 0.7) * 200.0) as u8;
                            let b = ((1.0 - life) * 100.0) as u8;

                            if (0.0..200.0).contains(&x) && (0.0..60.0).contains(&y) {
                                let color = Color::Rgb { r, g, b };
                                ctx.pixel_alpha(x as u16, y as u16, color, alpha as u8);
                                // Make particles slightly larger
                                if x > 0.0 {
                                    ctx.pixel_alpha(
                                        x as u16 - 1,
                                        y as u16,
                                        color,
                                        (alpha * 0.5) as u8,
                                    );
                                }
                                if y > 0.0 {
                                    ctx.pixel_alpha(
                                        x as u16,
                                        y as u16 - 1,
                                        color,
                                        (alpha * 0.5) as u8,
                                    );
                                }
                            }
                        }

                        // Emitter glow
                        ctx.fill_circle(100, 55, 3, Color::Yellow);
                    })
                    .build(),
            )
            .child(View::text(""))
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 31: Animated Canvas")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Bouncing ball animation (60 FPS)"))
                            .child(View::text("• Sine wave visualization (30 FPS)"))
                            .child(View::text("• Particle fountain (45 FPS)"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• animated_canvas(cx) helper"))
                            .child(View::text("• .fps() sets frame rate"))
                            .child(View::text("• .on_frame(|ctx, frame| { }) draws"))
                            .child(View::text("• Frame counter enables animation"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Watch the smooth animations"))
                            .child(View::text("• Notice different FPS rates"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 32_effects: side effects"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
