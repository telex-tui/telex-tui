//! Example 27: Keyed State (Order-Independent Hooks)
//!
//! This example demonstrates the new `state!` macro which provides
//! order-independent state hooks. Unlike `use_state`, these can be used
//! conditionally without causing panics.
//!
//! Run with: cargo run -p telex --example 27_keyed_state
//!
//! ## The Problem with use_state
//!
//! Traditional hooks must be called in the same order every render:
//! ```text
//! // WRONG - This panics!
//! if show_counter {
//!     let count = cx.use_state(|| 0);  // Index shifts based on condition
//! }
//! ```
//!
//! ## The Solution: state!
//!
//! Each macro invocation creates a unique type as the key, so order doesn't matter:
//! ```text
//! // SAFE - This works!
//! if show_counter {
//!     let count = state!(cx, || 0);  // Key is baked into the code location
//! }
//! ```

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

        // Two independent toggles
        let show_a = state!(cx, || true);
        let show_b = state!(cx, || true);

        // COUNTER A - state created inside conditional
        let counter_a = if show_a.get() {
            let count = state!(cx, || 0);
            let inc = with!(count => move || count.update(|n| *n += 1));

            View::hstack()
                .spacing(1)
                .child(
                    View::styled_text(format!("{}", count.get()))
                        .color(Color::Yellow)
                        .bold()
                        .build(),
                )
                .child(View::button().label("+").on_press(inc).build())
                .build()
        } else {
            View::styled_text("--").dim().build()
        };

        // COUNTER B - state created inside a DIFFERENT conditional
        let counter_b = if show_b.get() {
            let count = state!(cx, || 0);
            let inc = with!(count => move || count.update(|n| *n += 1));

            View::hstack()
                .spacing(1)
                .child(
                    View::styled_text(format!("{}", count.get()))
                        .color(Color::Magenta)
                        .bold()
                        .build(),
                )
                .child(View::button().label("+").on_press(inc).build())
                .build()
        } else {
            View::styled_text("--").dim().build()
        };

        let toggle_a = with!(show_a => move |_: bool| show_a.update(|b| *b = !*b));
        let toggle_b = with!(show_b => move |_: bool| show_b.update(|b| *b = !*b));

        View::vstack()
            .spacing(1)
            .child(
                View::styled_text("state! Demo")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::hstack()
                    .spacing(2)
                    // Counter A box
                    .child(
                        View::boxed()
                            .border(true)
                            .padding(1)
                            .max_width(25)
                            .child(
                                View::vstack()
                                    .child(View::styled_text("Counter A").bold().build())
                                    .child(View::gap(1))
                                    .child(
                                        View::hstack()
                                            .spacing(1)
                                            .child(View::text("Value:"))
                                            .child(counter_a)
                                            .build(),
                                    )
                                    .child(
                                        View::hstack()
                                            .spacing(1)
                                            .child(View::text("Show:"))
                                            .child(
                                                View::checkbox()
                                                    .checked(show_a.get())
                                                    .on_toggle(toggle_a)
                                                    .build(),
                                            )
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    // Counter B box
                    .child(
                        View::boxed()
                            .border(true)
                            .padding(1)
                            .max_width(25)
                            .child(
                                View::vstack()
                                    .child(View::styled_text("Counter B").bold().build())
                                    .child(View::gap(1))
                                    .child(
                                        View::hstack()
                                            .spacing(1)
                                            .child(View::text("Value:"))
                                            .child(counter_b)
                                            .build(),
                                    )
                                    .child(
                                        View::hstack()
                                            .spacing(1)
                                            .child(View::text("Show:"))
                                            .child(
                                                View::checkbox()
                                                    .checked(show_b.get())
                                                    .on_toggle(toggle_b)
                                                    .build(),
                                            )
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(View::styled_text("Try this:").bold().build())
            .child(View::text(
                "  1. Increment both counters to different values",
            ))
            .child(View::text("  2. Hide counter A (uncheck its box)"))
            .child(View::text("  3. Counter B continues to work just fine!"))
            .child(View::text("  4. Show A again - it remembers its value"))
            .child(View::gap(1))
            .child(
                View::styled_text("They don't interfere with each other.")
                    .color(Color::Green)
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .child(View::styled_text("The code:").bold().build())
                            .child(View::gap(1))
                            .child(
                                View::styled_text("if show_a.get() {")
                                    .color(Color::DarkGrey)
                                    .build(),
                            )
                            .child(
                                View::styled_text("    let count = state!(cx, || 0);")
                                    .color(Color::Yellow)
                                    .build(),
                            )
                            .child(View::styled_text("}").color(Color::DarkGrey).build())
                            .child(
                                View::styled_text("if show_b.get() {")
                                    .color(Color::DarkGrey)
                                    .build(),
                            )
                            .child(
                                View::styled_text("    let count = state!(cx, || 0);")
                                    .color(Color::Magenta)
                                    .build(),
                            )
                            .child(View::styled_text("}").color(Color::DarkGrey).build())
                            .child(View::gap(1))
                            .child(View::text("With use_state, hiding A would CRASH B"))
                            .child(View::text("(hook indices would shift)."))
                            .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::styled_text("Tab: navigate | F1 help | Ctrl+Q: quit")
                    .dim()
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 27: Keyed State")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• state! macro for order-independent hooks"))
                            .child(View::text("• Conditional state that doesn't crash"))
                            .child(View::text("• Two counters with hide/show toggles"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• state!(cx, || init) creates keyed state"))
                            .child(View::text("• Each call site gets unique key"))
                            .child(View::text("• Safe to use inside if blocks"))
                            .child(View::text("• Values persist when hidden/shown"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Increment both counters"))
                            .child(View::text("• Hide counter A"))
                            .child(View::text("• Counter B still works!"))
                            .child(View::text("• Show A again - value preserved"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 28_shared_state: shared state via keys"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
