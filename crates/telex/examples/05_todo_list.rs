//! Example 05: Todo List
//!
//! Demonstrates TextInput, List, and state management patterns.
//!
//! Run with: cargo run -p telex-tui --example 05_todo_list

use crossterm::event::KeyCode;
use crossterm::style::Color;
use telex::prelude::*;

telex::require_api!(0, 1);

fn main() {
    telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let items = state!(cx, || {
            vec![
                "Learn Telex".to_string(),
                "Build something cool".to_string(),
            ]
        });
        let input_value = state!(cx, String::new);
        let selected = state!(cx, || 0usize);
        let show_help = state!(cx, || false);

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        // Add new item on submit
        let on_submit = with!(items, input_value => move || {
            let text = input_value.get();
            if !text.is_empty() {
                items.update(|v| v.push(text));
                input_value.set(String::new());
            }
        });

        // Handle input changes
        let on_change = with!(input_value => move |text: String| {
            input_value.set(text);
        });

        // Delete selected item
        let on_delete = with!(items, selected => move || {
            let idx = selected.get();
            items.update(|v| {
                if idx < v.len() {
                    v.remove(idx);
                    // Adjust selection if needed
                    if idx > 0 && idx >= v.len() {
                        selected.set(idx - 1);
                    }
                }
            });
        });

        // Track selection
        let on_select = with!(selected => move |idx: usize| {
            selected.set(idx);
        });

        let item_count = items.get().len();

        View::vstack()
            .child(
                View::styled_text("Todo List")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::text_input()
                    .value(input_value.get())
                    .placeholder("Type something to add...")
                    .on_change(on_change)
                    .on_submit(on_submit)
                    .build(),
            )
            .child(View::gap(1))
            .child(if item_count > 0 {
                View::list()
                    .items(items.get())
                    .selected(selected.get())
                    .on_select(on_select)
                    .build()
            } else {
                View::styled_text("No items yet").dim().build()
            })
            .child(View::gap(1))
            .child(
                View::hstack()
                    .child(View::button().label("Delete").on_press(on_delete).build())
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::styled_text("Tab navigate • Enter add/select • F1 help • Ctrl+Q quit")
                    .dim()
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 05: Todo List")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• View::text_input() for text entry"))
                            .child(View::text("• View::list() for displaying items"))
                            .child(View::text("• on_submit callback for Enter key"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• Controlled input: value + on_change"))
                            .child(View::text("• Vec<String> state for list items"))
                            .child(View::text("• Conditional rendering (empty state)"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Type something and press Enter to add"))
                            .child(View::text("• Use ↑/↓ to select, then Delete button"))
                            .child(View::text("• Delete all items to see empty state"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 06_log_viewer: streaming text content"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
