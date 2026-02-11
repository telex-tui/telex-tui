//! Example 25: Context
//!
//! Demonstrates using provide_context and use_context for sharing
//! global state like themes, user preferences, or app configuration.
//!
//! Run with: `cargo run -p telex-tui --example 25_context`

use crossterm::event::KeyCode;
use telex::prelude::*;
use telex::Color;

telex::require_api!(0, 2);

fn main() {
    telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
}

// Shared state types that will be provided via context
#[derive(Clone)]
struct AppConfig {
    app_name: String,
    version: String,
}

#[derive(Clone, Copy, PartialEq)]
enum ColorTheme {
    Default,
    Ocean,
    Forest,
    Sunset,
}

impl ColorTheme {
    fn primary(&self) -> Color {
        match self {
            ColorTheme::Default => Color::White,
            ColorTheme::Ocean => Color::Cyan,
            ColorTheme::Forest => Color::Green,
            ColorTheme::Sunset => Color::Yellow,
        }
    }

    fn accent(&self) -> Color {
        match self {
            ColorTheme::Default => Color::Blue,
            ColorTheme::Ocean => Color::Blue,
            ColorTheme::Forest => Color::DarkGreen,
            ColorTheme::Sunset => Color::Red,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            ColorTheme::Default => "Default",
            ColorTheme::Ocean => "Ocean",
            ColorTheme::Forest => "Forest",
            ColorTheme::Sunset => "Sunset",
        }
    }
}

#[derive(Clone)]
struct User {
    name: String,
    logged_in: bool,
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

        // State that we'll provide via context
        let theme = state!(cx, || ColorTheme::Default);
        let user = state!(cx, || User {
            name: "Guest".to_string(),
            logged_in: false,
        });

        // Provide static config via context
        cx.provide_context(AppConfig {
            app_name: "Context Demo".to_string(),
            version: "1.0.0".to_string(),
        });

        // Provide dynamic state via context (current values)
        cx.provide_context(theme.get());
        cx.provide_context(user.get());

        // Theme switching handlers
        let set_default = with!(theme => move || theme.set(ColorTheme::Default));
        let set_ocean = with!(theme => move || theme.set(ColorTheme::Ocean));
        let set_forest = with!(theme => move || theme.set(ColorTheme::Forest));
        let set_sunset = with!(theme => move || theme.set(ColorTheme::Sunset));

        // Login/logout handlers
        let toggle_login = with!(user => move || {
            let current = user.get();
            if current.logged_in {
                user.set(User {
                    name: "Guest".to_string(),
                    logged_in: false,
                });
            } else {
                user.set(User {
                    name: "Alice".to_string(),
                    logged_in: true,
                });
            }
        });

        // Get current theme for styling
        let current_theme = theme.get();

        View::vstack()
            .spacing(1)
            // Header - uses context
            .child(render_header(&cx))
            .child(
                // Main content
                View::boxed()
                    .flex(1)
                    .border(true)
                    .padding(1)
                    .child(
                        View::vstack()
                            .spacing(1)
                            .child(
                                View::styled_text("Theme Selection:")
                                    .color(current_theme.primary())
                                    .bold()
                                    .build(),
                            )
                            .child(
                                View::hstack()
                                    .spacing(2)
                                    .child(
                                        View::button()
                                            .label("Default")
                                            .on_press(set_default)
                                            .build(),
                                    )
                                    .child(
                                        View::button().label("Ocean").on_press(set_ocean).build(),
                                    )
                                    .child(
                                        View::button().label("Forest").on_press(set_forest).build(),
                                    )
                                    .child(
                                        View::button().label("Sunset").on_press(set_sunset).build(),
                                    )
                                    .build(),
                            )
                            .child(View::gap(1))
                            .child(
                                View::styled_text("User Actions:")
                                    .color(current_theme.primary())
                                    .bold()
                                    .build(),
                            )
                            .child(
                                View::button()
                                    .label(if user.get().logged_in {
                                        "Logout"
                                    } else {
                                        "Login as Alice"
                                    })
                                    .on_press(toggle_login)
                                    .build(),
                            )
                            .child(View::spacer())
                            // User info panel - uses context
                            .child(render_user_panel(&cx))
                            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
                            .build(),
                    )
                    .build(),
            )
            // Status bar - uses context
            .child(render_status_bar(&cx))
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 25: Context")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Context API for global state"))
                            .child(View::text("• Theme colors propagate everywhere"))
                            .child(View::text("• User state shared across components"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• cx.provide_context() adds to context"))
                            .child(View::text("• cx.use_context::<T>() reads context"))
                            .child(View::text("• Avoids prop drilling"))
                            .child(View::text("• Great for themes, user, config"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Switch themes - colors update"))
                            .child(View::text("• Login/logout - panel updates"))
                            .child(View::text("• Header and status bar read context"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 26_radio_buttons: radio selections"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}

// Helper function that reads from context
fn render_header(cx: &Scope) -> View {
    // Read config and theme from context
    let config = cx.use_context::<AppConfig>();
    let theme = cx
        .use_context::<ColorTheme>()
        .unwrap_or(ColorTheme::Default);

    let title = config
        .map(|c| format!("{} v{}", c.app_name, c.version))
        .unwrap_or_else(|| "No config".to_string());

    View::boxed()
        .border(true)
        .padding(1)
        .child(
            View::vstack()
                .child(
                    View::styled_text(title)
                        .bold()
                        .color(theme.primary())
                        .build(),
                )
                .child(
                    View::styled_text("Demonstrates provide_context and use_context")
                        .dim()
                        .build(),
                )
                .build(),
        )
        .build()
}

// Helper function that reads user from context
fn render_user_panel(cx: &Scope) -> View {
    let user = cx.use_context::<User>();
    let theme = cx
        .use_context::<ColorTheme>()
        .unwrap_or(ColorTheme::Default);

    let (status_text, status_color) = match &user {
        Some(u) if u.logged_in => (format!("Logged in as: {}", u.name), theme.accent()),
        Some(_) => ("Not logged in".to_string(), Color::DarkGrey),
        None => ("User context not available".to_string(), Color::Red),
    };

    View::boxed()
        .border(true)
        .padding(1)
        .child(
            View::vstack()
                .child(
                    View::styled_text("User Panel (reads from context)")
                        .bold()
                        .color(theme.primary())
                        .build(),
                )
                .child(View::styled_text(status_text).color(status_color).build())
                .build(),
        )
        .build()
}

// Helper function for status bar
fn render_status_bar(cx: &Scope) -> View {
    let theme = cx
        .use_context::<ColorTheme>()
        .unwrap_or(ColorTheme::Default);

    View::boxed()
        .border(true)
        .child(
            View::hstack()
                .child(
                    View::styled_text(format!(" Theme: {} ", theme.name()))
                        .color(theme.accent())
                        .build(),
                )
                .child(View::spacer())
                .child(
                    View::styled_text(" Context values propagate automatically ")
                        .dim()
                        .build(),
                )
                .build(),
        )
        .build()
}
