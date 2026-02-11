mod jsx_test; // Test JSX syntax compilation

use std::thread;
use std::time::Duration;
use telex::prelude::*;
use telex::theme::{self, Theme};
use telex::Color;

/// Shared app configuration - provided via context
#[derive(Clone)]
struct AppConfig {
    app_name: String,
    version: String,
}

fn main() {
    telex::run_with_theme(
        |cx: Scope| {
            // Provide app config via context (no prop drilling needed!)
            cx.provide_context(AppConfig {
                app_name: "Telex Demo".to_string(),
                version: "0.2.1".to_string(),
            });

            // Theme selection state
            let theme_idx = state!(cx, || 2usize); // Start with nord
            let themes = ["dark", "light", "nord", "monokai"];

            // Modal visibility state
            let show_modal = state!(cx, || false);
            let sm = show_modal.clone();

            // Text input with cursor tracking
            let search = state!(cx, String::new);
            let search_cursor = state!(cx, || 0usize);

            // Text area content
            let notes = state!(cx, String::new);
            let notes_line = state!(cx, || 0usize);
            let notes_col = state!(cx, || 0usize);

            // Register keyboard commands for theme switching
            let ti = theme_idx.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::F(1)), move || {
                ti.set(0);
                theme::set_theme(Theme::dark());
            });

            let ti = theme_idx.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::F(2)), move || {
                ti.set(1);
                theme::set_theme(Theme::light());
            });

            let ti = theme_idx.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::F(3)), move || {
                ti.set(2);
                theme::set_theme(Theme::nord());
            });

            let ti = theme_idx.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::F(4)), move || {
                ti.set(3);
                theme::set_theme(Theme::monokai());
            });

            // F5 to toggle help modal
            let sm_toggle = show_modal.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::F(5)), move || {
                sm_toggle.update(|v| *v = !*v);
            });

            // Escape to close modal
            let sm_close = show_modal.clone();
            cx.use_command(KeyBinding::key(telex::KeyCode::Esc), move || {
                sm_close.set(false);
            });

            // Async data loading
            let data = async_data!(cx, || {
                thread::sleep(Duration::from_secs(2));
                Ok(vec![
                    "main".to_string(),
                    "develop".to_string(),
                    "feature/widgets".to_string(),
                ])
            });

            // Counter state
            let count = state!(cx, || 0);
            let c1 = count.clone();
            let c2 = count.clone();

            // List selection
            let selected = state!(cx, || 0);
            let sel = selected.clone();

            // Read config from context (demonstrating context usage)
            let config = cx.use_context::<AppConfig>().unwrap();

            // ── Left column: Search + Branches + TextArea ──
            let left_col = View::vstack()
                .spacing(0)
                .child(
                    View::boxed()
                        .border(true)
                        .flex(1) // Give search box flex space so it can render
                        .child({
                            let s = search.clone();
                            let sc = search_cursor.clone();
                            View::vstack()
                                .spacing(1)
                                .child(View::styled_text("Search (TextInput)").bold().build())
                                .child(
                                    View::text_input()
                                        .value(search.get())
                                        .placeholder("Type to search...")
                                        .cursor(search_cursor.get())
                                        .on_change(move |v| s.set(v))
                                        .on_cursor_change(move |pos| sc.set(pos))
                                        .build(),
                                )
                                .build()
                        })
                        .build(),
                )
                .child(
                    View::boxed()
                        .border(true)
                        .flex(1)
                        .child(
                            View::vstack()
                                .spacing(1)
                                .child(View::styled_text("Branches").bold().build())
                                .child({
                                    // Always show a List to keep focus indices stable
                                    let items = match data {
                                        Async::Loading => vec!["Loading...".to_string()],
                                        Async::Ready(ref items) => items.clone(),
                                        Async::Error(ref e) => vec![format!("Error: {}", e)],
                                    };
                                    View::list()
                                        .items(items)
                                        .selected(selected.get())
                                        .on_select(move |i| sel.set(i))
                                        .build()
                                })
                                .build(),
                        )
                        .build(),
                )
                .child(
                    View::boxed()
                        .border(true)
                        .flex(1)
                        .child({
                            let n = notes.clone();
                            let nl = notes_line.clone();
                            let nc = notes_col.clone();
                            View::vstack()
                                .spacing(1)
                                .child(View::styled_text("Notes (TextArea)").bold().build())
                                .child({
                                    View::text_area()
                                        .value(notes.get())
                                        .placeholder("Enter notes...")
                                        .cursor_line(notes_line.get())
                                        .cursor_col(notes_col.get())
                                        .rows(3)
                                        .on_change(move |s| n.set(s))
                                        .on_cursor_change(move |line, col| {
                                            nl.set(line);
                                            nc.set(col);
                                        })
                                        .build()
                                })
                                .build()
                        })
                        .build(),
                )
                .build();

            // ── Right column: Counter + Styled text ──
            let right_col = View::vstack()
                .spacing(0)
                .child(
                    View::boxed()
                        .border(true)
                        .flex(1)
                        .child(
                            View::vstack()
                                .spacing(1)
                                .child(View::styled_text("Counter").bold().build())
                                .child(View::text(format!("Value: {}", count.get())))
                                .child(
                                    View::hstack()
                                        .spacing(2)
                                        .child(
                                            View::button()
                                                .label("Subtract")
                                                .on_press(move || c1.update(|n| *n -= 1))
                                                .build(),
                                        )
                                        .child(
                                            View::button()
                                                .label("Add")
                                                .on_press(move || c2.update(|n| *n += 1))
                                                .build(),
                                        )
                                        .build(),
                                )
                                .build(),
                        )
                        .build(),
                )
                .child(
                    View::boxed()
                        .border(true)
                        .flex(1)
                        .child(
                            View::vstack()
                                .spacing(0)
                                .child(View::styled_text("Text Styling").bold().build())
                                .child(View::styled_text("Bold text").bold().build())
                                .child(View::styled_text("Italic text").italic().build())
                                .child(View::styled_text("Underlined").underline().build())
                                .child(View::styled_text("Red error").color(Color::Red).build())
                                .child(
                                    View::styled_text("Green success")
                                        .color(Color::Green)
                                        .bold()
                                        .build(),
                                )
                                .child(View::styled_text("Dimmed text").dim().build())
                                .build(),
                        )
                        .build(),
                )
                .build();

            // ── Help Modal ──
            let help_modal = View::modal()
                .visible(show_modal.get())
                .title("Help")
                .width(50)
                .height(50)
                .on_dismiss(move || sm.set(false))
                .child(
                    View::vstack()
                        .spacing(1)
                        .child(
                            View::styled_text("Keyboard Shortcuts")
                                .bold()
                                .underline()
                                .build(),
                        )
                        .child(View::text(""))
                        .child(View::text("F1-F4    Switch themes"))
                        .child(View::text("F5       Toggle help"))
                        .child(View::text("Tab      Next focus"))
                        .child(View::text("Escape   Close modal"))
                        .child(View::text("Ctrl+Q   Quit"))
                        .child(View::text(""))
                        .child(
                            View::hstack()
                                .spacing(2)
                                .child(
                                    View::button()
                                        .label("OK")
                                        .on_press({
                                            let sm = show_modal.clone();
                                            move || sm.set(false)
                                        })
                                        .build(),
                                )
                                .child(
                                    View::button()
                                        .label("Cancel")
                                        .on_press({
                                            let sm = show_modal.clone();
                                            move || sm.set(false)
                                        })
                                        .build(),
                                )
                                .build(),
                        )
                        .build(),
                )
                .build();

            // ── Main layout (uses config from context) ──
            View::vstack()
                .spacing(0)
                .child(
                    View::boxed()
                        .border(true)
                        .child(
                            View::hstack()
                                .child(
                                    View::styled_text(&config.app_name)
                                        .bold()
                                        .color(Color::Cyan)
                                        .build(),
                                )
                                .child(
                                    View::styled_text(format!(" v{}", config.version))
                                        .dim()
                                        .build(),
                                )
                                .child(View::text(format!(
                                    " │ Theme: {} │ F5: Help",
                                    themes[theme_idx.get()]
                                )))
                                .build(),
                        )
                        .build(),
                )
                .child(
                    View::boxed()
                        .flex(1) // Take remaining space after header
                        .child(
                            View::hstack()
                                .spacing(0)
                                .child(View::boxed().flex(1).child(left_col).build())
                                .child(View::boxed().flex(1).child(right_col).build())
                                .build(),
                        )
                        .build(),
                )
                .child(help_modal)
                .build()
        },
        Theme::nord(),
    )
    .unwrap();
}
