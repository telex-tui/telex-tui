//! Command system for keyboard shortcuts.
//!
//! Provides `use_command` hook for registering keybindings in components.

use crossterm::event::{KeyCode, KeyModifiers};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A keyboard shortcut definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    /// Create a new key binding.
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// Create a binding for a single key (no modifiers).
    pub fn key(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    /// Create a binding for Ctrl+key.
    pub fn ctrl(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    /// Create a binding for Alt+key.
    pub fn alt(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::ALT)
    }

    /// Create a binding for Shift+key.
    pub fn shift(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::SHIFT)
    }

    /// Check if this binding matches a key event.
    pub fn matches(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.code == code && self.modifiers == modifiers
    }
}

/// Callback type for commands.
pub type CommandCallback = Rc<dyn Fn()>;

/// Registry of active commands.
#[derive(Default)]
pub struct CommandRegistry {
    commands: RefCell<HashMap<KeyBinding, CommandCallback>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a command with a keybinding.
    pub fn register(&self, binding: KeyBinding, callback: CommandCallback) {
        self.commands.borrow_mut().insert(binding, callback);
    }

    /// Clear all registered commands (call before each render).
    pub fn clear(&self) {
        self.commands.borrow_mut().clear();
    }

    /// Try to execute a command matching the given key event.
    /// Returns true if a command was executed.
    pub fn execute(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        let commands = self.commands.borrow();
        for (binding, callback) in commands.iter() {
            if binding.matches(code, modifiers) {
                callback();
                return true;
            }
        }
        false
    }

    /// Check if any command is registered for this key.
    pub fn has_binding(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        let commands = self.commands.borrow();
        commands.keys().any(|b| b.matches(code, modifiers))
    }
}
