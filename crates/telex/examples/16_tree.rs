//! Example 16: Tree View
//!
//! Demonstrates the Tree widget for hierarchical navigation.
//!
//! Run with: cargo run -p telex-tui --example 16_tree

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
        let show_help = state!(cx, || false);

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        // Track selected path
        let selected = state!(cx, || vec![0usize]);

        // Track expanded state for each node (by path prefix)
        let expanded_paths = state!(cx, || {
            vec![
                vec![0],    // src/ expanded
                vec![0, 0], // src/components/ expanded
            ]
        });

        // Build tree items with current expanded state
        let items = build_tree(&expanded_paths.get());

        let on_select = with!(selected => move |path: TreePath| {
            selected.set(path);
        });

        let on_activate = with!(expanded_paths => move |path: TreePath| {
            // Toggle expand/collapse for the activated item
            let mut paths = expanded_paths.get().clone();
            if let Some(pos) = paths.iter().position(|p| *p == path) {
                // Currently expanded, collapse it
                paths.remove(pos);
            } else {
                // Currently collapsed, expand it
                paths.push(path.clone());
            }
            expanded_paths.set(paths);
        });

        let selected_label = get_item_at_path(&items, &selected.get())
            .map(|item| item.label.clone())
            .unwrap_or_else(|| "Nothing".to_string());

        View::vstack()
            .child(
                View::styled_text("File Browser")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(
                View::styled_text(format!("Selected: {}", selected_label))
                    .dim()
                    .build(),
            )
            .child(
                View::boxed()
                    .flex(1)
                    .border(true)
                    .child(
                        View::tree()
                            .items(items)
                            .selected(selected.get().clone())
                            .on_select(on_select)
                            .on_activate(on_activate)
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::styled_text(
                    "↑↓/jk: navigate | Enter: expand/collapse | F1 help | Ctrl+Q: quit",
                )
                .dim()
                .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 16: Tree View")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Hierarchical tree widget"))
                            .child(View::text("• Expand/collapse folders"))
                            .child(View::text("• Path-based selection tracking"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::tree() for hierarchical data"))
                            .child(View::text("• TreeItem::new().child() builds hierarchy"))
                            .child(View::text("• on_select returns TreePath (Vec<usize>)"))
                            .child(View::text("• on_activate for expand/collapse"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Navigate with arrow keys"))
                            .child(View::text("• Press Enter to expand/collapse folders"))
                            .child(View::text("• Watch the 'Selected:' text update"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 17_table: data tables with sorting"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}

fn build_tree(expanded_paths: &[TreePath]) -> Vec<TreeItem> {
    let is_expanded = |path: &[usize]| expanded_paths.iter().any(|p| p == path);

    vec![
        TreeItem::new("src")
            .icon("📁")
            .expanded(is_expanded(&[0]))
            .child(
                TreeItem::new("components")
                    .icon("📁")
                    .expanded(is_expanded(&[0, 0]))
                    .child(TreeItem::new("button.rs").icon("📄"))
                    .child(TreeItem::new("input.rs").icon("📄"))
                    .child(TreeItem::new("list.rs").icon("📄")),
            )
            .child(
                TreeItem::new("utils")
                    .icon("📁")
                    .expanded(is_expanded(&[0, 1]))
                    .child(TreeItem::new("helpers.rs").icon("📄"))
                    .child(TreeItem::new("macros.rs").icon("📄")),
            )
            .child(TreeItem::new("main.rs").icon("📄"))
            .child(TreeItem::new("lib.rs").icon("📄")),
        TreeItem::new("tests")
            .icon("📁")
            .expanded(is_expanded(&[1]))
            .child(TreeItem::new("integration_tests.rs").icon("📄"))
            .child(TreeItem::new("unit_tests.rs").icon("📄")),
        TreeItem::new("Cargo.toml").icon("📦"),
        TreeItem::new("README.md").icon("📝"),
    ]
}

fn get_item_at_path<'a>(items: &'a [TreeItem], path: &[usize]) -> Option<&'a TreeItem> {
    if path.is_empty() {
        return None;
    }

    let mut current_items = items;
    let mut result = None;

    for &idx in path {
        if idx < current_items.len() {
            result = Some(&current_items[idx]);
            current_items = &current_items[idx].children;
        } else {
            return None;
        }
    }

    result
}
