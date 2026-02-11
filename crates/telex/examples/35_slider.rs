//! Example 35: Slider — RGB Color Mixer
//!
//! Three sliders control red, green, and blue channels.
//! Shows the resulting color as a preview swatch and hex code.
//!
//! Run with: `cargo run -p telex-tui --example 35_slider`

use crossterm::event::KeyCode;
use crossterm::style::Color;
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

        let r = state!(cx, || 128.0);
        let g = state!(cx, || 0.0);
        let b = state!(cx, || 255.0);

        let rv = r.get() as u8;
        let gv = g.get() as u8;
        let bv = b.get() as u8;

        View::vstack()
            .spacing(1)
            .child(View::styled_text("RGB Color Mixer").bold().build())
            .child(
                View::slider()
                    .min(0.0)
                    .max(255.0)
                    .step(1.0)
                    .value(r.get())
                    .label(&format!("Red:   {:>3}", rv))
                    .on_change(with!(r => move |v: f64| r.set(v)))
                    .build(),
            )
            .child(
                View::slider()
                    .min(0.0)
                    .max(255.0)
                    .step(1.0)
                    .value(g.get())
                    .label(&format!("Green: {:>3}", gv))
                    .on_change(with!(g => move |v: f64| g.set(v)))
                    .build(),
            )
            .child(
                View::slider()
                    .min(0.0)
                    .max(255.0)
                    .step(1.0)
                    .value(b.get())
                    .label(&format!("Blue:  {:>3}", bv))
                    .on_change(with!(b => move |v: f64| b.set(v)))
                    .build(),
            )
            .child(
                View::styled_text("████████████████")
                    .color(Color::Rgb { r: rv, g: gv, b: bv })
                    .bold()
                    .build(),
            )
            .child(View::styled_text(format!("#{:02X}{:02X}{:02X}", rv, gv, bv)).bold().build())
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 35: Slider")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Three sliders for R, G, B"))
                            .child(View::text("• Color preview swatch"))
                            .child(View::text("• Live hex code"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::slider() with min/max/step"))
                            .child(View::text("• on_change callback with f64"))
                            .child(View::text("• Color::Rgb for true color"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Tab between sliders"))
                            .child(View::text("• Left/Right arrows to adjust"))
                            .child(View::text("• Watch the preview change"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("-> 36_reducer: state machine wizard"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
