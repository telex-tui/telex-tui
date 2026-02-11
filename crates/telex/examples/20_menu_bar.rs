//! Example 20: Menu Bar with Keyboard Navigation
//!
//! Demonstrates the menu bar with full keyboard support:
//! - Tab to focus the menu bar
//! - Enter/Space to open a menu
//! - Up/Down arrows to navigate items within a menu
//! - Left/Right arrows to switch between menus
//! - Enter to execute the selected item
//! - Escape to close the menu
//!
//! Run with: `cargo run -p telex-tui --example 20_menu_bar`

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

        // Menu state
        let active_menu = state!(cx, || Option::<usize>::None);
        let highlighted_menu = state!(cx, || 0usize);
        let selected_item = state!(cx, || 0usize);

        // App state
        let message = state!(cx, || "Use Tab to focus menu bar, then Enter to open".to_string());
        let counter = state!(cx, || 0i32);

        // Command handler - executes menu commands
        let handle_command = with!(message, counter, active_menu, selected_item => move |cmd_id: &'static str| {
            let msg = match cmd_id {
                "file.new" => "Created new file".to_string(),
                "file.open" => "Opening file...".to_string(),
                "file.save" => "File saved".to_string(),
                "file.quit" => "Use Ctrl+Q to quit".to_string(),
                "edit.undo" => "Undone".to_string(),
                "edit.redo" => "Redone".to_string(),
                "edit.cut" => "Cut to clipboard".to_string(),
                "edit.copy" => "Copied to clipboard".to_string(),
                "edit.paste" => "Pasted from clipboard".to_string(),
                "counter.increment" => {
                    counter.update(|n| *n += 1);
                    format!("Counter: {}", counter.get())
                }
                "counter.decrement" => {
                    counter.update(|n| *n -= 1);
                    format!("Counter: {}", counter.get())
                }
                "counter.reset" => {
                    counter.set(0);
                    "Counter reset".to_string()
                }
                _ => format!("Unknown: {}", cmd_id),
            };
            message.set(msg);
            // Close menu after executing command
            active_menu.set(None);
            selected_item.set(0);
        });

        // Menu change handler - opens/closes menus
        let on_menu_change = with!(active_menu, highlighted_menu, selected_item => move |idx: usize| {
            if active_menu.get() == Some(idx) {
                // Clicking same menu toggles it closed
                active_menu.set(None);
            } else {
                active_menu.set(Some(idx));
                highlighted_menu.set(idx); // Keep highlight in sync
                selected_item.set(0);
            }
        });

        // Highlight change handler - arrow key navigation when no menu is open
        let on_highlight_change = with!(highlighted_menu => move |idx: usize| {
            highlighted_menu.set(idx);
        });

        // Item change handler - navigates within menu
        let on_item_change = with!(selected_item => move |idx: usize| {
            selected_item.set(idx);
        });

        // Build menus
        let file_menu = Menu::new("File")
            .command_with_shortcut("file.new", "New", "Ctrl+N")
            .command_with_shortcut("file.open", "Open", "Ctrl+O")
            .command_with_shortcut("file.save", "Save", "Ctrl+S")
            .separator()
            .command_with_shortcut("file.quit", "Quit", "Ctrl+Q");

        let edit_menu = Menu::new("Edit")
            .command_with_shortcut("edit.undo", "Undo", "Ctrl+Z")
            .command_with_shortcut("edit.redo", "Redo", "Ctrl+Y")
            .separator()
            .command_with_shortcut("edit.cut", "Cut", "Ctrl+X")
            .command_with_shortcut("edit.copy", "Copy", "Ctrl+C")
            .command_with_shortcut("edit.paste", "Paste", "Ctrl+V");

        let counter_menu = Menu::new("Counter")
            .command("counter.increment", "Increment")
            .command("counter.decrement", "Decrement")
            .separator()
            .command("counter.reset", "Reset to Zero");

        View::vstack()
            .child(
                View::menu_bar()
                    .menu(file_menu)
                    .menu(edit_menu)
                    .menu(counter_menu)
                    .active_menu(active_menu.get())
                    .highlighted_menu(highlighted_menu.get())
                    .selected_item(selected_item.get())
                    .on_select(handle_command)
                    .on_menu_change(on_menu_change)
                    .on_highlight_change(on_highlight_change)
                    .on_item_change(on_item_change)
                    .build(),
            )
            .child(
                View::boxed()
                    .flex(1)
                    .border(true)
                    .padding(2)
                    .child(
                        View::vstack()
                            .spacing(1)
                            .child(View::styled_text("Menu Bar Demo").bold().build())
                            .child(
                                View::styled_text(format!("Counter: {}", counter.get()))
                                    .color(Color::Cyan)
                                    .bold()
                                    .build(),
                            )
                            .child(
                                View::styled_text(format!("Status: {}", message.get()))
                                    .dim()
                                    .build(),
                            )
                            .child(View::spacer())
                            .child(View::styled_text("Keyboard Navigation:").bold().build())
                            .child(View::text("  Tab         Focus menu bar"))
                            .child(View::text("  Enter       Open menu / Execute item"))
                            .child(View::text("  Up/Down     Navigate menu items"))
                            .child(View::text("  Left/Right  Switch between menus"))
                            .child(View::text("  Escape      Close menu"))
                            .child(View::text("  Ctrl+Q      Quit"))
                            .child(View::text("  F1          Help"))
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 20: Menu Bar")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Dropdown menu bar with keyboard nav"))
                            .child(View::text("• Menu items with shortcuts"))
                            .child(View::text("• Separators in menus"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::menu_bar() creates menu system"))
                            .child(View::text("• Menu::new().command() adds items"))
                            .child(View::text("• .command_with_shortcut() shows key hints"))
                            .child(View::text("• on_select receives command ID"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Tab to menu, Enter to open"))
                            .child(View::text("• Arrow keys navigate menus"))
                            .child(View::text("• Try the Counter menu"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 21_toasts: toast notifications"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
