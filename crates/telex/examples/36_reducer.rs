//! Example 36: Reducer — Multi-Step Wizard
//!
//! A state machine driven by reducer! macro. Each step of the wizard
//! dispatches actions to advance, go back, or reset.
//!
//! Run with: `cargo run -p telex-tui --example 36_reducer`

use crossterm::event::KeyCode;
use crossterm::style::Color;
use telex::prelude::*;

telex::require_api!(0, 2);

fn main() {
    telex::run(App).unwrap();
}

#[derive(Clone, PartialEq)]
enum WizardState {
    Welcome,
    Name(String),
    Color(String, String),
    Done(String, String),
}

#[derive(Clone)]
enum WizardAction {
    Next,
    Back,
    SetName(String),
    SetColor(String),
    Reset,
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let show_help = state!(cx, || false);

        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        let (wizard, dispatch) = reducer!(cx, WizardState::Welcome, |state: WizardState, action: WizardAction| {
            match (state, action) {
                (_, WizardAction::Reset) => WizardState::Welcome,
                (WizardState::Welcome, WizardAction::Next) => WizardState::Name(String::new()),
                (WizardState::Name(_), WizardAction::SetName(n)) => WizardState::Name(n),
                (WizardState::Name(name), WizardAction::Next) => {
                    let name = if name.is_empty() { "Anonymous".to_string() } else { name };
                    WizardState::Color(name, String::new())
                }
                (WizardState::Name(_), WizardAction::Back) => WizardState::Welcome,
                (WizardState::Color(name, _), WizardAction::SetColor(c)) => WizardState::Color(name, c),
                (WizardState::Color(name, color), WizardAction::Next) => {
                    let color = if color.is_empty() { "Blue".to_string() } else { color };
                    WizardState::Done(name, color)
                }
                (WizardState::Color(_, _), WizardAction::Back) => WizardState::Name(String::new()),
                (WizardState::Done(_, _), WizardAction::Back) => WizardState::Welcome,
                (s, _) => s,
            }
        });

        let step = match &wizard.get() {
            WizardState::Welcome => 1,
            WizardState::Name(_) => 2,
            WizardState::Color(_, _) => 3,
            WizardState::Done(_, _) => 4,
        };

        let progress = format!("Step {} of 4", step);
        let dots: String = (1..=4)
            .map(|i| if i <= step { "●" } else { "○" })
            .collect::<Vec<_>>()
            .join(" ");

        let content = match &wizard.get() {
            WizardState::Welcome => {
                let d = dispatch.clone();
                View::vstack()
                    .spacing(1)
                    .child(View::styled_text("Welcome to the Wizard!").color(Color::Cyan).bold().build())
                    .child(View::text("This example shows centralized state"))
                    .child(View::text("management with reducer!"))
                    .child(
                        View::button()
                            .label("[ Start -> ]")
                            .on_press(move || d(WizardAction::Next))
                            .build(),
                    )
                    .build()
            }
            WizardState::Name(name) => {
                let d1 = dispatch.clone();
                let d2 = dispatch.clone();
                let d3 = dispatch.clone();
                View::vstack()
                    .spacing(1)
                    .child(View::styled_text("What's your name?").color(Color::Cyan).bold().build())
                    .child(
                        View::text_input()
                            .value(name.clone())
                            .placeholder("Enter your name...")
                            .on_change(move |s: String| d1(WizardAction::SetName(s)))
                            .build(),
                    )
                    .child(
                        View::hstack()
                            .spacing(1)
                            .child(
                                View::button()
                                    .label("[ <- Back ]")
                                    .on_press(move || d2(WizardAction::Back))
                                    .build(),
                            )
                            .child(
                                View::button()
                                    .label("[ Next -> ]")
                                    .on_press(move || d3(WizardAction::Next))
                                    .build(),
                            )
                            .build(),
                    )
                    .build()
            }
            WizardState::Color(name, color) => {
                let d1 = dispatch.clone();
                let d2 = dispatch.clone();
                let d3 = dispatch.clone();
                View::vstack()
                    .spacing(1)
                    .child(View::styled_text(format!("Hi, {}! Pick a color:", name)).color(Color::Cyan).bold().build())
                    .child(
                        View::text_input()
                            .value(color.clone())
                            .placeholder("Enter a color (e.g. Blue)...")
                            .on_change(move |s: String| d1(WizardAction::SetColor(s)))
                            .build(),
                    )
                    .child(
                        View::hstack()
                            .spacing(1)
                            .child(
                                View::button()
                                    .label("[ <- Back ]")
                                    .on_press(move || d2(WizardAction::Back))
                                    .build(),
                            )
                            .child(
                                View::button()
                                    .label("[ Finish -> ]")
                                    .on_press(move || d3(WizardAction::Next))
                                    .build(),
                            )
                            .build(),
                    )
                    .build()
            }
            WizardState::Done(name, color) => {
                let d = dispatch.clone();
                View::vstack()
                    .spacing(1)
                    .child(View::styled_text("All done!").color(Color::Green).bold().build())
                    .child(View::text(format!("Name:  {}", name)))
                    .child(View::text(format!("Color: {}", color)))
                    .child(
                        View::button()
                            .label("[ Start Over ]")
                            .on_press(move || d(WizardAction::Reset))
                            .build(),
                    )
                    .build()
            }
        };

        View::vstack()
            .spacing(1)
            .child(View::styled_text("Reducer Wizard").bold().build())
            .child(
                View::hstack()
                    .spacing(1)
                    .child(View::styled_text(&progress).dim().build())
                    .child(View::styled_text(&dots).color(Color::Cyan).build())
                    .build(),
            )
            .child(View::styled_text("────────────────────────").dim().build())
            .child(content)
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 36: Reducer")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Multi-step wizard state machine"))
                            .child(View::text("• All transitions in one reducer fn"))
                            .child(View::text("• No scattered booleans"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• reducer!(cx, init, |state, action| ...)"))
                            .child(View::text("• Returns (State<S>, Rc<dyn Fn(A)>)"))
                            .child(View::text("• dispatch(action) to transition"))
                            .child(View::text("• Pattern match (state, action) pairs"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Walk through all 4 steps"))
                            .child(View::text("• Go back and change answers"))
                            .child(View::text("• Watch the progress dots"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("-> 37_error_boundary: crash protection"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
