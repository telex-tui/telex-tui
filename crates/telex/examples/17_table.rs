//! Example 17: Table Widget
//!
//! Demonstrates the Table widget for data-heavy applications.
//! Features sortable columns, row selection, and various column widths.
//!
//! Run with: cargo run -p telex-tui --example 17_table

use crossterm::event::KeyCode;
use telex::prelude::*;
use telex::Color;

telex::require_api!(0, 1);

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

        // Track selected row
        let selected = state!(cx, || 0usize);

        // Track sort state (column index, ascending)
        let sort_state = state!(cx, || None::<(usize, bool)>);

        // Sample pod data (like k9s)
        let base_data = vec![
            vec![
                "nginx-pod".to_string(),
                "Running".to_string(),
                "12%".to_string(),
                "256Mi".to_string(),
                "2h".to_string(),
            ],
            vec![
                "redis-cache".to_string(),
                "Running".to_string(),
                "8%".to_string(),
                "128Mi".to_string(),
                "5d".to_string(),
            ],
            vec![
                "api-server".to_string(),
                "Running".to_string(),
                "45%".to_string(),
                "512Mi".to_string(),
                "1h".to_string(),
            ],
            vec![
                "db-postgres".to_string(),
                "Running".to_string(),
                "23%".to_string(),
                "1Gi".to_string(),
                "3d".to_string(),
            ],
            vec![
                "worker-1".to_string(),
                "Pending".to_string(),
                "0%".to_string(),
                "0Mi".to_string(),
                "5m".to_string(),
            ],
            vec![
                "worker-2".to_string(),
                "Running".to_string(),
                "67%".to_string(),
                "384Mi".to_string(),
                "45m".to_string(),
            ],
            vec![
                "frontend".to_string(),
                "Running".to_string(),
                "5%".to_string(),
                "64Mi".to_string(),
                "12h".to_string(),
            ],
            vec![
                "metrics".to_string(),
                "CrashLoop".to_string(),
                "0%".to_string(),
                "32Mi".to_string(),
                "2m".to_string(),
            ],
        ];

        // Sort data based on current sort state
        let mut rows = base_data.clone();
        if let Some((col, ascending)) = sort_state.get() {
            rows.sort_by(|a, b| {
                let a_val = a.get(col).map(|s| s.as_str()).unwrap_or("");
                let b_val = b.get(col).map(|s| s.as_str()).unwrap_or("");
                if ascending {
                    a_val.cmp(b_val)
                } else {
                    b_val.cmp(a_val)
                }
            });
        }

        let on_select = with!(selected => move |idx: usize| {
            selected.set(idx);
        });

        let on_sort = with!(sort_state => move |col: usize, asc: bool| {
            sort_state.set(Some((col, asc)));
        });

        let on_activate = with!(selected => move |idx: usize| {
            // In a real app, this might open a details view
            selected.set(idx);
        });

        let selected_name = rows
            .get(selected.get())
            .and_then(|r| r.first())
            .map(|s| s.as_str())
            .unwrap_or("None");

        let sort_info = match sort_state.get() {
            Some((col, asc)) => {
                let col_name = match col {
                    0 => "NAME",
                    1 => "STATUS",
                    2 => "CPU",
                    3 => "MEMORY",
                    4 => "AGE",
                    _ => "?",
                };
                format!(
                    "Sorted by {} ({})",
                    col_name,
                    if asc { "asc" } else { "desc" }
                )
            }
            None => "Unsorted".to_string(),
        };

        View::vstack()
            .child(
                View::styled_text("Pod Dashboard")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(
                View::styled_text(format!("Selected: {} | {}", selected_name, sort_info))
                    .dim()
                    .build(),
            )
            .child(
                View::boxed()
                    .flex(1)
                    .border(true)
                    .child(
                        View::table()
                            .column("NAME")
                            .column_with(TableColumn::new("STATUS").width(ColumnWidth::Fixed(12)))
                            .column_with(
                                TableColumn::new("CPU")
                                    .width(ColumnWidth::Fixed(8))
                                    .align(TextAlign::Right),
                            )
                            .column_with(
                                TableColumn::new("MEMORY")
                                    .width(ColumnWidth::Fixed(10))
                                    .align(TextAlign::Right),
                            )
                            .column_with(
                                TableColumn::new("AGE")
                                    .width(ColumnWidth::Fixed(8))
                                    .align(TextAlign::Right),
                            )
                            .rows(rows)
                            .selected(selected.get())
                            .sort(sort_state.get())
                            .on_select(on_select)
                            .on_sort(on_sort)
                            .on_activate(on_activate)
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::styled_text("↑↓/jk: navigate | Enter: activate | F1 help | Ctrl+Q: quit")
                    .dim()
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 17: Table")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Data table with sortable columns"))
                            .child(View::text("• Row selection and activation"))
                            .child(View::text("• Fixed and flexible column widths"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::table() for tabular data"))
                            .child(View::text("• TableColumn for column config"))
                            .child(View::text("• ColumnWidth::Fixed or ColumnWidth::Flex"))
                            .child(View::text("• on_sort callback for sorting"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Navigate rows with arrow keys"))
                            .child(View::text("• Press Enter to activate a row"))
                            .child(View::text("• Sorting is managed via on_sort"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 18_progress_bar: progress indicators"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
