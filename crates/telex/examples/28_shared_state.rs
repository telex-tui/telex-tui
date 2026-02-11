//! Example 28: Shared State (Same Key = Same State)
//!
//! This example shows the OPPOSITE of example 27.
//!
//! In example 27, each `state!` call creates independent state
//! because each macro invocation generates a unique anonymous key type.
//!
//! Here, we use an EXPLICIT key type, so multiple calls with the SAME key
//! all access the SAME state.
//!
//! Run with: cargo run -p telex --example 28_shared_state

use crossterm::event::KeyCode;
use telex::prelude::*;
use telex::Color;

telex::require_api!(0, 1);

// Define a NAMED key type - anywhere this is used, we get the SAME state
struct SharedCounterKey;

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

        // Both panes will use the SAME key type = SAME state
        // Note: using use_state_keyed directly with an explicit key type

        // PANE A - gets the shared counter
        let count_a = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);
        let inc_a = with!(count_a => move || count_a.update(|n| *n += 1));

        // PANE B - uses the SAME key, so gets the SAME state!
        let count_b = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);
        let inc_b = with!(count_b => move || count_b.update(|n| *n += 1));

        View::vstack()
            .spacing(1)
            .child(
                View::styled_text("Shared State Demo")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::hstack()
                    .spacing(2)
                    // Pane A
                    .child(
                        View::boxed()
                            .border(true)
                            .padding(1)
                            .max_width(25)
                            .child(
                                View::vstack()
                                    .child(View::styled_text("Pane A").bold().build())
                                    .child(View::gap(1))
                                    .child(
                                        View::hstack()
                                            .spacing(1)
                                            .child(View::text("Value:"))
                                            .child(
                                                View::styled_text(format!("{}", count_a.get()))
                                                    .color(Color::Yellow)
                                                    .bold()
                                                    .build(),
                                            )
                                            .child(View::button().label("+").on_press(inc_a).build())
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    // Pane B
                    .child(
                        View::boxed()
                            .border(true)
                            .padding(1)
                            .max_width(25)
                            .child(
                                View::vstack()
                                    .child(View::styled_text("Pane B").bold().build())
                                    .child(View::gap(1))
                                    .child(
                                        View::hstack()
                                            .spacing(1)
                                            .child(View::text("Value:"))
                                            .child(
                                                View::styled_text(format!("{}", count_b.get()))
                                                    .color(Color::Yellow)
                                                    .bold()
                                                    .build(),
                                            )
                                            .child(View::button().label("+").on_press(inc_b).build())
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
            .child(View::text("  1. Click + on Pane A"))
            .child(View::text("  2. Watch Pane B update too!"))
            .child(View::text("  3. Click + on Pane B - same thing"))
            .child(View::gap(1))
            .child(
                View::styled_text("Both panes share the SAME state.")
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
                                View::styled_text("struct SharedCounterKey;  // Named key type")
                                    .color(Color::Green)
                                    .build(),
                            )
                            .child(View::gap(1))
                            .child(View::styled_text("// Pane A").color(Color::DarkGrey).build())
                            .child(
                                View::styled_text("let count_a = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);")
                                    .color(Color::Yellow)
                                    .build(),
                            )
                            .child(View::gap(1))
                            .child(View::styled_text("// Pane B - SAME key type!").color(Color::DarkGrey).build())
                            .child(
                                View::styled_text("let count_b = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);")
                                    .color(Color::Yellow)
                                    .build(),
                            )
                            .child(View::gap(1))
                            .child(View::text("Same key = same state. Both variables"))
                            .child(View::text("point to the same underlying value."))
                            .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .child(View::styled_text("Compare with example 27:").bold().build())
                            .child(View::gap(1))
                            .child(View::styled_text("state!(cx, || 0)  // Anonymous key").color(Color::Magenta).build())
                            .child(View::text("  Each call = unique key = independent state"))
                            .child(View::gap(1))
                            .child(View::styled_text("cx.use_state_keyed::<MyKey, _>(|| 0)  // Named key").color(Color::Yellow).build())
                            .child(View::text("  Same key = same state = shared everywhere"))
                            .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(View::styled_text("Tab: navigate | F1 help | Ctrl+Q: quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 28: Shared State")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Two panes sharing ONE counter"))
                            .child(View::text("• Both + buttons increment same value"))
                            .child(View::text("• Named key type = shared state"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• struct SharedCounterKey; defines key"))
                            .child(View::text("• use_state_keyed::<Key, _>() uses it"))
                            .child(View::text("• Same key = same underlying value"))
                            .child(View::text("• Opposite of state! (unique keys)"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Click + on Pane A"))
                            .child(View::text("• Watch Pane B update too!"))
                            .child(View::text("• Both buttons modify same state"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 29_canvas: pixel graphics"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build()
                    )
                    .build()
            )
            .build()
    }
}
