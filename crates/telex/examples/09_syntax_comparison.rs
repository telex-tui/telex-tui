//! Example 09: Syntax Comparison
//!
//! Demonstrates the two ways to build views in Telex:
//! 1. Builder pattern (Rust-native, explicit)
//! 2. view! macro (JSX-like, concise)
//!
//! Both produce identical results - choose whichever you prefer.
//!
//! Run with: cargo run -p telex-tui --example 09_syntax_comparison

use crossterm::event::KeyCode;
use telex::prelude::*;
use telex::Color;

telex::require_api!(0, 2);

fn main() {
    telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let use_jsx = state!(cx, || false);
        let show_help = state!(cx, || false);

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        let toggle = with!(use_jsx => move || use_jsx.set(!use_jsx.get()));

        // Show which syntax is currently displayed
        let syntax_name = if use_jsx.get() {
            "view! macro (JSX-like)"
        } else {
            "Builder pattern"
        };

        View::vstack()
            .child(
                View::styled_text("Syntax Comparison")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(
                View::styled_text("Same UI, two ways to write it")
                    .dim()
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::hstack()
                    .child(View::text("Current syntax: "))
                    .child(
                        View::styled_text(syntax_name)
                            .color(Color::Yellow)
                            .bold()
                            .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(if use_jsx.get() {
                        counter_jsx(cx.clone())
                    } else {
                        counter_builder(cx.clone())
                    })
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::button()
                    .label("Toggle Syntax")
                    .on_press(toggle)
                    .build(),
            )
            .child(View::gap(1))
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 09: Syntax Comparison")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Two syntaxes that produce identical output"))
                            .child(View::text("• Builder: View::vstack().child(...).build()"))
                            .child(View::text("• Macro: view! { <VStack>...</VStack> }"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• Builder is Rust-native, IDE-friendly"))
                            .child(View::text("• view! macro is JSX-like, less boilerplate"))
                            .child(View::text("• Choose based on your preference"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Toggle between syntaxes"))
                            .child(View::text("• Notice the output is identical"))
                            .child(View::text("• Check the source code to see both styles"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 10_state_explained: deep dive into state"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}

// =============================================================================
// Builder Pattern
// =============================================================================
// Rust-native, explicit, IDE-friendly. Each method call is clear.
// Good for: Complex conditional logic, learning the API, debugging.

fn counter_builder(cx: Scope) -> View {
    let count = state!(cx, || 0i32);

    // with! macro clones the handle for you - much cleaner!
    let increment = with!(count => move || count.update(|n| *n += 1));
    let decrement = with!(count => move || count.update(|n| *n -= 1));

    View::vstack()
        .child(
            View::styled_text("// Built with builder pattern")
                .color(Color::DarkGrey)
                .build(),
        )
        .child(View::gap(1))
        .child(
            View::styled_text(format!("Count: {}", count.get()))
                .bold()
                .build(),
        )
        .child(View::gap(1))
        .child(
            View::hstack()
                .child(View::button().label(" - ").on_press(decrement).build())
                .child(View::text(" "))
                .child(View::button().label(" + ").on_press(increment).build())
                .build(),
        )
        .build()
}

// =============================================================================
// view! Macro (JSX-like)
// =============================================================================
// Concise, familiar to React/JSX users. Less boilerplate.
// Good for: Rapid prototyping, simple layouts, JSX fans.

fn counter_jsx(cx: Scope) -> View {
    let count = state!(cx, || 0i32);

    // with! works great with view! macro too
    view! {
        <VStack>
            <StyledText color={Color::DarkGrey}>"// Built with view! macro + with!"</StyledText>
            <Spacer />
            <StyledText bold={true}>{format!("Count: {}", count.get())}</StyledText>
            <Spacer />
            <HStack>
                <Button on_press={with!(count => move || count.update(|n| *n -= 1))}>" - "</Button>
                <Text>" "</Text>
                <Button on_press={with!(count => move || count.update(|n| *n += 1))}>" + "</Button>
            </HStack>
        </VStack>
    }
}
