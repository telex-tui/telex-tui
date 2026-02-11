//! Unified command system for Command Palette and Menu Bar.
//!
//! Commands are defined once and can be used in both the Command Palette
//! (fuzzy search) and Menu Bar (dropdown menus).

use crate::command::KeyBinding;
use crate::view::Callback;
use std::rc::Rc;

/// A command that can be executed from the Command Palette or Menu Bar.
#[derive(Clone)]
pub struct Command {
    /// Unique identifier for the command (e.g., "file.save").
    pub id: &'static str,
    /// Display label for the command (e.g., "Save File").
    pub label: String,
    /// Optional keyboard shortcut.
    pub shortcut: Option<KeyBinding>,
    /// The action to execute when the command is triggered.
    pub action: Callback,
    /// Optional category for menu grouping (e.g., "File", "Edit").
    pub category: Option<String>,
}

impl Command {
    /// Create a new command builder with the given ID and label.
    pub fn builder(id: &'static str, label: impl Into<String>) -> CommandBuilder {
        CommandBuilder {
            id,
            label: label.into(),
            shortcut: None,
            action: None,
            category: None,
        }
    }

    /// Create a new command with the given ID and label (deprecated, use builder).
    #[deprecated(since = "0.1.0", note = "Use Command::builder instead")]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: &'static str, label: impl Into<String>) -> CommandBuilder {
        Self::builder(id, label)
    }
}

/// Builder for creating commands.
pub struct CommandBuilder {
    id: &'static str,
    label: String,
    shortcut: Option<KeyBinding>,
    action: Option<Callback>,
    category: Option<String>,
}

impl CommandBuilder {
    /// Set the keyboard shortcut for this command.
    pub fn shortcut(mut self, binding: KeyBinding) -> Self {
        self.shortcut = Some(binding);
        self
    }

    /// Set the category for menu grouping.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set the action callback for this command.
    pub fn action(mut self, callback: impl Fn() + 'static) -> Self {
        self.action = Some(Rc::new(callback));
        self
    }

    /// Build the command.
    pub fn build(self) -> Command {
        Command {
            id: self.id,
            label: self.label,
            shortcut: self.shortcut,
            action: self.action.unwrap_or_else(|| Rc::new(|| {})),
            category: self.category,
        }
    }
}

/// A collection of commands.
#[derive(Clone, Default)]
pub struct Commands {
    commands: Vec<Command>,
}

impl Commands {
    /// Create a new empty command collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a command to the collection.
    pub fn with_command(mut self, command: Command) -> Self {
        self.commands.push(command);
        self
    }

    /// Add a command to the collection (deprecated, use with_command).
    #[deprecated(since = "0.1.0", note = "Use with_command instead to avoid confusion with std::ops::Add")]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, command: Command) -> Self {
        self.with_command(command)
    }

    /// Get all commands.
    pub fn all(&self) -> &[Command] {
        &self.commands
    }

    /// Filter commands by a fuzzy search query.
    pub fn filter(&self, query: &str) -> Vec<&Command> {
        if query.is_empty() {
            return self.commands.iter().collect();
        }

        let query_lower = query.to_lowercase();
        let mut matches: Vec<(&Command, i32)> = self
            .commands
            .iter()
            .filter_map(|cmd| {
                let score = fuzzy_score(&cmd.label.to_lowercase(), &query_lower);
                if score > 0 {
                    Some((cmd, score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (higher is better)
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches.into_iter().map(|(cmd, _)| cmd).collect()
    }

    /// Get commands grouped by category.
    pub fn by_category(&self) -> Vec<(String, Vec<&Command>)> {
        let mut categories: std::collections::HashMap<String, Vec<&Command>> =
            std::collections::HashMap::new();

        for cmd in &self.commands {
            let cat = cmd.category.clone().unwrap_or_else(|| "Other".to_string());
            categories.entry(cat).or_default().push(cmd);
        }

        // Sort categories alphabetically, but put "File" first and "Other" last
        let mut result: Vec<(String, Vec<&Command>)> = categories.into_iter().collect();
        result.sort_by(|a, b| {
            match (&a.0[..], &b.0[..]) {
                ("File", "File") => std::cmp::Ordering::Equal,
                ("File", _) => std::cmp::Ordering::Less,
                (_, "File") => std::cmp::Ordering::Greater,
                ("Other", "Other") => std::cmp::Ordering::Equal,
                ("Other", _) => std::cmp::Ordering::Greater,
                (_, "Other") => std::cmp::Ordering::Less,
                _ => a.0.cmp(&b.0),
            }
        });

        result
    }

    /// Find a command by ID.
    pub fn find(&self, id: &str) -> Option<&Command> {
        self.commands.iter().find(|cmd| cmd.id == id)
    }

    /// Execute a command by ID.
    pub fn execute(&self, id: &str) {
        if let Some(cmd) = self.find(id) {
            (cmd.action)();
        }
    }
}

/// A menu item for the menu bar.
#[derive(Clone)]
pub enum MenuItem {
    /// A command reference by ID.
    Command(&'static str),
    /// A submenu with a label and items.
    Submenu(String, Vec<MenuItem>),
    /// A visual separator.
    Separator,
}

impl MenuItem {
    /// Create a command menu item.
    pub fn command(id: &'static str) -> Self {
        MenuItem::Command(id)
    }

    /// Create a submenu.
    pub fn submenu(label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        MenuItem::Submenu(label.into(), items)
    }

    /// Create a separator.
    pub fn separator() -> Self {
        MenuItem::Separator
    }
}

/// Menu bar definition.
#[derive(Clone, Default)]
pub struct MenuDefinition {
    /// Top-level menu items (usually submenus like "File", "Edit", etc.).
    pub items: Vec<MenuItem>,
}

impl MenuDefinition {
    /// Create a new menu definition.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a top-level menu item.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add a menu (submenu with label).
    pub fn menu(self, label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        self.add(MenuItem::Submenu(label.into(), items))
    }
}

/// Simple fuzzy matching score.
/// Returns a positive score if the query matches, 0 otherwise.
fn fuzzy_score(text: &str, query: &str) -> i32 {
    if query.is_empty() {
        return 1;
    }

    let text_chars: Vec<char> = text.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();

    // Try to find all query characters in order in the text
    let mut text_idx = 0;
    let mut query_idx = 0;
    let mut score = 0;
    let mut consecutive = 0;

    while text_idx < text_chars.len() && query_idx < query_chars.len() {
        if text_chars[text_idx] == query_chars[query_idx] {
            // Bonus for consecutive matches
            consecutive += 1;
            score += consecutive * 2;

            // Bonus for matching at word boundaries
            if text_idx == 0 || !text_chars[text_idx - 1].is_alphanumeric() {
                score += 5;
            }

            query_idx += 1;
        } else {
            consecutive = 0;
        }
        text_idx += 1;
    }

    // All query characters must be found
    if query_idx == query_chars.len() {
        score
    } else {
        0
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_score() {
        // Exact match
        assert!(fuzzy_score("save", "save") > 0);

        // Prefix match
        assert!(fuzzy_score("save file", "save") > 0);

        // Fuzzy match
        assert!(fuzzy_score("save file", "sf") > 0);

        // No match
        assert_eq!(fuzzy_score("save", "xyz"), 0);
    }

    #[test]
    fn test_commands_filter() {
        let commands = Commands::new()
            .add(Command::new("file.save", "Save File").build())
            .add(Command::new("file.open", "Open File").build())
            .add(Command::new("edit.undo", "Undo").build());

        let matches = commands.filter("file");
        assert_eq!(matches.len(), 2);

        let matches = commands.filter("save");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "file.save");
    }

    #[test]
    fn test_commands_by_category() {
        let commands = Commands::new()
            .add(Command::new("file.save", "Save").category("File").build())
            .add(Command::new("file.open", "Open").category("File").build())
            .add(Command::new("edit.undo", "Undo").category("Edit").build());

        let cats = commands.by_category();
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].0, "File"); // File comes first
        assert_eq!(cats[0].1.len(), 2);
    }
}
