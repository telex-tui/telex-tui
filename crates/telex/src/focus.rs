//! Focus management for keyboard navigation.
//!
//! Event handlers use nested pattern matching which clippy may flag as collapsible.
//! However, collapsing them reduces readability by mixing focus type checking with
//! callback presence checking in a single pattern.

#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_match)]

use std::collections::{HashMap, HashSet};

use crossterm::event::{KeyCode, KeyModifiers};

use crate::render::ScrollLimit;
use crate::text;
use crate::view::{
    Callback, ChangeCallback, CommandCallback, CursorChangeCallback, CursorPosCallback, Menu,
    PaletteCommand, RowActivateCallback, SelectCallback, SliderCallback, SortCallback,
    ToggleCallback, TreeActivateCallback, TreeItem, TreePath, TreeSelectCallback, View,
};

/// Represents a focusable element in the UI.
#[derive(Clone)]
pub enum Focusable {
    /// A button with an optional callback.
    Button(Option<Callback>),
    /// A scrollable box (scroll state tracked separately in scroll_states).
    Scrollable {
        /// If true, scroll direction is inverted (0 = bottom, scroll_up moves away from bottom)
        auto_scroll_bottom: bool,
    },
    /// A selectable list with items and selection callback.
    List {
        items_count: usize,
        selected: usize,
        on_select: Option<SelectCallback>,
    },
    /// A text input field.
    TextInput {
        value: String,
        cursor_pos: usize,
        on_change: Option<ChangeCallback>,
        on_cursor_change: Option<CursorPosCallback>,
        on_submit: Option<Callback>,
        on_key_up: Option<Callback>,
        on_key_down: Option<Callback>,
    },
    /// A multi-line text area.
    TextArea {
        value: String,
        cursor_line: usize,
        cursor_col: usize,
        on_change: Option<ChangeCallback>,
        on_cursor_change: Option<CursorChangeCallback>,
        /// Width at which to auto-wrap text (None = no wrap).
        wrap_width: Option<u16>,
    },
    /// A checkbox/toggle.
    Checkbox {
        checked: bool,
        on_toggle: Option<ToggleCallback>,
    },
    /// A radio group (mutually exclusive options).
    RadioGroup {
        options_count: usize,
        selected: usize,
        on_change: Option<SelectCallback>,
    },
    /// A tabbed interface.
    Tabs {
        tab_count: usize,
        active: usize,
        on_change: Option<SelectCallback>,
    },
    /// A hierarchical tree view.
    Tree {
        items: Vec<TreeItem>,
        selected: TreePath,
        on_select: Option<TreeSelectCallback>,
        on_activate: Option<TreeActivateCallback>,
    },
    /// A data table with rows and columns.
    Table {
        row_count: usize,
        selected: usize,
        sort: Option<(usize, bool)>,
        on_select: Option<SelectCallback>,
        on_sort: Option<SortCallback>,
        on_activate: Option<RowActivateCallback>,
    },
    /// A command palette for searching and executing commands.
    CommandPalette {
        visible: bool,
        query: String,
        commands: Vec<PaletteCommand>,
        selected: usize,
        on_query_change: Option<ChangeCallback>,
        on_select: Option<CommandCallback>,
        on_dismiss: Option<Callback>,
    },
    /// A menu bar with dropdown menus.
    MenuBar {
        menus: Vec<Menu>,
        active_menu: Option<usize>,
        highlighted_menu: usize,
        selected_item: usize,
        on_select: Option<CommandCallback>,
        on_menu_change: Option<SelectCallback>,
        on_highlight_change: Option<SelectCallback>,
        on_item_change: Option<SelectCallback>,
    },
    /// A form field with validation.
    FormField {
        #[allow(dead_code)] // Stored for debugging/future use
        name: String,
        value: String,
        cursor_pos: usize,
        on_change: Option<ChangeCallback>,
        on_blur: Option<Callback>,
    },
    /// An interactive PTY terminal.
    Terminal {
        handle: crate::terminal_state::TerminalHandle,
        #[allow(dead_code)]
        on_exit: Option<Callback>,
    },
    /// A slider for bounded numeric values.
    Slider {
        min: f64,
        max: f64,
        value: f64,
        step: f64,
        on_change: Option<SliderCallback>,
    },
}

/// Identity of a collected focusable: its `.key()` when given (and not a duplicate),
/// otherwise its position in this frame's focus order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum FocusId {
    Key(String),
    Position(usize),
}

/// Manages focus and scroll state for the application.
///
/// Focus, scroll offsets and cursor positions follow a widget's [`FocusId`]: a keyed
/// widget keeps its state wherever it moves in the tree, and loses it when it leaves the
/// tree; an unkeyed widget's state is kept by position, as it always has been.
#[derive(Default)]
pub struct FocusManager {
    /// Current focus index
    focus_index: usize,
    /// Focusable elements (collected during render)
    focusables: Vec<Focusable>,
    /// Identity of each focusable, aligned with `focusables`
    ids: Vec<FocusId>,
    /// Indices of focusables requesting focus this collection, in tree order
    requests: Vec<usize>,
    /// Ids that requested focus on the previous collection — a request is applied only
    /// when its id wasn't already requesting
    prev_requested: HashSet<FocusId>,
    /// Keys seen more than once in the latest collection (all but the first fall back to
    /// positional identity)
    duplicate_keys: Vec<String>,
    /// Working scroll states, aligned with `focusables` (loaded from the stores below on
    /// each collection, committed back on the next)
    scroll_states: Vec<(u16, u16)>,
    /// Scroll limits measured by the last render, keyed by focusable index
    scroll_limits: Vec<ScrollLimit>,
    /// Working cursor positions for text inputs, aligned with `focusables`
    cursor_states: Vec<usize>,
    /// Scroll state of keyed widgets present in the tree
    keyed_scroll: HashMap<String, (u16, u16)>,
    /// Cursor state of keyed widgets present in the tree
    keyed_cursor: HashMap<String, usize>,
    /// Scroll state of unkeyed widgets, by position (persists across re-collects)
    positional_scroll: Vec<(u16, u16)>,
    /// Cursor state of unkeyed widgets, by position (persists across re-collects)
    positional_cursor: Vec<usize>,
    /// Whether we're currently collecting focusables from within a modal
    in_modal: bool,
    /// Focused widget from before entering a modal (restored when it closes)
    saved_focus_id: Option<FocusId>,
    /// Whether focus styling should be shown (becomes true on first Tab/Shift+Tab, or when
    /// a focus request is applied)
    focus_visible: bool,
}


impl FocusManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current focus index.
    pub fn focus_index(&self) -> usize {
        self.focus_index
    }

    /// Get the total number of focusable elements.
    pub fn focusable_count(&self) -> usize {
        self.focusables.len()
    }

    /// Keys used by more than one focusable in the latest collection.
    pub(crate) fn duplicate_keys(&self) -> &[String] {
        &self.duplicate_keys
    }

    /// Get scroll offset for a focusable at the given index.
    pub fn scroll_offset(&self, index: usize) -> (u16, u16) {
        self.scroll_states.get(index).copied().unwrap_or((0, 0))
    }

    /// Get cursor offset for a text input at the given index.
    /// Returns usize::MAX if not set (use node's value).
    pub fn cursor_offset(&self, index: usize) -> usize {
        self.cursor_states.get(index).copied().unwrap_or(usize::MAX)
    }

    /// Update scroll states from render (e.g., after clamping).
    pub fn update_scroll_states(&mut self, offsets: &[(u16, u16)]) {
        for (i, offset) in offsets.iter().enumerate() {
            if let Some(state) = self.scroll_states.get_mut(i) {
                *state = *offset;
            }
        }
    }

    /// Replace the scroll limits with those measured by the latest render.
    pub(crate) fn update_scroll_limits(&mut self, limits: &[ScrollLimit]) {
        self.scroll_limits = limits.to_vec();
    }

    /// Measured scroll limit of the currently focused element (zero if not scrollable,
    /// or not rendered yet).
    pub(crate) fn focused_scroll_limit(&self) -> ScrollLimit {
        self.scroll_limits.get(self.focus_index).copied().unwrap_or_default()
    }

    /// Collect all focusable elements from the view tree.
    /// When a modal is visible, only collects focusables from within the modal.
    pub fn collect_focusables(&mut self, view: &View) {
        // Save the working state under last collection's ids, and note what was
        // focused, before the list is rebuilt
        self.commit_states();
        let prev_focused = self.ids.get(self.focus_index).cloned();

        self.focusables.clear();
        self.ids.clear();
        self.requests.clear();
        self.duplicate_keys.clear();

        // Check if there's a visible modal - if so, only collect from within it
        let modal_child = Self::find_visible_modal_content(view);
        self.collect_recursive(modal_child.unwrap_or(view));

        let new_request = self
            .requests
            .iter()
            .copied()
            .find(|&i| !self.prev_requested.contains(&self.ids[i]));

        if modal_child.is_some() {
            if !self.in_modal {
                // Entering modal - save focus and move to the modal's requested item
                // (or its first)
                self.saved_focus_id = prev_focused;
                self.in_modal = true;
                match self.requests.first() {
                    Some(&idx) => self.apply_request(idx),
                    None => self.focus_index = 0,
                }
            } else {
                self.follow_or_request(prev_focused, new_request);
            }
        } else if self.in_modal {
            // Leaving modal - restore the saved widget (by key, or else by position)
            self.in_modal = false;
            if let Some(idx) = self.saved_focus_id.take().and_then(|id| self.index_of(&id)) {
                self.focus_index = idx;
            }
        } else {
            self.follow_or_request(prev_focused, new_request);
        }

        self.prev_requested = self.requests.iter().map(|&i| self.ids[i].clone()).collect();

        // Keyed state lives as long as its widget. While a modal is up the rest of the
        // tree isn't collected, so absence then doesn't mean the widget is gone
        if modal_child.is_none() {
            self.prune_keyed_states();
        }
        self.load_states();

        // Ensure focus index is valid
        if !self.focusables.is_empty() && self.focus_index >= self.focusables.len() {
            self.focus_index = 0;
        }
    }

    /// Push a collected focusable with its identity and focus request.
    fn push_focusable(&mut self, key: &Option<String>, focused: bool, focusable: Focusable) {
        let idx = self.focusables.len();
        let id = match key {
            Some(k) if !self.ids.iter().any(|id| matches!(id, FocusId::Key(existing) if existing == k)) => {
                FocusId::Key(k.clone())
            }
            Some(k) => {
                // Duplicate: first in tree order keeps the key, this one is positional
                if !self.duplicate_keys.contains(k) {
                    self.duplicate_keys.push(k.clone());
                }
                FocusId::Position(idx)
            }
            None => FocusId::Position(idx),
        };
        self.focusables.push(focusable);
        self.ids.push(id);
        if focused {
            self.requests.push(idx);
        }
    }

    /// Apply a newly appeared focus request, or else keep focus on the same keyed widget
    /// if it moved.
    fn follow_or_request(&mut self, prev_focused: Option<FocusId>, new_request: Option<usize>) {
        if let Some(idx) = new_request {
            self.apply_request(idx);
        } else if let Some(id @ FocusId::Key(_)) = prev_focused {
            if let Some(idx) = self.index_of(&id) {
                self.focus_index = idx;
            }
        }
    }

    /// Focus moved by the app on purpose is shown, the same as a Tab press.
    fn apply_request(&mut self, idx: usize) {
        self.focus_index = idx;
        self.focus_visible = true;
    }

    /// Current index of a focusable by id: a key wherever it now is, a position if still
    /// in range.
    fn index_of(&self, id: &FocusId) -> Option<usize> {
        match id {
            FocusId::Key(_) => self.ids.iter().position(|other| other == id),
            FocusId::Position(idx) => (*idx < self.focusables.len()).then_some(*idx),
        }
    }

    /// Save the working per-index state into the keyed/positional stores.
    fn commit_states(&mut self) {
        for (i, id) in self.ids.iter().enumerate() {
            let scroll = self.scroll_states.get(i).copied().unwrap_or((0, 0));
            let cursor = self.cursor_states.get(i).copied().unwrap_or(usize::MAX);
            match id {
                FocusId::Key(k) => {
                    self.keyed_scroll.insert(k.clone(), scroll);
                    self.keyed_cursor.insert(k.clone(), cursor);
                }
                FocusId::Position(p) => {
                    if self.positional_scroll.len() <= *p {
                        self.positional_scroll.resize(*p + 1, (0, 0));
                        self.positional_cursor.resize(*p + 1, usize::MAX);
                    }
                    self.positional_scroll[*p] = scroll;
                    self.positional_cursor[*p] = cursor;
                }
            }
        }
    }

    /// Drop the state of keyed widgets no longer in the tree.
    fn prune_keyed_states(&mut self) {
        let present: HashSet<&str> = self
            .ids
            .iter()
            .filter_map(|id| match id {
                FocusId::Key(k) => Some(k.as_str()),
                FocusId::Position(_) => None,
            })
            .collect();
        self.keyed_scroll.retain(|k, _| present.contains(k.as_str()));
        self.keyed_cursor.retain(|k, _| present.contains(k.as_str()));
    }

    /// Rebuild the working per-index state from the stores for the new focus list.
    fn load_states(&mut self) {
        // MAX means "use value from view"
        self.scroll_states = self
            .ids
            .iter()
            .map(|id| match id {
                FocusId::Key(k) => self.keyed_scroll.get(k).copied().unwrap_or((0, 0)),
                FocusId::Position(p) => self.positional_scroll.get(*p).copied().unwrap_or((0, 0)),
            })
            .collect();
        self.cursor_states = self
            .ids
            .iter()
            .map(|id| match id {
                FocusId::Key(k) => self.keyed_cursor.get(k).copied().unwrap_or(usize::MAX),
                FocusId::Position(p) => self.positional_cursor.get(*p).copied().unwrap_or(usize::MAX),
            })
            .collect();
    }

    /// Find the content of a visible modal in the view tree.
    /// Returns the modal's child view if a visible modal is found.
    fn find_visible_modal_content(view: &View) -> Option<&View> {
        match view {
            View::Modal(node) => {
                if node.visible {
                    node.child.as_deref()
                } else {
                    None
                }
            }
            View::VStack(node) => {
                for child in &node.children {
                    if let Some(content) = Self::find_visible_modal_content(child) {
                        return Some(content);
                    }
                }
                None
            }
            View::HStack(node) => {
                for child in &node.children {
                    if let Some(content) = Self::find_visible_modal_content(child) {
                        return Some(content);
                    }
                }
                None
            }
            View::Box(node) => {
                if let Some(child) = &node.child {
                    Self::find_visible_modal_content(child)
                } else {
                    None
                }
            }
            View::Split(node) => Self::find_visible_modal_content(&node.first)
                .or_else(|| Self::find_visible_modal_content(&node.second)),
            View::Tabs(node) => {
                for child in &node.children {
                    if let Some(content) = Self::find_visible_modal_content(child) {
                        return Some(content);
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Collect focusables recursively, recording each one's key and focus request.
    fn collect_recursive(&mut self, view: &View) {
        match view {
            View::Button(btn) => {
                self.push_focusable(&btn.key, btn.focused, Focusable::Button(btn.on_press.clone()));
            }
            View::Box(node) => {
                // If scrollable and focusable, add as focusable first
                let is_scrollable = node.scroll || node.auto_scroll_bottom;
                if is_scrollable && node.focusable {
                    self.push_focusable(&node.key, node.focused, Focusable::Scrollable {
                        auto_scroll_bottom: node.auto_scroll_bottom,
                    });
                }
                // Then recurse into children
                if let Some(child) = &node.child {
                    self.collect_recursive(child);
                }
            }
            View::VStack(node) => {
                for child in &node.children {
                    self.collect_recursive(child);
                }
            }
            View::HStack(node) => {
                for child in &node.children {
                    self.collect_recursive(child);
                }
            }
            View::List(node) => {
                self.push_focusable(&node.key, node.focused, Focusable::List {
                    items_count: node.items.len(),
                    selected: node.selected,
                    on_select: node.on_select.clone(),
                });
            }
            View::TextInput(node) => {
                self.push_focusable(&node.key, node.focused, Focusable::TextInput {
                    value: node.value.clone(),
                    cursor_pos: node.cursor_pos,
                    on_change: node.on_change.clone(),
                    on_cursor_change: node.on_cursor_change.clone(),
                    on_submit: node.on_submit.clone(),
                    on_key_up: node.on_key_up.clone(),
                    on_key_down: node.on_key_down.clone(),
                });
            }
            View::TextArea(node) => {
                self.push_focusable(&node.key, node.focused, Focusable::TextArea {
                    value: node.value.clone(),
                    cursor_line: node.cursor_line,
                    cursor_col: node.cursor_col,
                    on_change: node.on_change.clone(),
                    on_cursor_change: node.on_cursor_change.clone(),
                    wrap_width: node.wrap_width,
                });
            }
            View::Checkbox(node) => {
                self.push_focusable(&node.key, node.focused, Focusable::Checkbox {
                    checked: node.checked,
                    on_toggle: node.on_toggle.clone(),
                });
            }
            View::RadioGroup(node) => {
                self.push_focusable(&node.key, node.focused, Focusable::RadioGroup {
                    options_count: node.options.len(),
                    selected: node.selected,
                    on_change: node.on_change.clone(),
                });
            }
            View::Modal(node) => {
                // Only collect focusables from visible modals
                if node.visible {
                    if let Some(child) = &node.child {
                        self.collect_recursive(child);
                    }
                }
            }
            View::Split(node) => {
                // Split is a layout container - recurse into both panes
                self.collect_recursive(&node.first);
                self.collect_recursive(&node.second);
            }
            View::Tabs(node) => {
                // Tabs itself is focusable for tab switching
                self.push_focusable(&node.key, node.focused, Focusable::Tabs {
                    tab_count: node.tabs.len(),
                    active: node.active,
                    on_change: node.on_change.clone(),
                });
                // Then recurse into the active tab's content
                if node.active < node.children.len() {
                    self.collect_recursive(&node.children[node.active]);
                }
            }
            View::Tree(node) => {
                self.push_focusable(&node.key, node.focused, Focusable::Tree {
                    items: node.items.clone(),
                    selected: node.selected.clone(),
                    on_select: node.on_select.clone(),
                    on_activate: node.on_activate.clone(),
                });
            }
            View::Table(node) => {
                self.push_focusable(&node.key, node.focused, Focusable::Table {
                    row_count: node.rows.len(),
                    selected: node.selected,
                    sort: node.sort,
                    on_select: node.on_select.clone(),
                    on_sort: node.on_sort.clone(),
                    on_activate: node.on_activate.clone(),
                });
            }
            View::CommandPalette(node) => {
                // Only add as focusable if visible
                if node.visible {
                    self.push_focusable(&node.key, node.focused, Focusable::CommandPalette {
                        visible: node.visible,
                        query: node.query.clone(),
                        commands: node.commands.clone(),
                        selected: node.selected,
                        on_query_change: node.on_query_change.clone(),
                        on_select: node.on_select.clone(),
                        on_dismiss: node.on_dismiss.clone(),
                    });
                }
            }
            View::MenuBar(node) => {
                self.push_focusable(&node.key, node.focused, Focusable::MenuBar {
                    menus: node.menus.clone(),
                    active_menu: node.active_menu,
                    highlighted_menu: node.highlighted_menu,
                    selected_item: node.selected_item,
                    on_select: node.on_select.clone(),
                    on_menu_change: node.on_menu_change.clone(),
                    on_highlight_change: node.on_highlight_change.clone(),
                    on_item_change: node.on_item_change.clone(),
                });
            }
            View::Form(node) => {
                // Form is a container - recurse into children
                for child in &node.children {
                    self.collect_recursive(child);
                }
            }
            View::FormField(node) => {
                self.push_focusable(&node.key, node.focused, Focusable::FormField {
                    name: node.name.clone(),
                    value: node.value.clone(),
                    cursor_pos: node.cursor_pos,
                    on_change: node.on_change.clone(),
                    on_blur: node.on_blur.clone(),
                });
            }
            View::Terminal(node) => {
                self.push_focusable(&node.key, node.focused, Focusable::Terminal {
                    handle: node.handle.clone(),
                    on_exit: node.on_exit.clone(),
                });
            }
            View::ErrorBoundary(node) => {
                // Collect focusables from the child view
                self.collect_recursive(&node.child);
            }
            View::Custom(_) => {
                // Custom widgets are not currently focusable through the
                // focus system (they'd need their own Focusable variant).
                // For now, skip. Users can wrap in a Box with scroll for focus.
            }
            View::Slider(node) => {
                self.push_focusable(&node.key, node.focused, Focusable::Slider {
                    min: node.min,
                    max: node.max,
                    value: node.value,
                    step: node.step,
                    on_change: node.on_change.clone(),
                });
            }
            View::Text(_)
            | View::Spacer(_)
            | View::ProgressBar(_)
            | View::StatusBar(_)
            | View::ToastContainer(_)
            | View::Canvas(_)
            | View::Image(_)
            | View::Empty => {}
        }
    }

    /// Move focus to the next focusable element.
    pub fn focus_next(&mut self) {
        if self.focusables.is_empty() {
            return;
        }
        if !self.focus_visible {
            // First Tab just reveals focus on the current element
            self.focus_visible = true;
            return;
        }
        self.focus_index = (self.focus_index + 1) % self.focusables.len();
    }

    /// Move focus to the previous focusable element.
    pub fn focus_prev(&mut self) {
        if self.focusables.is_empty() {
            return;
        }
        if !self.focus_visible {
            // First Shift+Tab just reveals focus on the current element
            self.focus_visible = true;
            return;
        }
        if self.focus_index == 0 {
            self.focus_index = self.focusables.len() - 1;
        } else {
            self.focus_index -= 1;
        }
    }

    /// Check if focus styling should be visible (user has started keyboard navigation).
    pub fn is_focus_visible(&self) -> bool {
        self.focus_visible
    }

    /// Whether the focused widget handles this key itself, so it should get the key before
    /// app-registered commands. Mirrors the run loop's widget dispatch: only keys a widget
    /// actually acts on count, so anything else still reaches the app's commands.
    /// Ctrl/Alt chords are decided by the caller (commands always see them first).
    pub(crate) fn focused_handles_key(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        use KeyCode::*;
        let plain = modifiers == KeyModifiers::NONE;
        let shifted_char = modifiers == KeyModifiers::SHIFT && matches!(code, Char(_));
        let activate = plain && matches!(code, Enter | Char(' '));
        match self.focusables.get(self.focus_index) {
            Some(Focusable::Button(on_press)) => activate && on_press.is_some(),
            Some(Focusable::Checkbox { on_toggle, .. }) => activate && on_toggle.is_some(),
            Some(Focusable::TextInput { on_submit, on_key_up, on_key_down, .. }) => {
                shifted_char
                    || (plain
                        && match code {
                            Char(_) | Backspace | Left | Right => true,
                            Up => on_key_up.is_some(),
                            Down => on_key_down.is_some(),
                            Enter => on_submit.is_some(),
                            _ => false,
                        })
            }
            Some(Focusable::TextArea { .. }) => {
                shifted_char || (plain && matches!(code, Char(_) | Enter | Backspace | Up | Down | Left | Right))
            }
            // Space reaches the Enter/Space arm, which doesn't type into form fields
            Some(Focusable::FormField { .. }) => {
                shifted_char || (plain && (matches!(code, Backspace) || matches!(code, Char(c) if c != ' ')))
            }
            Some(Focusable::List { .. }) => plain && matches!(code, Up | Down),
            Some(Focusable::RadioGroup { .. }) => plain && matches!(code, Up | Down | Char('j') | Char('k')),
            Some(Focusable::Tabs { .. }) => {
                plain && matches!(code, Left | Right | Char('[') | Char(']') | Char('1'..='9'))
            }
            Some(Focusable::Tree { on_activate, .. }) => {
                (plain && matches!(code, Up | Down | Left | Right | Home | End | Char('j') | Char('k') | Char('g')))
                    || (modifiers == KeyModifiers::SHIFT && matches!(code, Char('G') | Char('g')))
                    || (activate && on_activate.is_some())
            }
            Some(Focusable::Table { on_activate, .. }) => {
                (plain && matches!(code, Up | Down | Char('j') | Char('k'))) || (activate && on_activate.is_some())
            }
            Some(Focusable::Slider { .. }) => plain && matches!(code, Left | Right),
            Some(Focusable::Scrollable { .. }) => plain && matches!(code, Up | Down | PageUp | PageDown | Home | End),
            Some(Focusable::MenuBar { .. }) => {
                (plain && matches!(code, Left | Right)) || activate || (plain && matches!(code, Up | Down) && self.menu_bar_has_open_menu())
            }
            Some(Focusable::Terminal { .. }) => true,
            // The command palette takes its keys before commands already (run loop)
            Some(Focusable::CommandPalette { .. }) | None => false,
        }
    }

    /// Activate the currently focused element (for buttons and checkboxes).
    pub fn activate(&self) {
        match self.focusables.get(self.focus_index) {
            Some(Focusable::Button(Some(callback))) => {
                callback();
            }
            Some(Focusable::Checkbox { checked, on_toggle: Some(callback) }) => {
                callback(!checked);
            }
            _ => {}
        }
    }

    /// Check if the currently focused element is scrollable.
    pub fn is_focused_scrollable(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::Scrollable { .. })
        )
    }

    /// Check if the focused scrollable has auto_scroll_bottom (inverted scroll direction).
    pub fn is_focused_auto_scroll_bottom(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::Scrollable {
                auto_scroll_bottom: true
            })
        )
    }

    /// Check if the currently focused element is a list.
    pub fn is_focused_list(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::List { .. })
        )
    }

    /// Check if the currently focused element is a text input.
    pub fn is_focused_text_input(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::TextInput { .. })
        )
    }

    /// Check if the currently focused element is a text area.
    pub fn is_focused_text_area(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::TextArea { .. })
        )
    }

    /// Check if the currently focused element is a tabs widget.
    pub fn is_focused_tabs(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::Tabs { .. })
        )
    }

    /// Check if the currently focused element is a tree.
    pub fn is_focused_tree(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::Tree { .. })
        )
    }

    /// Check if the currently focused element is a table.
    pub fn is_focused_table(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::Table { .. })
        )
    }

    /// Update wrap_width for all TextArea focusables that don't have an explicit width set.
    /// This allows auto-wrap based on terminal width.
    /// Pass the content width (terminal width minus borders/margins).
    pub fn set_default_textarea_wrap_width(&mut self, width: u16) {
        for focusable in &mut self.focusables {
            if let Focusable::TextArea { wrap_width, .. } = focusable {
                if wrap_width.is_none() {
                    *wrap_width = Some(width);
                }
            }
        }
    }

    /// Get the type name of the currently focused element (for debugging).
    #[allow(dead_code)]
    pub fn focused_type_name(&self) -> &'static str {
        match self.focusables.get(self.focus_index) {
            Some(Focusable::Button(_)) => "Button",
            Some(Focusable::Scrollable { .. }) => "Scrollable",
            Some(Focusable::List { .. }) => "List",
            Some(Focusable::TextInput { .. }) => "TextInput",
            Some(Focusable::TextArea { .. }) => "TextArea",
            Some(Focusable::Checkbox { .. }) => "Checkbox",
            Some(Focusable::RadioGroup { .. }) => "RadioGroup",
            Some(Focusable::Tabs { .. }) => "Tabs",
            Some(Focusable::Tree { .. }) => "Tree",
            Some(Focusable::Table { .. }) => "Table",
            Some(Focusable::CommandPalette { .. }) => "CommandPalette",
            Some(Focusable::MenuBar { .. }) => "MenuBar",
            Some(Focusable::FormField { .. }) => "FormField",
            Some(Focusable::Terminal { .. }) => "Terminal",
            Some(Focusable::Slider { .. }) => "Slider",
            None => "None",
        }
    }

    /// Check if the currently focused element is a form field.
    pub fn is_focused_form_field(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::FormField { .. })
        )
    }

    /// Check if the currently focused element is a command palette.
    pub fn is_focused_command_palette(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::CommandPalette { .. })
        )
    }

    /// Check if the currently focused element is a menu bar.
    pub fn is_focused_menu_bar(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::MenuBar { .. })
        )
    }

    /// Check if any command palette is visible in the focusables.
    #[allow(dead_code)]
    pub fn has_visible_command_palette(&self) -> bool {
        self.focusables
            .iter()
            .any(|f| matches!(f, Focusable::CommandPalette { visible: true, .. }))
    }

    /// Check if any menu bar has an open dropdown.
    #[allow(dead_code)]
    pub fn has_open_menu(&self) -> bool {
        self.focusables.iter().any(|f| {
            matches!(
                f,
                Focusable::MenuBar {
                    active_menu: Some(_),
                    ..
                }
            )
        })
    }

    /// Move list selection up.
    pub fn list_select_prev(&self) {
        if let Some(Focusable::List {
            items_count,
            selected,
            on_select,
        }) = self.focusables.get(self.focus_index)
        {
            if *items_count > 0 {
                let new_selected = if *selected == 0 {
                    items_count - 1
                } else {
                    selected - 1
                };
                if let Some(callback) = on_select {
                    callback(new_selected);
                }
            }
        }
    }

    /// Move list selection down.
    pub fn list_select_next(&self) {
        if let Some(Focusable::List {
            items_count,
            selected,
            on_select,
        }) = self.focusables.get(self.focus_index)
        {
            if *items_count > 0 {
                let new_selected = (selected + 1) % items_count;
                if let Some(callback) = on_select {
                    callback(new_selected);
                }
            }
        }
    }

    /// Switch to the previous tab.
    pub fn tabs_select_prev(&self) {
        if let Some(Focusable::Tabs {
            tab_count,
            active,
            on_change,
        }) = self.focusables.get(self.focus_index)
        {
            if *tab_count > 0 {
                let new_active = if *active == 0 {
                    tab_count - 1
                } else {
                    active - 1
                };
                if let Some(callback) = on_change {
                    callback(new_active);
                }
            }
        }
    }

    /// Switch to the next tab.
    pub fn tabs_select_next(&self) {
        if let Some(Focusable::Tabs {
            tab_count,
            active,
            on_change,
        }) = self.focusables.get(self.focus_index)
        {
            if *tab_count > 0 {
                let new_active = (active + 1) % tab_count;
                if let Some(callback) = on_change {
                    callback(new_active);
                }
            }
        }
    }

    /// Switch to a specific tab by index.
    pub fn tabs_select(&self, index: usize) {
        if let Some(Focusable::Tabs {
            tab_count,
            on_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if index < *tab_count {
                if let Some(callback) = on_change {
                    callback(index);
                }
            }
        }
    }

    /// Move radio group selection up (previous option).
    pub fn radio_group_select_prev(&self) {
        if let Some(Focusable::RadioGroup {
            options_count,
            selected,
            on_change,
        }) = self.focusables.get(self.focus_index)
        {
            if *options_count > 0 {
                let new_selected = if *selected == 0 {
                    options_count - 1
                } else {
                    selected - 1
                };
                if let Some(callback) = on_change {
                    callback(new_selected);
                }
            }
        }
    }

    /// Move radio group selection down (next option).
    pub fn radio_group_select_next(&self) {
        if let Some(Focusable::RadioGroup {
            options_count,
            selected,
            on_change,
        }) = self.focusables.get(self.focus_index)
        {
            if *options_count > 0 {
                let new_selected = (selected + 1) % options_count;
                if let Some(callback) = on_change {
                    callback(new_selected);
                }
            }
        }
    }

    /// Check if the focused element is a radio group.
    pub fn is_focused_radio_group(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::RadioGroup { .. })
        )
    }

    /// Handle text input for the focused text input.
    pub fn text_input_key(&mut self, key: char) {
        if let Some(Focusable::TextInput {
            value,
            cursor_pos,
            on_change,
            on_cursor_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            // Use stored cursor state if available
            let current_pos = self.cursor_states.get(self.focus_index)
                .filter(|&&p| p != usize::MAX)
                .copied()
                .unwrap_or(*cursor_pos);

            // Insert at grapheme position (grapheme-aware)
            let grapheme_count = text::grapheme_count(value);
            let pos = current_pos.min(grapheme_count);
            let new_value = text::insert_at_grapheme(value, pos, &key.to_string());
            let new_pos = pos + 1;

            // Update internal cursor state
            if let Some(state) = self.cursor_states.get_mut(self.focus_index) {
                *state = new_pos;
            }

            if let Some(callback) = on_change {
                callback(new_value);
            }
            // Call cursor callback if provided
            if let Some(cursor_cb) = on_cursor_change {
                cursor_cb(new_pos);
            }
        }
    }

    /// Handle backspace for the focused text input.
    pub fn text_input_backspace(&mut self) {
        if let Some(Focusable::TextInput {
            value,
            cursor_pos,
            on_change,
            on_cursor_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            // Use stored cursor state if available
            let current_pos = self.cursor_states.get(self.focus_index)
                .filter(|&&p| p != usize::MAX)
                .copied()
                .unwrap_or(*cursor_pos);

            // Remove grapheme before cursor (grapheme-aware)
            let grapheme_count = text::grapheme_count(value);
            let pos = current_pos.min(grapheme_count);
            if pos > 0 {
                if let Some(new_value) = text::remove_at_grapheme(value, pos - 1) {
                    let new_pos = pos - 1;

                    // Update internal cursor state
                    if let Some(state) = self.cursor_states.get_mut(self.focus_index) {
                        *state = new_pos;
                    }

                    if let Some(callback) = on_change {
                        callback(new_value);
                    }
                    // Call cursor callback if provided
                    if let Some(cursor_cb) = on_cursor_change {
                        cursor_cb(new_pos);
                    }
                }
            }
        }
    }

    /// Handle Enter key for the focused text input (submit).
    pub fn text_input_submit(&self) {
        if let Some(Focusable::TextInput { on_submit: Some(callback), .. }) = self.focusables.get(self.focus_index)
        {
            callback();
        }
    }

    /// Handle Up arrow key for the focused text input.
    pub fn text_input_key_up(&self) {
        if let Some(Focusable::TextInput { on_key_up: Some(callback), .. }) = self.focusables.get(self.focus_index)
        {
            callback();
        }
    }

    /// Handle Down arrow key for the focused text input.
    pub fn text_input_key_down(&self) {
        if let Some(Focusable::TextInput { on_key_down: Some(callback), .. }) = self.focusables.get(self.focus_index)
        {
            callback();
        }
    }


    /// Move cursor left in text input.
    pub fn text_input_cursor_left(&mut self) {
        if let Some(Focusable::TextInput {
            cursor_pos,
            on_cursor_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            // Use stored cursor state if available, otherwise use view's cursor_pos
            let current_pos = self.cursor_states.get(self.focus_index)
                .filter(|&&p| p != usize::MAX)
                .copied()
                .unwrap_or(*cursor_pos);

            if current_pos > 0 {
                let new_pos = current_pos - 1;
                // Update internal state
                if let Some(state) = self.cursor_states.get_mut(self.focus_index) {
                    *state = new_pos;
                }
                // Call callback if provided
                if let Some(callback) = on_cursor_change {
                    callback(new_pos);
                }
            }
        }
    }

    /// Move cursor right in text input.
    pub fn text_input_cursor_right(&mut self) {
        if let Some(Focusable::TextInput {
            value,
            cursor_pos,
            on_cursor_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            let max_pos = text::grapheme_count(value);
            // Use stored cursor state if available, otherwise use view's cursor_pos
            let current_pos = self.cursor_states.get(self.focus_index)
                .filter(|&&p| p != usize::MAX)
                .copied()
                .unwrap_or(*cursor_pos);

            if current_pos < max_pos {
                let new_pos = current_pos + 1;
                // Update internal state
                if let Some(state) = self.cursor_states.get_mut(self.focus_index) {
                    *state = new_pos;
                }
                // Call callback if provided
                if let Some(callback) = on_cursor_change {
                    callback(new_pos);
                }
            }
        }
    }

    /// Scroll the currently focused element up.
    pub fn scroll_up(&mut self, amount: u16) {
        if let Some((scroll_y, _)) = self.scroll_states.get_mut(self.focus_index) {
            *scroll_y = scroll_y.saturating_sub(amount);
        }
    }

    /// Scroll the currently focused element down.
    pub fn scroll_down(&mut self, amount: u16, max_scroll: u16) {
        if let Some((scroll_y, _)) = self.scroll_states.get_mut(self.focus_index) {
            *scroll_y = (*scroll_y + amount).min(max_scroll);
        }
    }

    /// Scroll the currently focused element left.
    #[allow(dead_code)]
    pub fn scroll_left(&mut self, amount: u16) {
        if let Some((_, scroll_x)) = self.scroll_states.get_mut(self.focus_index) {
            *scroll_x = scroll_x.saturating_sub(amount);
        }
    }

    /// Scroll the currently focused element right.
    #[allow(dead_code)]
    pub fn scroll_right(&mut self, amount: u16, max_scroll: u16) {
        if let Some((_, scroll_x)) = self.scroll_states.get_mut(self.focus_index) {
            *scroll_x = (*scroll_x + amount).min(max_scroll);
        }
    }

    /// Scroll to the top of the currently focused element.
    pub fn scroll_home(&mut self) {
        if let Some((scroll_y, _)) = self.scroll_states.get_mut(self.focus_index) {
            *scroll_y = 0;
        }
    }

    /// Scroll to the bottom of the currently focused element.
    pub fn scroll_end(&mut self, max_scroll: u16) {
        if let Some((scroll_y, _)) = self.scroll_states.get_mut(self.focus_index) {
            *scroll_y = max_scroll;
        }
    }

    /// Split text into lines, preserving trailing empty lines.
    fn split_lines(text: &str) -> Vec<String> {
        if text.is_empty() {
            vec![String::new()]
        } else {
            let mut lines: Vec<String> = text.lines().map(String::from).collect();
            // Preserve trailing newline as empty line
            if text.ends_with('\n') {
                lines.push(String::new());
            }
            if lines.is_empty() {
                lines.push(String::new());
            }
            lines
        }
    }

    /// Handle text input for the focused text area.
    pub fn text_area_key(&self, key: char) {
        if let Some(Focusable::TextArea {
            value,
            cursor_line,
            cursor_col,
            on_change,
            on_cursor_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_change {
                let mut lines = Self::split_lines(value);

                // Insert character at cursor position (grapheme-aware)
                let (new_line, new_col) = if *cursor_line < lines.len() {
                    let line = &lines[*cursor_line];
                    let grapheme_count = text::grapheme_count(line);
                    let col = (*cursor_col).min(grapheme_count);
                    // Insert at grapheme position
                    let new_line_content = text::insert_at_grapheme(line, col, &key.to_string());
                    lines[*cursor_line] = new_line_content;
                    (*cursor_line, col + 1)
                } else {
                    (*cursor_line, *cursor_col)
                };

                // Content is sacred - only user intent (Enter key) creates newlines.
                // Visual wrapping is handled by the renderer via soft_wrap().

                let new_value = lines.join("\n");
                callback(new_value);

                // Report new cursor position
                if let Some(cursor_cb) = on_cursor_change {
                    cursor_cb(new_line, new_col);
                }
            }
        }
    }

    /// Handle Enter key in text area (insert new line).
    pub fn text_area_enter(&self) {
        if let Some(Focusable::TextArea {
            value,
            cursor_line,
            cursor_col,
            on_change,
            on_cursor_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_change {
                let mut lines = Self::split_lines(value);

                // Split line at cursor position (grapheme-aware)
                if *cursor_line < lines.len() {
                    let line = &lines[*cursor_line];
                    let gc = text::grapheme_count(line);
                    let col = (*cursor_col).min(gc);
                    // Split at grapheme boundary
                    let byte_offset =
                        text::grapheme_to_byte_offset(line, col).unwrap_or(line.len());
                    // Create owned strings before modifying lines
                    let before = line[..byte_offset].to_string();
                    let after = line[byte_offset..].to_string();
                    lines[*cursor_line] = before;
                    lines.insert(cursor_line + 1, after);
                }

                callback(lines.join("\n"));

                // Move cursor to beginning of new line
                if let Some(cursor_cb) = on_cursor_change {
                    cursor_cb(*cursor_line + 1, 0);
                }
            }
        }
    }

    /// Handle backspace in text area.
    pub fn text_area_backspace(&self) {
        if let Some(Focusable::TextArea {
            value,
            cursor_line,
            cursor_col,
            on_change,
            on_cursor_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_change {
                let mut lines = Self::split_lines(value);
                if lines.is_empty() || (lines.len() == 1 && lines[0].is_empty()) {
                    return;
                }

                let (new_line, new_col) = if *cursor_col > 0 && *cursor_line < lines.len() {
                    // Delete grapheme before cursor (grapheme-aware)
                    let line = &lines[*cursor_line];
                    let grapheme_count = text::grapheme_count(line);
                    let col = (*cursor_col).min(grapheme_count);
                    if col > 0 {
                        if let Some(new_line_content) = text::remove_at_grapheme(line, col - 1) {
                            lines[*cursor_line] = new_line_content;
                        }
                    }
                    (*cursor_line, col.saturating_sub(1))
                } else if *cursor_line > 0 {
                    // Join with previous line
                    let prev_line_graphemes = text::grapheme_count(&lines[cursor_line - 1]);
                    let current_line = lines.remove(*cursor_line);
                    lines[cursor_line - 1].push_str(&current_line);
                    (*cursor_line - 1, prev_line_graphemes)
                } else {
                    (*cursor_line, *cursor_col)
                };

                callback(lines.join("\n"));

                // Report new cursor position
                if let Some(cursor_cb) = on_cursor_change {
                    cursor_cb(new_line, new_col);
                }
            }
        }
    }

    /// Move cursor up one line in text area.
    pub fn text_area_cursor_up(&self) {
        if let Some(Focusable::TextArea {
            value,
            cursor_line,
            cursor_col,
            on_cursor_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if *cursor_line > 0 {
                let lines = Self::split_lines(value);
                let new_line = cursor_line - 1;
                // Try to keep same column, but clamp to line length
                let target_line_len = lines
                    .get(new_line)
                    .map(|l| text::grapheme_count(l))
                    .unwrap_or(0);
                let new_col = (*cursor_col).min(target_line_len);

                if let Some(cursor_cb) = on_cursor_change {
                    cursor_cb(new_line, new_col);
                }
            }
        }
    }

    /// Move cursor down one line in text area.
    pub fn text_area_cursor_down(&self) {
        if let Some(Focusable::TextArea {
            value,
            cursor_line,
            cursor_col,
            on_cursor_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            let lines = Self::split_lines(value);
            if *cursor_line + 1 < lines.len() {
                let new_line = cursor_line + 1;
                // Try to keep same column, but clamp to line length
                let target_line_len = lines
                    .get(new_line)
                    .map(|l| text::grapheme_count(l))
                    .unwrap_or(0);
                let new_col = (*cursor_col).min(target_line_len);

                if let Some(cursor_cb) = on_cursor_change {
                    cursor_cb(new_line, new_col);
                }
            }
        }
    }

    /// Move cursor left one position in text area.
    pub fn text_area_cursor_left(&self) {
        if let Some(Focusable::TextArea {
            value,
            cursor_line,
            cursor_col,
            on_cursor_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            let (new_line, new_col) = if *cursor_col > 0 {
                // Move left within current line
                (*cursor_line, cursor_col - 1)
            } else if *cursor_line > 0 {
                // Wrap to end of previous line
                let lines = Self::split_lines(value);
                let prev_line = cursor_line - 1;
                let prev_line_len = lines
                    .get(prev_line)
                    .map(|l| text::grapheme_count(l))
                    .unwrap_or(0);
                (prev_line, prev_line_len)
            } else {
                // Already at start
                (*cursor_line, *cursor_col)
            };

            if let Some(cursor_cb) = on_cursor_change {
                cursor_cb(new_line, new_col);
            }
        }
    }

    /// Move cursor right one position in text area.
    pub fn text_area_cursor_right(&self) {
        if let Some(Focusable::TextArea {
            value,
            cursor_line,
            cursor_col,
            on_cursor_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            let lines = Self::split_lines(value);
            let current_line_len = lines
                .get(*cursor_line)
                .map(|l| text::grapheme_count(l))
                .unwrap_or(0);

            let (new_line, new_col) = if *cursor_col < current_line_len {
                // Move right within current line
                (*cursor_line, cursor_col + 1)
            } else if *cursor_line + 1 < lines.len() {
                // Wrap to start of next line
                (cursor_line + 1, 0)
            } else {
                // Already at end
                (*cursor_line, *cursor_col)
            };

            if let Some(cursor_cb) = on_cursor_change {
                cursor_cb(new_line, new_col);
            }
        }
    }

    // ========== Tree Navigation ==========

    /// Get visible items in a tree as (path, depth, item) tuples.
    fn flatten_tree<'a>(
        items: &'a [TreeItem],
        base_path: &[usize],
    ) -> Vec<(TreePath, usize, &'a TreeItem)> {
        let mut result = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let mut path = base_path.to_vec();
            path.push(i);
            let depth = path.len() - 1;
            result.push((path.clone(), depth, item));
            if item.expanded && !item.children.is_empty() {
                result.extend(Self::flatten_tree(&item.children, &path));
            }
        }
        result
    }

    /// Move tree selection to previous visible item.
    pub fn tree_select_prev(&self) {
        if let Some(Focusable::Tree {
            items,
            selected,
            on_select,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            let visible = Self::flatten_tree(items, &[]);
            if visible.is_empty() {
                return;
            }

            // Find current position in visible list
            let current_idx = visible
                .iter()
                .position(|(path, _, _)| path == selected)
                .unwrap_or(0);

            let new_idx = if current_idx == 0 {
                visible.len() - 1
            } else {
                current_idx - 1
            };

            if let Some(callback) = on_select {
                callback(visible[new_idx].0.clone());
            }
        }
    }

    /// Move tree selection to next visible item.
    pub fn tree_select_next(&self) {
        if let Some(Focusable::Tree {
            items,
            selected,
            on_select,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            let visible = Self::flatten_tree(items, &[]);
            if visible.is_empty() {
                return;
            }

            // Find current position in visible list
            let current_idx = visible
                .iter()
                .position(|(path, _, _)| path == selected)
                .unwrap_or(0);

            let new_idx = if current_idx >= visible.len() - 1 {
                0
            } else {
                current_idx + 1
            };

            if let Some(callback) = on_select {
                callback(visible[new_idx].0.clone());
            }
        }
    }

    /// Select the first visible tree item.
    pub fn tree_select_first(&self) {
        if let Some(Focusable::Tree {
            items,
            on_select,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            let visible = Self::flatten_tree(items, &[]);
            if visible.is_empty() {
                return;
            }
            if let Some(callback) = on_select {
                callback(visible[0].0.clone());
            }
        }
    }

    /// Select the last visible tree item.
    pub fn tree_select_last(&self) {
        if let Some(Focusable::Tree {
            items,
            on_select,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            let visible = Self::flatten_tree(items, &[]);
            if visible.is_empty() {
                return;
            }
            if let Some(callback) = on_select {
                callback(visible[visible.len() - 1].0.clone());
            }
        }
    }

    /// Activate the currently selected tree item (trigger on_activate callback).
    pub fn tree_activate(&self) {
        if let Some(Focusable::Tree {
            selected,
            on_activate,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_activate {
                callback(selected.clone());
            }
        }
    }

    /// Get the currently selected tree path.
    #[allow(dead_code)]
    pub fn tree_selected(&self) -> Option<TreePath> {
        if let Some(Focusable::Tree { selected, .. }) = self.focusables.get(self.focus_index) {
            Some(selected.clone())
        } else {
            None
        }
    }

    // ========== Table Navigation ==========

    /// Move table selection to previous row.
    pub fn table_select_prev(&self) {
        if let Some(Focusable::Table {
            row_count,
            selected,
            on_select,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if *row_count > 0 {
                let new_selected = if *selected == 0 {
                    row_count - 1
                } else {
                    selected - 1
                };
                if let Some(callback) = on_select {
                    callback(new_selected);
                }
            }
        }
    }

    /// Move table selection to next row.
    pub fn table_select_next(&self) {
        if let Some(Focusable::Table {
            row_count,
            selected,
            on_select,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if *row_count > 0 {
                let new_selected = (selected + 1) % row_count;
                if let Some(callback) = on_select {
                    callback(new_selected);
                }
            }
        }
    }

    /// Activate the currently selected table row.
    pub fn table_activate(&self) {
        if let Some(Focusable::Table {
            selected,
            on_activate,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_activate {
                callback(*selected);
            }
        }
    }

    /// Toggle sort on a column (or cycle sort direction).
    #[allow(dead_code)]
    pub fn table_sort_column(&self, col: usize) {
        if let Some(Focusable::Table { sort, on_sort, .. }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_sort {
                // If already sorting by this column, toggle direction
                // Otherwise, sort ascending on the new column
                let new_asc = match sort {
                    Some((current_col, asc)) if *current_col == col => !asc,
                    _ => true,
                };
                callback(col, new_asc);
            }
        }
    }

    // ========== Command Palette ==========

    /// Handle text input for the command palette query.
    pub fn command_palette_key(&self, key: char) {
        if let Some(Focusable::CommandPalette {
            query,
            on_query_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_query_change {
                let new_query = format!("{}{}", query, key);
                callback(new_query);
            }
        }
    }

    /// Handle backspace for the command palette query.
    pub fn command_palette_backspace(&self) {
        if let Some(Focusable::CommandPalette {
            query,
            on_query_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_query_change {
                let mut chars: Vec<char> = query.chars().collect();
                chars.pop();
                callback(chars.into_iter().collect());
            }
        }
    }

    /// Dismiss the command palette.
    #[allow(dead_code)]
    pub fn command_palette_dismiss(&self) {
        if let Some(Focusable::CommandPalette { on_dismiss, .. }) =
            self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_dismiss {
                callback();
            }
        }
    }

    /// Execute the selected command in the palette.
    pub fn command_palette_execute(&self) {
        if let Some(Focusable::CommandPalette {
            commands,
            selected,
            query,
            on_select,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            // Filter commands by query
            let filtered = filter_commands(commands, query);
            if let Some(cmd) = filtered.get(*selected) {
                if let Some(callback) = on_select {
                    callback(cmd.id);
                }
            }
        }
    }

    /// Get filtered command count for the command palette.
    #[allow(dead_code)]
    pub fn command_palette_filtered_count(&self) -> usize {
        if let Some(Focusable::CommandPalette {
            commands, query, ..
        }) = self.focusables.get(self.focus_index)
        {
            filter_commands(commands, query).len()
        } else {
            0
        }
    }

    // ========== Menu Bar ==========

    /// Switch to the next menu (when a menu is open). Also updates highlight.
    pub fn menu_bar_next(&self) {
        if let Some(Focusable::MenuBar {
            menus,
            active_menu,
            on_menu_change,
            on_highlight_change,
            on_item_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            let current = active_menu.unwrap_or(0);
            let next = if current + 1 >= menus.len() {
                0
            } else {
                current + 1
            };

            if let Some(callback) = on_menu_change {
                callback(next);
            }
            // Keep highlight in sync
            if let Some(callback) = on_highlight_change {
                callback(next);
            }
            // Reset item selection when switching menus
            if let Some(callback) = on_item_change {
                callback(0);
            }
        }
    }

    /// Switch to the previous menu (when a menu is open). Also updates highlight.
    pub fn menu_bar_prev(&self) {
        if let Some(Focusable::MenuBar {
            menus,
            active_menu,
            on_menu_change,
            on_highlight_change,
            on_item_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            let current = active_menu.unwrap_or(0);
            let prev = if current == 0 {
                menus.len().saturating_sub(1)
            } else {
                current - 1
            };

            if let Some(callback) = on_menu_change {
                callback(prev);
            }
            // Keep highlight in sync
            if let Some(callback) = on_highlight_change {
                callback(prev);
            }
            // Reset item selection when switching menus
            if let Some(callback) = on_item_change {
                callback(0);
            }
        }
    }

    /// Execute the selected menu item.
    pub fn menu_bar_execute(&self) {
        if let Some(Focusable::MenuBar {
            menus,
            active_menu,
            selected_item,
            on_select,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(menu_idx) = active_menu {
                if let Some(menu) = menus.get(*menu_idx) {
                    // Filter out separators and get the command at selected_item
                    let commands: Vec<_> = menu
                        .items
                        .iter()
                        .filter_map(|item| {
                            if let crate::view::MenuItemNode::Command { id, .. } = item {
                                Some(*id)
                            } else {
                                None
                            }
                        })
                        .collect();
                    if let Some(cmd_id) = commands.get(*selected_item) {
                        if let Some(callback) = on_select {
                            callback(cmd_id);
                        }
                    }
                }
            }
        }
    }

    /// Get the number of command items in the active menu.
    #[allow(dead_code)]
    pub fn menu_bar_item_count(&self) -> usize {
        if let Some(Focusable::MenuBar {
            menus, active_menu, ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(menu_idx) = active_menu {
                if let Some(menu) = menus.get(*menu_idx) {
                    return menu
                        .items
                        .iter()
                        .filter(|item| matches!(item, crate::view::MenuItemNode::Command { .. }))
                        .count();
                }
            }
        }
        0
    }

    /// Check if the menu bar has an open menu.
    pub fn menu_bar_has_open_menu(&self) -> bool {
        if let Some(Focusable::MenuBar { active_menu, .. }) = self.focusables.get(self.focus_index)
        {
            active_menu.is_some()
        } else {
            false
        }
    }

    /// Open the currently highlighted menu.
    pub fn menu_bar_open(&self) {
        if let Some(Focusable::MenuBar {
            highlighted_menu,
            on_menu_change,
            on_item_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_menu_change {
                callback(*highlighted_menu);
            }
            if let Some(callback) = on_item_change {
                callback(0);
            }
        }
    }

    /// Close the active menu.
    pub fn menu_bar_close(&self) {
        if let Some(Focusable::MenuBar {
            active_menu,
            on_menu_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if active_menu.is_some() {
                // Call on_menu_change with current index to toggle it closed
                if let (Some(callback), Some(idx)) = (on_menu_change, active_menu) {
                    callback(*idx);
                }
            }
        }
    }

    /// Move highlight to next menu (without opening). Wraps around.
    pub fn menu_bar_highlight_next(&self) {
        if let Some(Focusable::MenuBar {
            menus,
            highlighted_menu,
            on_highlight_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_highlight_change {
                let next = if *highlighted_menu + 1 >= menus.len() {
                    0
                } else {
                    highlighted_menu + 1
                };
                callback(next);
            }
        }
    }

    /// Move highlight to previous menu (without opening). Wraps around.
    pub fn menu_bar_highlight_prev(&self) {
        if let Some(Focusable::MenuBar {
            menus,
            highlighted_menu,
            on_highlight_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_highlight_change {
                let prev = if *highlighted_menu == 0 {
                    menus.len().saturating_sub(1)
                } else {
                    highlighted_menu - 1
                };
                callback(prev);
            }
        }
    }

    /// Select the next item in the active menu.
    pub fn menu_bar_select_next(&self) {
        if let Some(Focusable::MenuBar {
            menus,
            active_menu,
            selected_item,
            on_item_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(menu_idx) = active_menu {
                if let Some(menu) = menus.get(*menu_idx) {
                    let item_count = menu
                        .items
                        .iter()
                        .filter(|item| matches!(item, crate::view::MenuItemNode::Command { .. }))
                        .count();
                    if item_count > 0 {
                        let new_selected = (selected_item + 1) % item_count;
                        if let Some(callback) = on_item_change {
                            callback(new_selected);
                        }
                    }
                }
            }
        }
    }

    /// Select the previous item in the active menu.
    pub fn menu_bar_select_prev(&self) {
        if let Some(Focusable::MenuBar {
            menus,
            active_menu,
            selected_item,
            on_item_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(menu_idx) = active_menu {
                if let Some(menu) = menus.get(*menu_idx) {
                    let item_count = menu
                        .items
                        .iter()
                        .filter(|item| matches!(item, crate::view::MenuItemNode::Command { .. }))
                        .count();
                    if item_count > 0 {
                        let new_selected = if *selected_item == 0 {
                            item_count - 1
                        } else {
                            selected_item - 1
                        };
                        if let Some(callback) = on_item_change {
                            callback(new_selected);
                        }
                    }
                }
            }
        }
    }

    // ========== Form Field ==========

    /// Handle text input for the focused form field.
    pub fn form_field_key(&self, key: char) {
        if let Some(Focusable::FormField {
            value,
            cursor_pos,
            on_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_change {
                // Insert at grapheme position
                let grapheme_count = text::grapheme_count(value);
                let pos = (*cursor_pos).min(grapheme_count);
                let new_value = text::insert_at_grapheme(value, pos, &key.to_string());
                callback(new_value);
            }
        }
    }

    /// Handle backspace for the focused form field.
    pub fn form_field_backspace(&self) {
        if let Some(Focusable::FormField {
            value,
            cursor_pos,
            on_change,
            ..
        }) = self.focusables.get(self.focus_index)
        {
            if let Some(callback) = on_change {
                let grapheme_count = text::grapheme_count(value);
                let pos = (*cursor_pos).min(grapheme_count);
                if pos > 0 {
                    if let Some(new_value) = text::remove_at_grapheme(value, pos - 1) {
                        callback(new_value);
                    }
                }
            }
        }
    }

    /// Trigger blur callback for the focused form field.
    #[allow(dead_code)]
    pub fn form_field_blur(&self) {
        if let Some(Focusable::FormField { on_blur, .. }) = self.focusables.get(self.focus_index) {
            if let Some(callback) = on_blur {
                callback();
            }
        }
    }

    /// Get the current cursor position of the focused form field.
    #[allow(dead_code)]
    pub fn form_field_cursor(&self) -> usize {
        if let Some(Focusable::FormField { cursor_pos, .. }) = self.focusables.get(self.focus_index)
        {
            *cursor_pos
        } else {
            0
        }
    }

    // ========== Terminal ==========

    /// Send key event to the focused terminal as PTY input.
    pub fn terminal_key(&self, key: crossterm::event::KeyEvent) -> Result<(), String> {
        if let Some(Focusable::Terminal { handle, .. }) = self.focusables.get(self.focus_index) {
            let bytes = key_event_to_bytes(key);
            handle.send_input(&bytes)?;
        }
        Ok(())
    }

    /// Check if a terminal has exited and invoke callback if needed.
    #[allow(dead_code)]
    pub fn terminal_check_exit(&self) {
        if let Some(Focusable::Terminal { handle, on_exit }) = self.focusables.get(self.focus_index) {
            if handle.is_exited() {
                if let Some(callback) = on_exit {
                    callback();
                }
            }
        }
    }

    /// Check if the currently focused element is a terminal.
    pub fn is_focused_terminal(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::Terminal { .. })
        )
    }

    /// Poll all terminal handles for new output.
    pub fn poll_terminals(&self) {
        for focusable in &self.focusables {
            if let Focusable::Terminal { handle, .. } = focusable {
                handle.poll();
            }
        }
    }

    // ========== Slider ==========

    pub fn is_focused_slider(&self) -> bool {
        matches!(
            self.focusables.get(self.focus_index),
            Some(Focusable::Slider { .. })
        )
    }

    /// Increment the slider value by one step.
    pub fn slider_increment(&self) {
        if let Some(Focusable::Slider {
            min,
            max,
            value,
            step,
            on_change,
        }) = self.focusables.get(self.focus_index)
        {
            let new_value = (value + step).min(*max).max(*min);
            if let Some(cb) = on_change {
                cb(new_value);
            }
        }
    }

    /// Decrement the slider value by one step.
    pub fn slider_decrement(&self) {
        if let Some(Focusable::Slider {
            min,
            max,
            value,
            step,
            on_change,
        }) = self.focusables.get(self.focus_index)
        {
            let new_value = (value - step).min(*max).max(*min);
            if let Some(cb) = on_change {
                cb(new_value);
            }
        }
    }
}

/// Convert a KeyEvent to bytes for PTY input.
fn key_event_to_bytes(key: crossterm::event::KeyEvent) -> Vec<u8> {
    use crossterm::event::{KeyCode, KeyModifiers};

    let mut bytes = Vec::new();

    match key.code {
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+letter sends control codes (Ctrl+A = 0x01, etc.)
                if c.is_ascii_alphabetic() {
                    let ctrl_code = (c.to_ascii_lowercase() as u8) - b'a' + 1;
                    bytes.push(ctrl_code);
                } else {
                    // Other Ctrl+ combinations
                    match c {
                        '@' => bytes.push(0x00), // Ctrl+@
                        '[' => bytes.push(0x1b), // Ctrl+[ (ESC)
                        '\\' => bytes.push(0x1c), // Ctrl+\
                        ']' => bytes.push(0x1d), // Ctrl+]
                        '^' => bytes.push(0x1e), // Ctrl+^
                        '_' => bytes.push(0x1f), // Ctrl+_
                        '?' => bytes.push(0x7f), // Ctrl+? (DEL)
                        _ => bytes.extend_from_slice(c.to_string().as_bytes()),
                    }
                }
            } else if key.modifiers.contains(KeyModifiers::ALT) {
                // Alt+key sends ESC followed by the key
                bytes.push(0x1b);
                bytes.extend_from_slice(c.to_string().as_bytes());
            } else {
                bytes.extend_from_slice(c.to_string().as_bytes());
            }
        }
        KeyCode::Enter => bytes.push(b'\r'),
        KeyCode::Backspace => bytes.push(0x08),
        KeyCode::Tab => bytes.push(b'\t'),
        KeyCode::Esc => bytes.push(0x1b),
        KeyCode::Up => bytes.extend_from_slice(b"\x1b[A"),
        KeyCode::Down => bytes.extend_from_slice(b"\x1b[B"),
        KeyCode::Right => bytes.extend_from_slice(b"\x1b[C"),
        KeyCode::Left => bytes.extend_from_slice(b"\x1b[D"),
        KeyCode::Home => bytes.extend_from_slice(b"\x1b[H"),
        KeyCode::End => bytes.extend_from_slice(b"\x1b[F"),
        KeyCode::PageUp => bytes.extend_from_slice(b"\x1b[5~"),
        KeyCode::PageDown => bytes.extend_from_slice(b"\x1b[6~"),
        KeyCode::Delete => bytes.extend_from_slice(b"\x1b[3~"),
        KeyCode::Insert => bytes.extend_from_slice(b"\x1b[2~"),
        KeyCode::F(n) => {
            // F1-F12 key sequences
            match n {
                1 => bytes.extend_from_slice(b"\x1bOP"),
                2 => bytes.extend_from_slice(b"\x1bOQ"),
                3 => bytes.extend_from_slice(b"\x1bOR"),
                4 => bytes.extend_from_slice(b"\x1bOS"),
                5 => bytes.extend_from_slice(b"\x1b[15~"),
                6 => bytes.extend_from_slice(b"\x1b[17~"),
                7 => bytes.extend_from_slice(b"\x1b[18~"),
                8 => bytes.extend_from_slice(b"\x1b[19~"),
                9 => bytes.extend_from_slice(b"\x1b[20~"),
                10 => bytes.extend_from_slice(b"\x1b[21~"),
                11 => bytes.extend_from_slice(b"\x1b[23~"),
                12 => bytes.extend_from_slice(b"\x1b[24~"),
                _ => {}
            }
        }
        _ => {}
    }

    bytes
}

/// Filter palette commands by query using fuzzy matching.
fn filter_commands<'a>(commands: &'a [PaletteCommand], query: &str) -> Vec<&'a PaletteCommand> {
    if query.is_empty() {
        return commands.iter().collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches: Vec<(&PaletteCommand, i32)> = commands
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

    matches.sort_by(|a, b| b.1.cmp(&a.1));
    matches.into_iter().map(|(cmd, _)| cmd).collect()
}

/// Simple fuzzy matching score.
fn fuzzy_score(text: &str, query: &str) -> i32 {
    if query.is_empty() {
        return 1;
    }

    let text_chars: Vec<char> = text.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();

    let mut text_idx = 0;
    let mut query_idx = 0;
    let mut score = 0;
    let mut consecutive = 0;

    while text_idx < text_chars.len() && query_idx < query_chars.len() {
        if text_chars[text_idx] == query_chars[query_idx] {
            consecutive += 1;
            score += consecutive * 2;
            if text_idx == 0 || !text_chars[text_idx - 1].is_alphanumeric() {
                score += 5;
            }
            query_idx += 1;
        } else {
            consecutive = 0;
        }
        text_idx += 1;
    }

    if query_idx == query_chars.len() {
        score
    } else {
        0
    }
}
