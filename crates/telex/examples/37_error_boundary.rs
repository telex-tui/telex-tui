//! Example 37: Error Boundary — Crash Protection
//!
//! A component that deliberately panics when a counter reaches 5.
//! The error boundary catches the panic and renders a fallback.
//!
//! Run with: `cargo run -p telex-tui --example 37_error_boundary`

use crossterm::event::KeyCode;
use crossterm::style::Color;
use telex::buffer::{Buffer, Rect};
use telex::prelude::*;
use telex::widget::Widget;

telex::require_api!(0, 2);

fn main() {
    telex::run(App).unwrap();
}

/// A widget that panics at render time when the counter reaches 5.
struct RiskyCounter(i32);

impl Widget for RiskyCounter {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        assert!(self.0 < 5, "Counter hit 5 — boom!");
        let text = format!("Counter: {} (panics at 5)", self.0);
        buf.write_str(area.x, area.y, &text, Color::Green, Color::Reset);
    }

    fn height_hint(&self, _width: u16) -> Option<u16> {
        Some(1)
    }
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let show_help = state!(cx, || false);

        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        let count = state!(cx, || 0i32);

        // The panic must happen at render time (inside render_view) so the
        // error boundary's catch_unwind can catch it. A custom widget defers
        // execution to the render pass.
        let risky_view = View::custom(std::rc::Rc::new(std::cell::RefCell::new(RiskyCounter(count.get()))));

        let fallback = View::vstack()
            .child(View::styled_text("CAUGHT PANIC").color(Color::Red).bold().build())
            .child(View::text("The child view panicked."))
            .child(View::text("But the app is still running!"))
            .build();

        View::vstack()
            .spacing(1)
            .child(View::styled_text("Error Boundary Demo").bold().build())
            .child(
                View::hstack()
                    .spacing(2)
                    .child(
                        View::vstack()
                            .spacing(1)
                            .child(View::styled_text("Protected Panel").bold().build())
                            .child(
                                View::error_boundary()
                                    .child(risky_view)
                                    .fallback(fallback)
                                    .build(),
                            )
                            .build(),
                    )
                    .child(
                        View::vstack()
                            .spacing(1)
                            .child(View::styled_text("How It Works").bold().build())
                            .child(View::text("The left panel asserts"))
                            .child(View::text("count < 5. When it hits"))
                            .child(View::text("5, the error boundary"))
                            .child(View::text("catches the panic and"))
                            .child(View::text("renders the fallback."))
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::hstack()
                    .spacing(1)
                    .child(
                        View::button()
                            .label("[ + Increment ]")
                            .on_press(with!(count => move || count.update(|n| *n += 1)))
                            .build(),
                    )
                    .child(
                        View::button()
                            .label("[ Reset to 0 ]")
                            .on_press(with!(count => move || count.set(0)))
                            .build(),
                    )
                    .child(View::styled_text(format!("count = {}", count.get())).dim().build())
                    .build(),
            )
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 37: Error Boundary")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• A counter that panics at 5"))
                            .child(View::text("• Error boundary catches the panic"))
                            .child(View::text("• Red fallback replaces the crash"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::error_boundary()"))
                            .child(View::text("• .child(risky) .fallback(safe)"))
                            .child(View::text("• Panics are caught, not propagated"))
                            .child(View::text("• App keeps running after panic"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Increment to 5 to trigger panic"))
                            .child(View::text("• Reset to 0 to recover"))
                            .child(View::text("• Keep incrementing past 5"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("-> 38_custom_widget: Game of Life"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
