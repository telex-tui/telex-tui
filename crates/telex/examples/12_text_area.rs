//! Example 12: TextArea
//!
//! Demonstrates the TextArea widget for multi-line text editing.
//!
//! Run with: cargo run -p telex-tui --example 12_text_area

use crossterm::event::KeyCode;
use telex::prelude::*;
use telex::theme::{set_theme, Theme};
use telex::Color;

telex::require_api!(0, 2);

fn main() {
    telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let show_help = state!(cx, || false);
        let theme_idx = state!(cx, || 0usize);

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        // F2 cycles through themes
        cx.use_command(
            KeyBinding::key(KeyCode::F(2)),
            with!(theme_idx => move || {
                let next = (theme_idx.get() + 1) % 6;
                theme_idx.set(next);
                let theme = match next {
                    0 => Theme::nord(),
                    1 => Theme::dark(),
                    2 => Theme::light(),
                    3 => Theme::dracula(),
                    4 => Theme::gruvbox_dark(),
                    _ => Theme::catppuccin_mocha(),
                };
                set_theme(theme);
            }),
        );

        let content = state!(cx, String::new);
        let cursor_line = state!(cx, || 0usize);
        let cursor_col = state!(cx, || 0usize);

        // Track changes and cursor position
        let on_change = with!(content => move |text: String| {
            content.set(text);
        });

        let on_cursor_change = with!(cursor_line, cursor_col => move |line: usize, col: usize| {
            cursor_line.set(line);
            cursor_col.set(col);
        });

        // Calculate stats
        let text = content.get();
        let line_count = if text.is_empty() {
            0
        } else {
            text.lines().count()
        };
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();

        let theme_name = match theme_idx.get() {
            0 => "Nord",
            1 => "Dark",
            2 => "Light",
            3 => "Dracula",
            4 => "Gruvbox Dark",
            _ => "Catppuccin Mocha",
        };

        View::vstack()
            .spacing(1)
            .child(
                View::hstack()
                    .child(View::styled_text("Notes").color(Color::Cyan).bold().build())
                    .child(View::spacer())
                    .child(View::styled_text(format!("Theme: {}", theme_name)).dim().build())
                    .build(),
            )
            .child(
                View::styled_text("A simple multi-line text editor")
                    .dim()
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::text_area()
                    .value(content.get())
                    .placeholder("Start typing your notes here...")
                    .rows(12)
                    .cursor_line(cursor_line.get())
                    .cursor_col(cursor_col.get())
                    .on_change(on_change)
                    .on_cursor_change(on_cursor_change)
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::hstack()
                    .spacing(3)
                    .child(
                        View::styled_text(format!("Lines: {}", line_count))
                            .color(Color::DarkGrey)
                            .build(),
                    )
                    .child(
                        View::styled_text(format!("Words: {}", word_count))
                            .color(Color::DarkGrey)
                            .build(),
                    )
                    .child(
                        View::styled_text(format!("Chars: {}", char_count))
                            .color(Color::DarkGrey)
                            .build(),
                    )
                    .build(),
            )
            .child(View::gap(1))
            .child(View::styled_text("F1 help • F2 theme • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 12: TextArea")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Multi-line text editing with TextArea"))
                            .child(View::text("• Real-time line/word/char counts"))
                            .child(View::text("• Cursor position tracking"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::text_area() for multi-line input"))
                            .child(View::text("• on_change callback for text updates"))
                            .child(View::text("• on_cursor_change for cursor tracking"))
                            .child(View::text("• placeholder text when empty"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Type multiple lines of text"))
                            .child(View::text("• Watch the stats update in real-time"))
                            .child(View::text("• Use arrow keys to navigate"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 13_split_panes: resizable panel layouts"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
