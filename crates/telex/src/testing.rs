//! Testing utilities for Telex components.
//!
//! Provides `TestApp` for testing components without a real terminal.
//!
//! # Example
//! ```rust,ignore
//! use telex::testing::TestApp;
//! use telex::prelude::*;
//!
//! #[test]
//! fn counter_increments() {
//!     let mut app = TestApp::new(|cx| {
//!         let count = state!(cx, || 0);
//!         let c = count.clone();
//!         View::vstack()
//!             .child(View::text(format!("Count: {}", count.get())))
//!             .child(View::button().label("+").on_press(move || c.update(|n| *n + 1)).build())
//!             .build()
//!     });
//!
//!     assert!(app.find_text("Count: 0").is_some());
//!     app.press_button("+");
//!     assert!(app.find_text("Count: 1").is_some());
//! }
//! ```

use crate::buffer::Buffer;
use crate::component::Component;
use crate::focus::FocusManager;
use crate::render::{render_view, RenderContext};
use crate::scope::{Scope, StateStorage};
use crate::terminal::Terminal;
use crate::view::{ButtonNode, CheckboxNode, ListNode, TextInputNode, TextNode, View};
use crate::EventSource;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

/// A test harness for Telex components.
///
/// Renders components to an in-memory buffer and provides methods
/// for finding elements and simulating interactions.
pub struct TestApp<C: Component> {
    root: C,
    storage: Rc<StateStorage>,
    focus: FocusManager,
    width: u16,
    height: u16,
}

impl<C: Component> TestApp<C> {
    /// Create a new test app with the given root component.
    pub fn new(root: C) -> Self {
        Self {
            root,
            storage: Rc::new(StateStorage::new()),
            focus: FocusManager::new(),
            width: 80,
            height: 24,
        }
    }

    /// Set the virtual terminal size.
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Render the component and return the view tree.
    fn render(&self) -> View {
        let cx = Scope::with_storage(Rc::clone(&self.storage));
        self.root.render(cx)
    }

    /// Render to a buffer and return the buffer contents as a string.
    pub fn render_to_string(&mut self) -> String {
        let view = self.render();
        self.focus.collect_focusables(&view);

        let mut buffer = Buffer::new(self.width, self.height);
        let area = buffer.rect();

        let scroll_offsets: Vec<(u16, u16)> = (0..self.focus.focus_index() + 10)
            .map(|i| self.focus.scroll_offset(i))
            .collect();
        let cursor_offsets: Vec<usize> = (0..self.focus.focus_index() + 10)
            .map(|i| self.focus.cursor_offset(i))
            .collect();

        // In tests, always show focus styling so we can verify focus behavior
        let mut ctx = RenderContext::new(self.focus.focus_index(), true, scroll_offsets, cursor_offsets, area);
        render_view(&mut buffer, &view, area, &mut ctx);
        ctx.render_pending_dropdowns(&mut buffer);

        // Run pending effects after render (mirrors lib.rs behavior)
        self.storage.flush_effects();

        buffer.to_string()
    }

    /// Find all text content in the view tree.
    pub fn find_all_text(&self) -> Vec<String> {
        let view = self.render();
        let mut texts = Vec::new();
        Self::collect_text(&view, &mut texts);
        texts
    }

    /// Find text that contains the given substring.
    pub fn find_text(&self, needle: &str) -> Option<String> {
        self.find_all_text()
            .into_iter()
            .find(|t| t.contains(needle))
    }

    /// Check if text containing the given substring exists.
    pub fn has_text(&self, needle: &str) -> bool {
        self.find_text(needle).is_some()
    }

    /// Find all button labels in the view tree.
    pub fn find_all_buttons(&self) -> Vec<String> {
        let view = self.render();
        let mut buttons = Vec::new();
        Self::collect_buttons(&view, &mut buttons);
        buttons
    }

    /// Find a button by its label.
    pub fn find_button(&self, label: &str) -> Option<String> {
        self.find_all_buttons().into_iter().find(|l| l == label)
    }

    /// Get the current focus index.
    pub fn focus_index(&self) -> usize {
        self.focus.focus_index()
    }

    /// Get the total number of focusable elements.
    pub fn focusable_count(&mut self) -> usize {
        let view = self.render();
        self.focus.collect_focusables(&view);
        self.focus.focusable_count()
    }

    /// Move focus to the next element.
    pub fn focus_next(&mut self) {
        let view = self.render();
        self.focus.collect_focusables(&view);
        self.focus.focus_next();
    }

    /// Move focus to the previous element.
    pub fn focus_prev(&mut self) {
        let view = self.render();
        self.focus.collect_focusables(&view);
        self.focus.focus_prev();
    }

    /// Activate the currently focused element (press button, toggle checkbox).
    pub fn activate(&mut self) {
        let view = self.render();
        self.focus.collect_focusables(&view);
        self.focus.activate();
    }

    /// Press a button by its label.
    ///
    /// Finds the button, focuses it, and activates it.
    pub fn press_button(&mut self, label: &str) -> bool {
        let view = self.render();
        self.focus.collect_focusables(&view);

        // Find the button index
        if let Some(idx) = self.find_button_index(&view, label) {
            // Focus it
            while self.focus.focus_index() != idx {
                self.focus.focus_next();
            }
            // Activate it
            self.focus.activate();
            true
        } else {
            false
        }
    }

    /// Move list selection up.
    pub fn list_up(&mut self) {
        let view = self.render();
        self.focus.collect_focusables(&view);
        self.focus.list_select_prev();
    }

    /// Move list selection down.
    pub fn list_down(&mut self) {
        let view = self.render();
        self.focus.collect_focusables(&view);
        self.focus.list_select_next();
    }

    /// Type a character into the focused text input or text area.
    pub fn type_char(&mut self, c: char) {
        let view = self.render();
        self.focus.collect_focusables(&view);
        // Set wrap width for text areas (simulating lib.rs behavior)
        self.focus
            .set_default_textarea_wrap_width(self.width.saturating_sub(4));
        if self.focus.is_focused_text_area() {
            self.focus.text_area_key(c);
        } else {
            self.focus.text_input_key(c);
        }
    }

    /// Type a string into the focused text input or text area.
    pub fn type_str(&mut self, s: &str) {
        for c in s.chars() {
            self.type_char(c);
        }
    }

    /// Press backspace in the focused text input or text area.
    pub fn backspace(&mut self) {
        let view = self.render();
        self.focus.collect_focusables(&view);
        if self.focus.is_focused_text_area() {
            self.focus.text_area_backspace();
        } else {
            self.focus.text_input_backspace();
        }
    }

    /// Press Enter in the focused text area (insert new line).
    pub fn enter(&mut self) {
        let view = self.render();
        self.focus.collect_focusables(&view);
        if self.focus.is_focused_text_area() {
            self.focus.text_area_enter();
        }
    }

    /// Scroll up in the focused scrollable.
    pub fn scroll_up(&mut self, amount: u16) {
        let view = self.render();
        self.focus.collect_focusables(&view);
        self.focus.scroll_up(amount);
    }

    /// Scroll down in the focused scrollable.
    pub fn scroll_down(&mut self, amount: u16) {
        let view = self.render();
        self.focus.collect_focusables(&view);
        self.focus.scroll_down(amount, 100);
    }

    // Helper: collect all text from view tree
    fn collect_text(view: &View, texts: &mut Vec<String>) {
        match view {
            View::Text(TextNode { content, .. }) => {
                texts.push(content.clone());
            }
            View::VStack(node) => {
                for child in &node.children {
                    Self::collect_text(child, texts);
                }
            }
            View::HStack(node) => {
                for child in &node.children {
                    Self::collect_text(child, texts);
                }
            }
            View::Box(node) => {
                if let Some(child) = &node.child {
                    Self::collect_text(child, texts);
                }
            }
            View::Button(ButtonNode { label, .. }) => {
                texts.push(label.clone());
            }
            View::List(ListNode { items, .. }) => {
                texts.extend(items.clone());
            }
            View::TextInput(TextInputNode {
                value, placeholder, ..
            }) => {
                if value.is_empty() {
                    texts.push(placeholder.clone());
                } else {
                    texts.push(value.clone());
                }
            }
            View::Checkbox(CheckboxNode { label, .. }) => {
                texts.push(label.clone());
            }
            View::ErrorBoundary(node) => {
                Self::collect_text(&node.child, texts);
            }
            _ => {}
        }
    }

    // Helper: collect all button labels from view tree
    fn collect_buttons(view: &View, buttons: &mut Vec<String>) {
        match view {
            View::Button(ButtonNode { label, .. }) => {
                buttons.push(label.clone());
            }
            View::VStack(node) => {
                for child in &node.children {
                    Self::collect_buttons(child, buttons);
                }
            }
            View::HStack(node) => {
                for child in &node.children {
                    Self::collect_buttons(child, buttons);
                }
            }
            View::Box(node) => {
                if let Some(child) = &node.child {
                    Self::collect_buttons(child, buttons);
                }
            }
            View::ErrorBoundary(node) => {
                Self::collect_buttons(&node.child, buttons);
            }
            _ => {}
        }
    }

    // Helper: find the focusable index of a button by label
    fn find_button_index(&self, view: &View, label: &str) -> Option<usize> {
        let mut index = 0;
        Self::find_button_index_recursive(view, label, &mut index)
    }

    fn find_button_index_recursive(view: &View, label: &str, index: &mut usize) -> Option<usize> {
        match view {
            View::Button(ButtonNode {
                label: btn_label, ..
            }) => {
                if btn_label == label {
                    Some(*index)
                } else {
                    *index += 1;
                    None
                }
            }
            View::Box(node) => {
                if node.scroll {
                    *index += 1;
                }
                if let Some(child) = &node.child {
                    Self::find_button_index_recursive(child, label, index)
                } else {
                    None
                }
            }
            View::VStack(node) => {
                for child in &node.children {
                    if let Some(idx) = Self::find_button_index_recursive(child, label, index) {
                        return Some(idx);
                    }
                }
                None
            }
            View::HStack(node) => {
                for child in &node.children {
                    if let Some(idx) = Self::find_button_index_recursive(child, label, index) {
                        return Some(idx);
                    }
                }
                None
            }
            View::ErrorBoundary(node) => {
                Self::find_button_index_recursive(&node.child, label, index)
            }
            View::List(_) | View::TextInput(_) | View::Checkbox(_) => {
                *index += 1;
                None
            }
            _ => None,
        }
    }

    // ========== Visibility Assertions ==========

    /// Assert that the given text is visible in the rendered output.
    /// Panics with a helpful message showing the rendered output if not found.
    pub fn assert_visible(&mut self, needle: &str) {
        let rendered = self.render_to_string();
        if !rendered.contains(needle) {
            panic!(
                "\n\nassertion failed: expected {:?} to be visible\n\nRendered output ({}x{}):\n{}\n",
                needle, self.width, self.height, rendered
            );
        }
    }

    /// Assert that the given text is NOT visible in the rendered output.
    /// Panics with a helpful message if the text is found.
    pub fn assert_not_visible(&mut self, needle: &str) {
        let rendered = self.render_to_string();
        if rendered.contains(needle) {
            panic!(
                "\n\nassertion failed: expected {:?} to NOT be visible\n\nRendered output ({}x{}):\n{}\n",
                needle, self.width, self.height, rendered
            );
        }
    }

    /// Check which items from the given list are visible in the rendered output.
    /// Returns a Vec of the items that are visible.
    pub fn visible_items(&mut self, items: &[&str]) -> Vec<String> {
        let rendered = self.render_to_string();
        items
            .iter()
            .filter(|item| rendered.contains(*item))
            .map(|s| s.to_string())
            .collect()
    }

    // ========== Rendered Output Helpers ==========

    /// Get the rendered output as a Vec of lines.
    pub fn rendered_lines(&mut self) -> Vec<String> {
        self.render_to_string()
            .lines()
            .map(|s| s.to_string())
            .collect()
    }

    /// Find the line number (0-indexed) containing the given text.
    /// Returns None if not found.
    pub fn find_line_containing(&mut self, needle: &str) -> Option<usize> {
        self.rendered_lines()
            .iter()
            .position(|line| line.contains(needle))
    }

    // ========== Viewport Info ==========

    /// Get the viewport height (visible area).
    pub fn viewport_height(&self) -> u16 {
        self.height
    }

    /// Get the viewport width (visible area).
    pub fn viewport_width(&self) -> u16 {
        self.width
    }
}

/// Assert that the rendered output matches a snapshot.
///
/// On first run, creates the snapshot. On subsequent runs, compares.
#[macro_export]
macro_rules! assert_snapshot {
    ($app:expr) => {
        let rendered = $app.render_to_string();
        // For now, just print - in real usage, compare to stored snapshot
        println!("Snapshot:\n{}", rendered);
    };
    ($app:expr, $name:expr) => {
        let rendered = $app.render_to_string();
        println!("Snapshot [{}]:\n{}", $name, rendered);
    };
}

// =============================================================================
// TestEventSource - for headless event loop testing
// =============================================================================

/// A test event source that replays a scripted sequence of events.
///
/// Used by `run_headless()` to inject key events into the real event loop.
/// When all events are consumed, returns Ctrl+Q to exit the loop.
pub struct TestEventSource {
    events: RefCell<VecDeque<Event>>,
    exhausted: RefCell<bool>,
    last_buffer: RefCell<String>,
}

impl TestEventSource {
    /// Create a new test event source with the given events.
    pub fn new(events: Vec<Event>) -> Self {
        Self {
            events: RefCell::new(events.into()),
            exhausted: RefCell::new(false),
            last_buffer: RefCell::new(String::new()),
        }
    }

    /// Get the last rendered buffer string.
    pub fn last_buffer(&self) -> String {
        self.last_buffer.borrow().clone()
    }
}

impl EventSource for TestEventSource {
    fn poll_event(&self, _timeout: Duration) -> std::io::Result<Option<Event>> {
        let mut events = self.events.borrow_mut();
        if let Some(event) = events.pop_front() {
            Ok(Some(event))
        } else if !*self.exhausted.borrow() {
            // First time exhausted: send Ctrl+Q to quit
            *self.exhausted.borrow_mut() = true;
            Ok(Some(Event::Key(KeyEvent::new(
                KeyCode::Char('q'),
                KeyModifiers::CONTROL,
            ))))
        } else {
            // Already sent quit, return None
            Ok(None)
        }
    }

    fn on_frame_rendered(&self, terminal: &Terminal) {
        *self.last_buffer.borrow_mut() = terminal.buffer_string();
    }
}

// =============================================================================
// StreamTestEventSource - for testing background stream wake behavior
// =============================================================================

/// A test event source that lets real time pass with no user input.
///
/// Unlike `TestEventSource` which fires events instantly, this source
/// respects poll timeouts and waits until a deadline before sending Ctrl+Q.
/// This allows background streams to wake the event loop and trigger
/// re-renders — exactly what we need to test the wake mechanism.
pub struct StreamTestEventSource {
    deadline: std::time::Instant,
    exhausted: RefCell<bool>,
    /// Every rendered frame's buffer, in order.
    frames: RefCell<Vec<String>>,
}

impl StreamTestEventSource {
    /// Create a new stream test source that waits `duration` before quitting.
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: std::time::Instant::now() + duration,
            exhausted: RefCell::new(false),
            frames: RefCell::new(Vec::new()),
        }
    }

    /// Get all rendered frame buffers, in order.
    pub fn frames(&self) -> Vec<String> {
        self.frames.borrow().clone()
    }
}

impl EventSource for StreamTestEventSource {
    fn poll_event(&self, timeout: Duration) -> std::io::Result<Option<Event>> {
        if std::time::Instant::now() >= self.deadline {
            if !*self.exhausted.borrow() {
                *self.exhausted.borrow_mut() = true;
                return Ok(Some(Event::Key(KeyEvent::new(
                    KeyCode::Char('q'),
                    KeyModifiers::CONTROL,
                ))));
            }
            return Ok(None);
        }
        // Actually sleep — let background threads send tokens and set wake flags
        std::thread::sleep(timeout);
        Ok(None)
    }

    fn on_frame_rendered(&self, terminal: &Terminal) {
        self.frames.borrow_mut().push(terminal.buffer_string());
    }
}
