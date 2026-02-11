//! Example 13: Split Panes
//!
//! Demonstrates the Split widget for creating resizable panel layouts.
//!
//! Run with: cargo run -p telex-tui --example 13_split_panes

use crossterm::event::KeyCode;
use crossterm::style::Color;
use telex::prelude::*;

telex::require_api!(0, 2);

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

        let selected_item = state!(cx, || 0usize);

        let items = vec![
            "README.md".to_string(),
            "Cargo.toml".to_string(),
            "src/".to_string(),
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "tests/".to_string(),
        ];

        let on_select = with!(selected_item => move |idx: usize| {
            selected_item.set(idx);
        });

        let detail_text = match selected_item.get() {
            0 => "# README\n\nThis is a demo of split panes.\n\nThe left panel shows a file list,\nthe right panel shows details.",
            1 => "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"",
            2 => "Directory: src/\n\nContains the main source files.",
            3 => "fn main() {\n    println!(\"Hello, world!\");\n}",
            4 => "pub mod utils;\npub mod widgets;",
            5 => "Directory: tests/\n\nContains integration tests.",
            _ => "Select an item to see details.",
        };

        // Horizontal split: file list on left, details on right
        View::vstack()
            .child(
                View::boxed()
                    .flex(1)
                    .child(
                        View::split()
                            .horizontal()
                            .ratio(0.3)
                            .min_first(15)
                            .first(
                                View::vstack()
                                    .child(
                                        View::styled_text("Files")
                                            .color(Color::Cyan)
                                            .bold()
                                            .build(),
                                    )
                                    .child(
                                        View::list()
                                            .items(items)
                                            .selected(selected_item.get())
                                            .on_select(on_select)
                                            .build(),
                                    )
                                    .child(View::gap(1))
                                    .build(),
                            )
                            .second(
                                View::vstack()
                                    .child(
                                        View::styled_text("Details")
                                            .color(Color::Green)
                                            .bold()
                                            .build(),
                                    )
                                    .child(
                                        View::boxed()
                                            .border(true)
                                            .flex(1)
                                            .child(View::text(detail_text))
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::styled_text("↑↓: select file | F1 help | Ctrl+Q: quit")
                    .dim()
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 13: Split Panes")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Horizontal split: file list / details"))
                            .child(View::text("• ratio(0.3) = 30% left, 70% right"))
                            .child(View::text("• min_first(15) sets minimum pane width"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::split() creates resizable panes"))
                            .child(View::text("• .horizontal() or .vertical() orientation"))
                            .child(View::text("• .first() and .second() set pane content"))
                            .child(View::text("• Splits can be nested"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Select different files to see details"))
                            .child(View::text("• Resize terminal to see layout adapt"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 14_tabs: tabbed interfaces"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
