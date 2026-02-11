//! Example 32: Side Effects with effect! and effect_once!
//!
//! Demonstrates the effect macros for running side effects:
//! - `effect_once!` - runs only on first render (initialization)
//! - `effect!` - runs when dependencies change
//!
//! These macros are order-independent and safe to use in conditionals.
//!
//! Run with: `cargo run -p telex-tui --example 32_effects`

use crossterm::event::KeyCode;
use telex::prelude::*;

telex::require_api!(0, 2);

fn main() {
    telex::run(App).unwrap();
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

        let count = state!(cx, || 0);
        let name = state!(cx, String::new);
        let last_effect = state!(cx, || String::from("(none yet)"));
        let init_done = state!(cx, || false);

        // Effect that runs only once - initialization
        // Using effect_once! macro (order-independent)
        effect_once!(cx, with!(init_done, last_effect => move || {
            init_done.set(true);
            last_effect.set("effect_once!: initialized!".to_string());
            || {}
        }));

        // Effect that runs when count changes
        // Using effect! macro (order-independent)
        effect!(cx, count.get(), with!(last_effect => move |&val| {
            last_effect.set(format!("effect!: count → {}", val));
            || {}
        }));

        // Effect that runs when name changes
        effect!(cx, name.get(), with!(last_effect => move |n: &String| {
            if !n.is_empty() {
                last_effect.set(format!("effect!: name → \"{}\"", n));
            }
            || {}
        }));

        View::vstack()
            .spacing(1)
            .child(View::styled_text("effect! Demo").bold().build())
            .child(View::text(""))
            .child(View::text(format!("Counter: {}", count.get())))
            .child(
                View::hstack()
                    .spacing(1)
                    .child(
                        View::button()
                            .label("[ - ]")
                            .on_press(with!(count => move || count.update(|n| *n -= 1)))
                            .build(),
                    )
                    .child(
                        View::button()
                            .label("[ + ]")
                            .on_press(with!(count => move || count.update(|n| *n += 1)))
                            .build(),
                    )
                    .build(),
            )
            .child(View::text(""))
            .child({
                let n = name.get();
                View::text(format!(
                    "Name: {}",
                    if n.is_empty() { "(empty)" } else { &n }
                ))
            })
            .child(
                View::text_input()
                    .value(name.get())
                    .placeholder("Type your name...")
                    .on_change(with!(name => move |s| name.set(s)))
                    .build(),
            )
            .child(View::text(""))
            .child(View::styled_text("─── Effect Status ───").dim().build())
            .child(View::text(""))
            .child(View::text(format!(
                "Initialized: {}",
                if init_done.get() { "✓ yes" } else { "no" }
            )))
            .child(View::text(format!("Last effect: {}", last_effect.get())))
            .child(View::text(""))
            .child(View::styled_text("─── How it works ───").dim().build())
            .child(View::text(""))
            .child(View::text("effect_once!  → Ran once at startup"))
            .child(View::text("effect!       → Runs when deps change"))
            .child(View::text(""))
            .child(
                View::styled_text("Press +/- or type to see effects trigger")
                    .dim()
                    .build(),
            )
            .child(View::text(""))
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 32: Effects")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• effect_once! runs at startup"))
                            .child(View::text("• effect! runs when deps change"))
                            .child(View::text("• Last effect shows what triggered"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• effect_once!(cx, || { cleanup })"))
                            .child(View::text("• effect!(cx, deps, |&d| { cleanup })"))
                            .child(View::text("• Return || {} for cleanup"))
                            .child(View::text("• Effects run AFTER render"))
                            .child(View::text("• Safe in conditionals!"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Click +/- to change counter"))
                            .child(View::text("• Type in the name field"))
                            .child(View::text("• Watch 'Last effect' update"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
