//! Example 24: Async Data Loading
//!
//! Demonstrates the async_data! macro for loading data asynchronously,
//! showing loading states, success states, and error handling.
//!
//! Run with: `cargo run -p telex-tui --example 24_async_data`

use crossterm::event::KeyCode;
use std::thread;
use std::time::Duration;
use telex::prelude::*;
use telex::Color;

telex::require_api!(0, 2);

fn main() {
    telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
}

struct App;

// Simulated data types
#[derive(Clone)]
struct UserProfile {
    name: String,
    email: String,
    member_since: String,
}

#[derive(Clone)]
struct Stats {
    posts: u32,
    followers: u32,
    following: u32,
}

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let show_help = state!(cx, || false);

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        // Simulate fetching user profile (slow - 2 seconds)
        let profile = async_data!(cx, || {
            thread::sleep(Duration::from_secs(2));
            Ok(UserProfile {
                name: "Alice Johnson".to_string(),
                email: "alice@example.com".to_string(),
                member_since: "January 2024".to_string(),
            })
        });

        // Simulate fetching user stats (medium - 1 second)
        let stats = async_data!(cx, || {
            thread::sleep(Duration::from_secs(1));
            Ok(Stats {
                posts: 142,
                followers: 1234,
                following: 567,
            })
        });

        // Simulate a failing request
        let failing_data = async_data!(cx, || {
            thread::sleep(Duration::from_millis(500));
            Err::<String, _>("Network error: Connection refused".to_string())
        });

        View::vstack()
            .spacing(1)
            .child(
                // Header
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .child(View::styled_text("Async Data Loading Demo").bold().build())
                            .child(
                                View::styled_text(
                                    "Demonstrates use_async for loading data with loading/error states",
                                )
                                .dim()
                                .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(
                // Main content - three columns
                View::hstack()
                    .spacing(1)
                    // Profile section
                    .child(
                        View::boxed()
                            .flex(1)
                            .border(true)
                            .padding(1)
                            .child(render_profile_section(&profile))
                            .build(),
                    )
                    // Stats section
                    .child(
                        View::boxed()
                            .flex(1)
                            .border(true)
                            .padding(1)
                            .child(render_stats_section(&stats))
                            .build(),
                    )
                    // Error section
                    .child(
                        View::boxed()
                            .flex(1)
                            .border(true)
                            .padding(1)
                            .child(render_error_section(&failing_data))
                            .build(),
                    )
                    .build(),
            )
            .child(
                // Status footer
                View::boxed()
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .child(View::text(format!(
                                "Overall status: {}",
                                if profile.is_loading() || stats.is_loading() {
                                    "Loading..."
                                } else if profile.is_error() || stats.is_error() || failing_data.is_error() {
                                    "Some requests failed"
                                } else {
                                    "All data loaded"
                                }
                            )))
                            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 24: Async Data")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Three async data loads in parallel"))
                            .child(View::text("• Loading, success, and error states"))
                            .child(View::text("• Different load times for each"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• async_data!() macro runs in background"))
                            .child(View::text("• Returns Async<T> enum"))
                            .child(View::text("• .is_loading() / .is_error() helpers"))
                            .child(View::text("• Pattern match for state handling"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Watch data load progressively"))
                            .child(View::text("• Notice the failing request"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 25_context: context API"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build()
                    )
                    .build()
            )
            .build()
    }
}

fn render_profile_section(profile: &Async<UserProfile>) -> View {
    View::vstack()
        .spacing(1)
        .child(
            View::styled_text("User Profile")
                .bold()
                .color(Color::Cyan)
                .build(),
        )
        .child(View::styled_text("(loads in 2s)").dim().build())
        .child(View::gap(1))
        .child(match profile {
            Async::Loading => View::vstack()
                .child(View::styled_text("Loading...").dim().build())
                .child(View::text("[=========>          ]"))
                .build(),
            Async::Ready(p) => View::vstack()
                .spacing(0)
                .child(View::text(format!("Name: {}", p.name)))
                .child(View::text(format!("Email: {}", p.email)))
                .child(View::text(format!("Member since: {}", p.member_since)))
                .build(),
            Async::Error(e) => View::styled_text(format!("Error: {}", e))
                .color(Color::Red)
                .build(),
        })
        .build()
}

fn render_stats_section(stats: &Async<Stats>) -> View {
    View::vstack()
        .spacing(1)
        .child(
            View::styled_text("User Stats")
                .bold()
                .color(Color::Green)
                .build(),
        )
        .child(View::styled_text("(loads in 1s)").dim().build())
        .child(View::gap(1))
        .child(match stats {
            Async::Loading => View::vstack()
                .child(View::styled_text("Loading...").dim().build())
                .child(View::text("[==================> ]"))
                .build(),
            Async::Ready(s) => View::vstack()
                .spacing(0)
                .child(View::text(format!("Posts: {}", s.posts)))
                .child(View::text(format!("Followers: {}", s.followers)))
                .child(View::text(format!("Following: {}", s.following)))
                .build(),
            Async::Error(e) => View::styled_text(format!("Error: {}", e))
                .color(Color::Red)
                .build(),
        })
        .build()
}

fn render_error_section(data: &Async<String>) -> View {
    View::vstack()
        .spacing(1)
        .child(
            View::styled_text("Failing Request")
                .bold()
                .color(Color::Red)
                .build(),
        )
        .child(View::styled_text("(fails after 0.5s)").dim().build())
        .child(View::gap(1))
        .child(match data {
            Async::Loading => View::vstack()
                .child(View::styled_text("Loading...").dim().build())
                .child(View::text("[=====================]"))
                .build(),
            Async::Ready(d) => View::text(format!("Data: {}", d)),
            Async::Error(e) => View::vstack()
                .child(
                    View::styled_text("Request failed!")
                        .color(Color::Red)
                        .build(),
                )
                .child(View::styled_text(e.clone()).dim().build())
                .build(),
        })
        .build()
}
