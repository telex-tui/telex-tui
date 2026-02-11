//! Example 07: File Browser
//!
//! Demonstrates real filesystem navigation with a list view.
//!
//! Run with: cargo run -p telex-tui --example 07_file_browser

use crossterm::event::KeyCode;
use crossterm::style::Color;
use std::fs;
use std::path::PathBuf;
use telex::prelude::*;

telex::require_api!(0, 2);

fn main() {
    telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
}

struct App;

fn list_directory(path: &PathBuf) -> Vec<String> {
    let mut entries = Vec::new();

    // Add parent directory option if not at root
    if path.parent().is_some() {
        entries.push("..".to_string());
    }

    if let Ok(read_dir) = fs::read_dir(path) {
        let mut items: Vec<_> = read_dir
            .filter_map(|e| e.ok())
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() {
                    format!("{}/", name)
                } else {
                    name
                }
            })
            .collect();
        items.sort();
        entries.extend(items);
    }

    entries
}

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let current_path =
            state!(cx, || std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")));
        let selected = state!(cx, || 0usize);
        let show_file_info = state!(cx, || false);
        let selected_file_path = state!(cx, String::new);
        let show_help = state!(cx, || false);
        let entries = list_directory(&current_path.get());

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        // Track selection (just updates index, doesn't navigate)
        let on_select = with!(selected => move |idx: usize| {
            selected.set(idx);
        });

        // Dismiss modal
        let on_dismiss = with!(show_file_info => move || show_file_info.set(false));

        // Open directory or show file info on Enter
        let entries_for_cmd = entries.clone();
        cx.use_command(
            KeyBinding::key(KeyCode::Enter),
            with!(current_path, selected, show_file_info, selected_file_path => move || {
                let idx = selected.get();
                if idx < entries_for_cmd.len() {
                    let entry = &entries_for_cmd[idx];
                    let path = current_path.get();

                    if entry == ".." {
                        if let Some(parent) = path.parent() {
                            current_path.set(parent.to_path_buf());
                            selected.set(0);
                        }
                    } else if entry.ends_with('/') {
                        let dir_name = entry.trim_end_matches('/');
                        let new_path = path.join(dir_name);
                        current_path.set(new_path);
                        selected.set(0);
                    } else {
                        // It's a file - show info modal
                        let full_path = path.join(entry);
                        selected_file_path.set(full_path.to_string_lossy().to_string());
                        show_file_info.set(true);
                    }
                }
            }),
        );

        let path_display = current_path.get().to_string_lossy().to_string();

        View::vstack()
            .child(
                View::styled_text("File Browser")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::styled_text(&path_display)
                    .color(Color::Yellow)
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::list()
                    .items(entries)
                    .selected(selected.get())
                    .on_select(on_select)
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::styled_text("↑/↓ navigate • Enter open • F1 help • Ctrl+Q quit")
                    .dim()
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_file_info.get())
                    .title("File")
                    .width(60)
                    .height(20)
                    .on_dismiss(on_dismiss)
                    .child(
                        View::vstack()
                            .child(View::text(selected_file_path.get()))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 07: File Browser")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Real filesystem navigation"))
                            .child(View::text("• cx.use_command() for keyboard shortcuts"))
                            .child(View::text("• Modal for file details"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• std::fs::read_dir for directory listing"))
                            .child(View::text(
                                "• KeyBinding::key(KeyCode::Enter) for Enter handling",
                            ))
                            .child(View::text("• Directories shown with trailing /"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Navigate into directories with Enter"))
                            .child(View::text("• Go up with '..' entry"))
                            .child(View::text("• Select a file to see its path"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text(
                                "→ 08_system_monitor: multiple concurrent streams",
                            ))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
