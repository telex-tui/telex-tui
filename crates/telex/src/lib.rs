//! Telex - A DX-first TUI framework for Rust.
//!
//! Build terminal apps that feel good to write.

// =============================================================================
// API Versioning
// =============================================================================

/// Current API major version.
/// For 0.x releases, minor version bumps may contain breaking changes.
pub const API_VERSION_MAJOR: u32 = 0;

/// Current API minor version.
pub const API_VERSION_MINOR: u32 = 2;

/// Current API patch version.
pub const API_VERSION_PATCH: u32 = 0;

/// Check that your code is compatible with the current Telex API version.
///
/// For pre-1.0 versions (0.x.y), this requires an exact major.minor match,
/// since breaking changes can occur on minor version bumps.
///
/// For 1.0+ versions, this requires the same major version and that your
/// required minor version is not newer than the library's minor version.
///
/// # Example
/// ```rust,ignore
/// use telex::prelude::*;
///
/// telex::require_api!(0, 2);  // Requires API version 0.2.x
///
/// fn main() {
///     telex::run(App).unwrap();
/// }
/// ```
///
/// If the version doesn't match, you'll get a compile-time error with
/// guidance on how to migrate.
#[macro_export]
macro_rules! require_api {
    ($major:literal, $minor:literal) => {
        const _: () = {
            // For 0.x versions, require exact major.minor match (breaking changes on minor bumps)
            // For 1.x+, require same major and compatible minor (required <= current)
            if $crate::API_VERSION_MAJOR == 0 {
                // Pre-1.0: exact match required
                assert!(
                    $major == $crate::API_VERSION_MAJOR && $minor == $crate::API_VERSION_MINOR,
                    concat!(
                        "Telex API version mismatch: this code requires ", $major, ".", $minor,
                        " but the library is version ",
                        env!("CARGO_PKG_VERSION"),
                        ". See https://docs.rs/telex for migration guides."
                    )
                );
            } else {
                // Post-1.0: same major, compatible minor
                assert!(
                    $major == $crate::API_VERSION_MAJOR,
                    concat!(
                        "Telex API major version mismatch: this code requires major version ", $major,
                        " but the library is version ",
                        env!("CARGO_PKG_VERSION"),
                        ". This is a breaking change - see https://docs.rs/telex for migration guides."
                    )
                );
                assert!(
                    $minor <= $crate::API_VERSION_MINOR,
                    concat!(
                        "Telex API minor version too new: this code requires ", $major, ".", $minor,
                        " but the library is version ",
                        env!("CARGO_PKG_VERSION"),
                        ". Please upgrade the telex dependency in your Cargo.toml."
                    )
                );
            }
        };
    };
}

// =============================================================================
// Modules
// =============================================================================

mod async_state;
pub mod buffer;
pub mod canvas;
pub mod channel;
mod command;
pub mod command_system;
mod component;
mod context;
mod focus;
pub mod form;
pub mod image;
pub mod markdown;
mod render;
mod scope;
mod state;
mod stream_state;
mod terminal;
mod terminal_state;
pub mod testing;
pub mod text;
pub mod theme;
pub mod toast;
mod view;
pub mod widget;

pub mod prelude;

pub use async_state::Async;
pub use channel::{ChannelDrain, ChannelHandle, PortHandle, WakingSender};
pub use command::KeyBinding;
pub use component::Component;
pub use scope::Scope;
pub use state::State;
pub use stream_state::{StreamHandle, StreamState, TextStreamHandle};
pub use telex_macro::{async_data, channel as channel_macro, effect, effect_once, interval, port, reducer, state, stream, terminal, text_stream, text_stream_with_restart, view, with};
pub use terminal::Terminal;
pub use terminal_state::{TerminalBuffer, TerminalHandle};
pub use view::{
    Align, BoxBuilder, BoxNode, ButtonBuilder, ButtonNode, Callback, CanvasBuilder, CanvasNode,
    ChangeCallback, CheckboxBuilder, CheckboxNode, ColumnWidth, CommandCallback,
    CommandPaletteBuilder, CommandPaletteNode, CustomNode, ErrorBoundaryBuilder,
    ErrorBoundaryNode, FormBuilder, FormFieldBuilder, FormFieldNode, FormNode,
    FormSubmitCallback, HStackBuilder, HStackNode, ImageBuilder, ImageNode, Justify, LayoutMode,
    SliderBuilder, SliderCallback, SliderNode,
    ListBuilder, ListNode, Menu, MenuBarBuilder, MenuBarNode, MenuItemNode, ModalBuilder,
    ModalNode, Orientation, PaletteCommand, RadioGroupBuilder, RadioGroupNode, SelectCallback,
    SpacerNode, SplitBuilder, SplitNode, TabPosition, TableBuilder, TableColumn, TableNode,
    TabsBuilder, TabsNode, TextAlign, TextAreaBuilder, TextAreaNode, TextBuilder,
    TextInputBuilder, TextInputNode, TextNode, TerminalBuilder, TerminalNode,
    ToastContainerBuilder, ToastContainerNode, ToastItem, ToastLevelView, ToastPosition,
    ToggleCallback, TreeActivateCallback, TreeBuilder, TreeItem, TreeNode, TreePath,
    TreeSelectCallback, VStackBuilder, VStackNode, View,
};

// Re-export canvas types for pixel-level drawing
pub use canvas::{animated_canvas, AnimatedCanvasBuilder, DrawContext, PixelBuffer};

// Re-export image types
pub use image::ImageSource;

// Re-export crossterm types needed for event handling and styling
pub use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
pub use crossterm::style::Color;

use command::CommandRegistry;
use context::ContextStorage;
use focus::FocusManager;
use scope::StateStorage;
use std::cell::Cell;
use std::io::{self, Result};
use std::panic;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::time::Duration;

thread_local! {
    /// When true, the panic hook skips terminal cleanup because the panic will
    /// be caught by an error boundary's `catch_unwind`.
    pub(crate) static IN_ERROR_BOUNDARY: Cell<bool> = const { Cell::new(false) };
}
use theme::Theme;

/// Trait for providing input events to the run loop.
///
/// The default implementation (`CrosstermEventSource`) uses crossterm's real
/// terminal. Tests can provide a mock implementation via `run_headless()`.
pub trait EventSource {
    /// Poll for an event with the given timeout.
    /// Returns `Ok(Some(event))` if an event is available, `Ok(None)` on timeout.
    fn poll_event(&self, timeout: Duration) -> io::Result<Option<Event>>;

    /// Called after each frame is rendered. Default is a no-op.
    /// The test event source uses this to capture the rendered buffer.
    fn on_frame_rendered(&self, _terminal: &Terminal) {}
}

/// Event source that reads from the real terminal via crossterm.
struct CrosstermEventSource;

impl EventSource for CrosstermEventSource {
    fn poll_event(&self, timeout: Duration) -> io::Result<Option<Event>> {
        if crossterm::event::poll(timeout)? {
            Ok(Some(crossterm::event::read()?))
        } else {
            Ok(None)
        }
    }
}

/// Check if any modal is visible in the view tree.
fn has_visible_modal(view: &View) -> bool {
    match view {
        View::Modal(node) => node.visible,
        View::VStack(node) => node.children.iter().any(has_visible_modal),
        View::HStack(node) => node.children.iter().any(has_visible_modal),
        View::Box(node) => node
            .child
            .as_ref()
            .map(|c| has_visible_modal(c))
            .unwrap_or(false),
        View::Split(node) => has_visible_modal(&node.first) || has_visible_modal(&node.second),
        View::Tabs(node) => node.children.iter().any(has_visible_modal),
        View::ErrorBoundary(node) => has_visible_modal(&node.child),
        _ => false,
    }
}

/// Check if any command palette is visible in the view tree.
fn has_visible_command_palette(view: &View) -> bool {
    match view {
        View::CommandPalette(node) => node.visible,
        View::VStack(node) => node.children.iter().any(has_visible_command_palette),
        View::HStack(node) => node.children.iter().any(has_visible_command_palette),
        View::Box(node) => node
            .child
            .as_ref()
            .map(|c| has_visible_command_palette(c))
            .unwrap_or(false),
        View::Split(node) => {
            has_visible_command_palette(&node.first) || has_visible_command_palette(&node.second)
        }
        View::Tabs(node) => node.children.iter().any(has_visible_command_palette),
        View::ErrorBoundary(node) => has_visible_command_palette(&node.child),
        _ => false,
    }
}

/// Call the dismiss callback on visible command palettes.
fn call_command_palette_dismiss(view: &View) {
    match view {
        View::CommandPalette(node) => {
            if node.visible {
                if let Some(callback) = &node.on_dismiss {
                    callback();
                }
            }
        }
        View::VStack(node) => {
            for child in &node.children {
                call_command_palette_dismiss(child);
            }
        }
        View::HStack(node) => {
            for child in &node.children {
                call_command_palette_dismiss(child);
            }
        }
        View::Box(node) => {
            if let Some(child) = &node.child {
                call_command_palette_dismiss(child);
            }
        }
        View::Split(node) => {
            call_command_palette_dismiss(&node.first);
            call_command_palette_dismiss(&node.second);
        }
        View::Tabs(node) => {
            for child in &node.children {
                call_command_palette_dismiss(child);
            }
        }
        View::ErrorBoundary(node) => {
            call_command_palette_dismiss(&node.child);
        }
        _ => {}
    }
}

/// Find visible modals in the view tree and call their on_dismiss callbacks.
fn call_modal_dismiss(view: &View) {
    match view {
        View::Modal(node) => {
            if node.visible {
                if let Some(callback) = &node.on_dismiss {
                    callback();
                }
            }
        }
        View::VStack(node) => {
            for child in &node.children {
                call_modal_dismiss(child);
            }
        }
        View::HStack(node) => {
            for child in &node.children {
                call_modal_dismiss(child);
            }
        }
        View::Box(node) => {
            if let Some(child) = &node.child {
                call_modal_dismiss(child);
            }
        }
        View::Split(node) => {
            call_modal_dismiss(&node.first);
            call_modal_dismiss(&node.second);
        }
        View::Tabs(node) => {
            for child in &node.children {
                call_modal_dismiss(child);
            }
        }
        View::ErrorBoundary(node) => {
            call_modal_dismiss(&node.child);
        }
        _ => {}
    }
}

/// Check if debug mode is enabled via TELEX_DEBUG environment variable.
pub fn is_debug_mode() -> bool {
    std::env::var("TELEX_DEBUG")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

/// Run the application with the given root component and theme.
///
/// # Example
/// ```rust,no_run
/// use telex::prelude::*;
/// use telex::theme::Theme;
///
/// telex::run_with_theme(
///     |cx| view! { <Text>"Hello, Telex!"</Text> },
///     Theme::nord(),
/// ).unwrap();
/// ```
pub fn run_with_theme<C: Component>(root: C, theme: Theme) -> Result<()> {
    theme::set_theme(theme);
    run(root)
}

/// Run the application with the given root component.
///
/// This is the main entry point for Telex applications.
///
/// # Example
/// ```rust,no_run
/// use telex::prelude::*;
///
/// telex::run(|cx| view! { <Text>"Hello, Telex!"</Text> }).unwrap();
/// ```
///
/// # Debug Mode
/// Set `TELEX_DEBUG=1` to enable debug mode, which shows render timing
/// and focus information.
pub fn run<C: Component>(root: C) -> Result<()> {
    // Set up custom panic handler to restore terminal on panic
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // If we're inside an error boundary's catch_unwind, skip cleanup —
        // the boundary will handle rendering the fallback.
        if IN_ERROR_BOUNDARY.with(|f| f.get()) {
            return;
        }

        // Try to restore terminal state
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );

        // Print a helpful error message
        eprintln!("\n┌─ Telex Panic ─────────────────────────────────────────────────┐");
        eprintln!("│                                                              │");

        // Extract panic message
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        // Word wrap the message
        for line in message.lines() {
            let chunks: Vec<&str> = line
                .as_bytes()
                .chunks(58)
                .map(|c| std::str::from_utf8(c).unwrap_or(""))
                .collect();
            for chunk in chunks {
                eprintln!("│  {:<58}│", chunk);
            }
        }

        eprintln!("│                                                              │");

        // Print location if available
        if let Some(location) = panic_info.location() {
            eprintln!(
                "│  Location: {}:{}:{:<25}│",
                location.file().split('/').next_back().unwrap_or(location.file()),
                location.line(),
                location.column()
            );
        }

        eprintln!("│                                                              │");
        eprintln!("│  Tip: Check your hook order - hooks must be called          │");
        eprintln!("│  unconditionally in the same order every render.            │");
        eprintln!("│                                                              │");
        eprintln!("└──────────────────────────────────────────────────────────────┘\n");

        // Call default hook for stack trace
        default_hook(panic_info);
    }));

    let terminal = Terminal::new()?;
    let event_source = CrosstermEventSource;
    run_inner(root, terminal, &event_source)
}

/// Run a component headlessly with scripted events. For testing only.
///
/// Runs the real event loop with a headless terminal and injected events.
/// When all events are consumed, the loop exits and returns the final
/// rendered frame as a string.
///
/// This exercises the same key dispatch logic as the real `run()` function.
pub fn run_headless<C: Component>(
    root: C,
    width: u16,
    height: u16,
    events: Vec<Event>,
) -> String {
    let terminal = Terminal::new_headless(width, height);
    let event_source = testing::TestEventSource::new(events);
    let _ = run_inner(root, terminal, &event_source);
    event_source.last_buffer()
}

/// Inner event loop shared by `run()` and `run_headless()`.
fn run_inner<C: Component, E: EventSource>(
    root: C,
    mut terminal: Terminal,
    event_source: &E,
) -> Result<()> {
    let mut focus = FocusManager::new();
    let storage = Rc::new(StateStorage::new());
    let commands = Rc::new(CommandRegistry::new());
    let context = Rc::new(ContextStorage::new());
    let debug_mode = is_debug_mode();

    let mut frame_count = 0u64;
    let mut needs_render = true; // Always render on first frame
    let wake_flag = storage.wake_flag().clone();

    loop {
        let render_start = std::time::Instant::now();

        // Decay effect cycle counter (sliding window for infinite loop detection)
        storage.decay_effect_counter();

        // Drain all registered channels (external events -> frame buffers)
        // Clear first, then drain so components see only this frame's messages.
        storage.clear_channels();
        storage.drain_channels();

        // Channel data means we need to render
        if storage.has_channel_data() {
            needs_render = true;
        }

        // Poll terminal output (before rendering, so we pick up any new data)
        focus.poll_terminals();

        // Compute poll timeout: 0ms if wake flag is set (external event arrived),
        // otherwise 16ms (~60fps). Reset the flag before polling.
        let woken = wake_flag.swap(false, Ordering::Acquire);
        let poll_timeout = if woken || needs_render {
            Duration::ZERO
        } else {
            Duration::from_millis(16)
        };

        // Skip render if nothing changed since last frame.
        // If input arrives during the skip-render poll, save it so we can
        // dispatch it after re-rendering (instead of dropping it on the floor).
        let mut pending_event: Option<Event> = None;
        if !needs_render {
            if let Some(event) = event_source.poll_event(poll_timeout)? {
                if let Event::Resize(_, _) = event {
                    needs_render = true;
                    continue;
                }
                // Input arrived — save it and fall through to render + dispatch
                pending_event = Some(event);
            } else {
                continue; // No input, no channel data, skip frame
            }
        }
        needs_render = false; // Reset for next frame; input/effects/channels will set it again

        // Clear command registry before each render
        commands.clear();

        // Create scope with existing storage, command registry, and context
        let cx = Scope::with_all(
            Rc::clone(&storage),
            Rc::clone(&commands),
            Rc::clone(&context),
        );

        // Render the view
        let view = root.render(cx);

        // Collect focusables for navigation
        focus.collect_focusables(&view);

        // Set default wrap width for text areas based on terminal width
        // (subtract 2 for TextArea borders)
        focus.set_default_textarea_wrap_width(terminal.width().saturating_sub(2));

        let render_time = render_start.elapsed();
        frame_count += 1;

        // Get scroll and cursor offsets for all focusables
        let scroll_offsets: Vec<(u16, u16)> = (0..focus.focus_index() + 10)
            .map(|i| focus.scroll_offset(i))
            .collect();
        let cursor_offsets: Vec<usize> = (0..focus.focus_index() + 10)
            .map(|i| focus.cursor_offset(i))
            .collect();

        // Check if modal is visible for render context
        let modal_visible = has_visible_modal(&view);

        // Draw with focus and scroll info, get back clamped offsets
        let clamped_offsets = terminal.draw(
            &view,
            focus.focus_index(),
            focus.is_focus_visible(),
            scroll_offsets,
            cursor_offsets,
            modal_visible,
        )?;
        focus.update_scroll_states(&clamped_offsets);

        // Draw debug info if enabled
        if debug_mode {
            terminal.draw_debug(
                frame_count,
                render_time.as_micros() as u64,
                focus.focus_index(),
                focus.focusable_count(),
            )?;
        }

        // Run pending effects (after render, before input handling)
        // If effects ran and potentially modified state, re-render once
        if storage.flush_effects() {
            // Effects ran - re-render to show any state changes they made
            // Only do this once per frame to prevent infinite loops
            needs_render = true;
            let cx = Scope::with_all(
                Rc::clone(&storage),
                Rc::clone(&commands),
                Rc::clone(&context),
            );
            let view = root.render(cx);
            focus.collect_focusables(&view);
            let scroll_offsets: Vec<(u16, u16)> = (0..focus.focus_index() + 10)
                .map(|i| focus.scroll_offset(i))
                .collect();
            let cursor_offsets: Vec<usize> = (0..focus.focus_index() + 10)
                .map(|i| focus.cursor_offset(i))
                .collect();
            let modal_visible = has_visible_modal(&view);
            let clamped_offsets = terminal.draw(
                &view,
                focus.focus_index(),
                focus.is_focus_visible(),
                scroll_offsets,
                cursor_offsets,
                modal_visible,
            )?;
            focus.update_scroll_states(&clamped_offsets);
            // Don't flush effects again - just one re-render per frame
        }

        // Store current buffer for test retrieval (headless mode)
        event_source.on_frame_rendered(&terminal);

        // For now, use a generous max_scroll to allow scrolling
        // TODO: Calculate actual content height for focused scrollable
        let max_scroll = 100u16;
        let viewport_height = terminal.height().saturating_sub(6); // Approximate visible rows

        // Handle input — use saved event from skip-render poll, or poll fresh
        let input_event = if pending_event.is_some() {
            pending_event.take()
        } else {
            event_source.poll_event(Duration::from_millis(16))?
        };
        if let Some(event) = input_event {
            // Any input means we should re-render next frame
            needs_render = true;

            // Handle resize - just continue to trigger redraw
            if let Event::Resize(_, _) = event {
                continue;
            }

            if let Event::Key(key) = event {
                // Check if a modal is visible - if so, only allow Escape
                let modal_visible = has_visible_modal(&view);
                let palette_visible = has_visible_command_palette(&view);

                // When modal is visible, Escape dismisses it
                // Other keys work normally (focus is scoped to modal content)
                if modal_visible && key.code == KeyCode::Esc && key.modifiers == KeyModifiers::NONE
                {
                    call_modal_dismiss(&view);
                    continue;
                }

                // Handle command palette input when visible
                if palette_visible {
                    match (key.modifiers, key.code) {
                        (KeyModifiers::NONE, KeyCode::Esc) => {
                            call_command_palette_dismiss(&view);
                        }
                        (KeyModifiers::NONE, KeyCode::Enter) => {
                            if focus.is_focused_command_palette() {
                                focus.command_palette_execute();
                            }
                        }
                        (KeyModifiers::NONE, KeyCode::Up) => {
                            // Navigate up in palette - handled by state in component
                        }
                        (KeyModifiers::NONE, KeyCode::Down) => {
                            // Navigate down in palette - handled by state in component
                        }
                        (KeyModifiers::NONE, KeyCode::Backspace) => {
                            if focus.is_focused_command_palette() {
                                focus.command_palette_backspace();
                            }
                        }
                        (KeyModifiers::NONE, KeyCode::Char(c)) => {
                            if focus.is_focused_command_palette() {
                                focus.command_palette_key(c);
                            }
                        }
                        (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                            if focus.is_focused_command_palette() {
                                focus.command_palette_key(c.to_ascii_uppercase());
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // Escape closes open menu bar dropdowns
                if key.code == KeyCode::Esc && key.modifiers == KeyModifiers::NONE
                    && focus.is_focused_menu_bar() && focus.menu_bar_has_open_menu() {
                    focus.menu_bar_close();
                    continue;
                }

                // First, try user-registered commands
                if commands.execute(key.code, key.modifiers) {
                    continue;
                }

                match (key.modifiers, key.code) {
                    // Ctrl+Q to quit (but not Ctrl+C, as that should pass through to terminal)
                    (m, KeyCode::Char('q')) if m.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    // Ctrl+Shift+[ to escape terminal focus
                    (m, KeyCode::Char('['))
                        if m.contains(KeyModifiers::CONTROL)
                            && m.contains(KeyModifiers::SHIFT) =>
                    {
                        if focus.is_focused_terminal() {
                            focus.focus_next();
                        }
                    }
                    // Terminal passthrough - send all keys to terminal if focused
                    _ if focus.is_focused_terminal() => {
                        if let Err(e) = focus.terminal_key(key) {
                            eprintln!("Terminal input error: {}", e);
                        }
                    }
                    // Tab to focus next
                    (KeyModifiers::NONE, KeyCode::Tab) => {
                        focus.focus_next();
                    }
                    // Shift+Tab to focus previous
                    (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                        focus.focus_prev();
                    }
                    // Enter or Space to activate (for buttons, checkboxes, tree, table, menu bar)
                    (KeyModifiers::NONE, KeyCode::Enter | KeyCode::Char(' ')) => {
                        if focus.is_focused_text_area() {
                            if key.code == KeyCode::Enter {
                                focus.text_area_enter();
                            } else {
                                focus.text_area_key(' ');
                            }
                        } else if focus.is_focused_text_input() {
                            if key.code == KeyCode::Enter {
                                // Enter in text input submits
                                focus.text_input_submit();
                            } else {
                                // Space in text input adds a space
                                focus.text_input_key(' ');
                            }
                        } else if focus.is_focused_tree() {
                            focus.tree_activate();
                        } else if focus.is_focused_table() {
                            focus.table_activate();
                        } else if focus.is_focused_menu_bar() {
                            if focus.menu_bar_has_open_menu() {
                                // Execute selected item
                                focus.menu_bar_execute();
                            } else {
                                // Open first menu
                                focus.menu_bar_open();
                            }
                        } else {
                            focus.activate();
                        }
                    }
                    // Backspace for text input/area/form field
                    (KeyModifiers::NONE, KeyCode::Backspace) => {
                        if focus.is_focused_text_input() {
                            focus.text_input_backspace();
                        } else if focus.is_focused_text_area() {
                            focus.text_area_backspace();
                        } else if focus.is_focused_form_field() {
                            focus.form_field_backspace();
                        }
                    }
                    // Arrow keys for scrolling (when focused on scrollable) or list/tree/table/radio/textarea/menu/text input navigation
                    (KeyModifiers::NONE, KeyCode::Up) => {
                        if focus.is_focused_text_input() {
                            focus.text_input_key_up();
                        } else if focus.is_focused_text_area() {
                            focus.text_area_cursor_up();
                        } else if focus.is_focused_menu_bar() && focus.menu_bar_has_open_menu() {
                            focus.menu_bar_select_prev();
                        } else if focus.is_focused_scrollable() {
                            // For auto_scroll_bottom, Up means scroll away from bottom (increase offset)
                            if focus.is_focused_auto_scroll_bottom() {
                                focus.scroll_down(1, max_scroll);
                            } else {
                                focus.scroll_up(1);
                            }
                        } else if focus.is_focused_list() {
                            focus.list_select_prev();
                        } else if focus.is_focused_tree() {
                            focus.tree_select_prev();
                        } else if focus.is_focused_table() {
                            focus.table_select_prev();
                        } else if focus.is_focused_radio_group() {
                            focus.radio_group_select_prev();
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Down) => {
                        if focus.is_focused_text_input() {
                            focus.text_input_key_down();
                        } else if focus.is_focused_text_area() {
                            focus.text_area_cursor_down();
                        } else if focus.is_focused_menu_bar() && focus.menu_bar_has_open_menu() {
                            focus.menu_bar_select_next();
                        } else if focus.is_focused_scrollable() {
                            // For auto_scroll_bottom, Down means scroll toward bottom (decrease offset)
                            if focus.is_focused_auto_scroll_bottom() {
                                focus.scroll_up(1);
                            } else {
                                focus.scroll_down(1, max_scroll);
                            }
                        } else if focus.is_focused_list() {
                            focus.list_select_next();
                        } else if focus.is_focused_tree() {
                            focus.tree_select_next();
                        } else if focus.is_focused_table() {
                            focus.table_select_next();
                        } else if focus.is_focused_radio_group() {
                            focus.radio_group_select_next();
                        }
                    }
                    // Page Up/Down
                    (KeyModifiers::NONE, KeyCode::PageUp) => {
                        if focus.is_focused_scrollable() {
                            if focus.is_focused_auto_scroll_bottom() {
                                focus.scroll_down(viewport_height, max_scroll);
                            } else {
                                focus.scroll_up(viewport_height);
                            }
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::PageDown) => {
                        if focus.is_focused_scrollable() {
                            if focus.is_focused_auto_scroll_bottom() {
                                focus.scroll_up(viewport_height);
                            } else {
                                focus.scroll_down(viewport_height, max_scroll);
                            }
                        }
                    }
                    // Home/End
                    (KeyModifiers::NONE, KeyCode::Home) => {
                        if focus.is_focused_scrollable() {
                            // For auto_scroll_bottom, Home goes to top (max offset from bottom)
                            if focus.is_focused_auto_scroll_bottom() {
                                focus.scroll_end(max_scroll);
                            } else {
                                focus.scroll_home();
                            }
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::End) => {
                        if focus.is_focused_scrollable() {
                            // For auto_scroll_bottom, End goes to bottom (zero offset)
                            if focus.is_focused_auto_scroll_bottom() {
                                focus.scroll_home();
                            } else {
                                focus.scroll_end(max_scroll);
                            }
                        }
                    }
                    // Left/Right arrows for text inputs, text areas, tabs, tree, and menu bar
                    (KeyModifiers::NONE, KeyCode::Left) => {
                        if focus.is_focused_text_input() {
                            focus.text_input_cursor_left();
                        } else if focus.is_focused_text_area() {
                            focus.text_area_cursor_left();
                        } else if focus.is_focused_menu_bar() {
                            if focus.menu_bar_has_open_menu() {
                                focus.menu_bar_prev();
                            } else {
                                focus.menu_bar_highlight_prev();
                            }
                        } else if focus.is_focused_tabs() {
                            focus.tabs_select_prev();
                        } else if focus.is_focused_slider() {
                            focus.slider_decrement();
                        } else if focus.is_focused_tree() {
                            // Left triggers activate (app should collapse)
                            focus.tree_activate();
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Right) => {
                        if focus.is_focused_text_input() {
                            focus.text_input_cursor_right();
                        } else if focus.is_focused_text_area() {
                            focus.text_area_cursor_right();
                        } else if focus.is_focused_menu_bar() {
                            if focus.menu_bar_has_open_menu() {
                                focus.menu_bar_next();
                            } else {
                                focus.menu_bar_highlight_next();
                            }
                        } else if focus.is_focused_tabs() {
                            focus.tabs_select_next();
                        } else if focus.is_focused_slider() {
                            focus.slider_increment();
                        } else if focus.is_focused_tree() {
                            // Right triggers activate (app should expand)
                            focus.tree_activate();
                        }
                    }
                    // Character input for text fields, tabs, tree, and form fields
                    (KeyModifiers::NONE, KeyCode::Char(c)) => {
                        if focus.is_focused_text_input() {
                            focus.text_input_key(c);
                        } else if focus.is_focused_text_area() {
                            focus.text_area_key(c);
                        } else if focus.is_focused_form_field() {
                            focus.form_field_key(c);
                        } else if focus.is_focused_tabs() {
                            // Handle [ ] for tab cycling and 1-9 for direct selection
                            match c {
                                '[' => focus.tabs_select_prev(),
                                ']' => focus.tabs_select_next(),
                                '1'..='9' => {
                                    let idx = (c as usize) - ('1' as usize);
                                    focus.tabs_select(idx);
                                }
                                _ => {}
                            }
                        } else if focus.is_focused_tree() {
                            // j/k for vim-style navigation, space for activate
                            match c {
                                'j' => focus.tree_select_next(),
                                'k' => focus.tree_select_prev(),
                                ' ' => focus.tree_activate(),
                                _ => {}
                            }
                        } else if focus.is_focused_table() {
                            // j/k for vim-style navigation
                            match c {
                                'j' => focus.table_select_next(),
                                'k' => focus.table_select_prev(),
                                _ => {}
                            }
                        } else if focus.is_focused_radio_group() {
                            // j/k for vim-style navigation
                            match c {
                                'j' => focus.radio_group_select_next(),
                                'k' => focus.radio_group_select_prev(),
                                _ => {}
                            }
                        }
                    }
                    (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                        if focus.is_focused_text_input() {
                            focus.text_input_key(c.to_ascii_uppercase());
                        } else if focus.is_focused_text_area() {
                            focus.text_area_key(c.to_ascii_uppercase());
                        } else if focus.is_focused_form_field() {
                            focus.form_field_key(c.to_ascii_uppercase());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Run all effect cleanup functions before exiting
    storage.cleanup_all_effects();

    terminal.cleanup()?;
    Ok(())
}
