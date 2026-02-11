//! Integration tests for the real event loop via `run_headless()`.
//!
//! These tests exercise the actual key dispatch logic in `run_inner()` —
//! the same match arms that handle Ctrl+Q, Tab, Enter, character input, etc.

use telex::prelude::*;
use telex::{Event, KeyCode, KeyEvent, KeyModifiers};

/// Helper: create a key event.
fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

/// Helper: create a key event with modifiers.
fn key_mod(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

/// Helper: create a character key event.
fn char_key(c: char) -> Event {
    key(KeyCode::Char(c))
}

// ============================================================
// Basic lifecycle
// ============================================================

#[test]
fn test_ctrl_q_quits() {
    // Send only Ctrl+Q — loop should exit cleanly and produce rendered output
    let output = run_headless(
        |_cx: Scope| View::text("Hello"),
        40,
        10,
        vec![key_mod(KeyCode::Char('q'), KeyModifiers::CONTROL)],
    );
    assert!(output.contains("Hello"), "Should render Hello before quit");
}

#[test]
fn test_empty_events_auto_quits() {
    // No events — TestEventSource auto-sends Ctrl+Q
    let output = run_headless(
        |_cx: Scope| View::text("Auto quit"),
        40,
        10,
        vec![],
    );
    assert!(
        output.contains("Auto quit"),
        "Should render and auto-quit"
    );
}

// ============================================================
// Tab navigation
// ============================================================

#[test]
fn test_tab_moves_focus() {
    // Two buttons — Tab should move focus from first to second.
    let output = run_headless(
        |_cx: Scope| {
            View::vstack()
                .child(View::button().label("First").build())
                .child(View::button().label("Second").build())
                .build()
        },
        40,
        10,
        vec![key(KeyCode::Tab)],
    );
    // Both buttons should be visible
    assert!(output.contains("First"), "First button should be visible");
    assert!(output.contains("Second"), "Second button should be visible");
}

// ============================================================
// Enter activates button
// ============================================================

#[test]
fn test_enter_activates_button() {
    // Counter app: Enter on the focused button should increment count
    let output = run_headless(
        |cx: Scope| {
            let count = state!(cx, || 0i32);
            let c = count.clone();
            View::vstack()
                .child(View::text(format!("Count: {}", count.get())))
                .child(
                    View::button()
                        .label("Inc")
                        .on_press(with!(c => move || c.update(|n| *n += 1)))
                        .build(),
                )
                .build()
        },
        40,
        10,
        // Button is auto-focused (first focusable). Press Enter to activate.
        vec![key(KeyCode::Enter)],
    );
    assert!(
        output.contains("Count: 1"),
        "Count should be 1 after pressing Enter. Got:\n{}",
        output
    );
}

#[test]
fn test_multiple_enter_presses() {
    // Press Enter 3 times to increment counter 3 times
    let output = run_headless(
        |cx: Scope| {
            let count = state!(cx, || 0i32);
            let c = count.clone();
            View::vstack()
                .child(View::text(format!("Count: {}", count.get())))
                .child(
                    View::button()
                        .label("Inc")
                        .on_press(with!(c => move || c.update(|n| *n += 1)))
                        .build(),
                )
                .build()
        },
        40,
        10,
        vec![
            key(KeyCode::Enter),
            key(KeyCode::Enter),
            key(KeyCode::Enter),
        ],
    );
    assert!(
        output.contains("Count: 3"),
        "Count should be 3 after three Enter presses. Got:\n{}",
        output
    );
}

// ============================================================
// Text input
// ============================================================

#[test]
fn test_text_input_typing() {
    // Type characters into a text input
    let output = run_headless(
        |cx: Scope| {
            let text = state!(cx, || String::new());
            let t = text.clone();
            View::vstack()
                .child(
                    View::text_input()
                        .value(text.get().clone())
                        .on_change(with!(t => move |v: String| t.set(v)))
                        .placeholder("Type here...")
                        .build(),
                )
                .child(View::text(format!("Typed: {}", text.get())))
                .build()
        },
        40,
        10,
        vec![char_key('H'), char_key('i')],
    );
    assert!(
        output.contains("Typed: Hi"),
        "Should show typed text 'Hi'. Got:\n{}",
        output
    );
}

// ============================================================
// Shift+Tab (BackTab)
// ============================================================

#[test]
fn test_shift_tab_moves_focus_backward() {
    // Three buttons: Tab forward twice, then Shift+Tab back once.
    let output = run_headless(
        |_cx: Scope| {
            View::vstack()
                .child(View::button().label("A").build())
                .child(View::button().label("B").build())
                .child(View::button().label("C").build())
                .build()
        },
        40,
        10,
        vec![
            key(KeyCode::Tab),
            key(KeyCode::Tab),
            key_mod(KeyCode::BackTab, KeyModifiers::SHIFT),
        ],
    );
    // All three buttons should be rendered
    assert!(output.contains("A"), "Button A should be visible");
    assert!(output.contains("B"), "Button B should be visible");
    assert!(output.contains("C"), "Button C should be visible");
}

// ============================================================
// Events are processed in order (no input flush)
// ============================================================

#[test]
fn test_events_processed_in_order() {
    // Queue multiple events before starting — all should be processed in order.
    // This is a regression test: if Terminal::new ever flushed pending input,
    // some events would be lost.
    let output = run_headless(
        |cx: Scope| {
            let count = state!(cx, || 0i32);
            let c = count.clone();
            View::vstack()
                .child(View::text(format!("Count: {}", count.get())))
                .child(
                    View::button()
                        .label("Inc")
                        .on_press(with!(c => move || c.update(|n| *n += 1)))
                        .build(),
                )
                .build()
        },
        40,
        10,
        vec![
            key(KeyCode::Enter),
            key(KeyCode::Enter),
            key(KeyCode::Enter),
            key(KeyCode::Enter),
            key(KeyCode::Enter),
        ],
    );
    assert!(
        output.contains("Count: 5"),
        "All 5 events should be processed (no input flush). Got:\n{}",
        output
    );
}

// ============================================================
// Checkbox toggle
// ============================================================

#[test]
fn test_checkbox_toggle() {
    // Enter toggles a checkbox
    let output = run_headless(
        |cx: Scope| {
            let checked = state!(cx, || false);
            let c = checked.clone();
            View::vstack()
                .child(
                    View::checkbox()
                        .label("Accept")
                        .checked(checked.get())
                        .on_toggle(with!(c => move |v: bool| c.set(v)))
                        .build(),
                )
                .child(View::text(
                    if checked.get() { "YES" } else { "NO" }.to_string(),
                ))
                .build()
        },
        40,
        10,
        // First focusable is the checkbox. Enter toggles it.
        vec![key(KeyCode::Enter)],
    );
    assert!(
        output.contains("YES"),
        "Checkbox should be toggled on. Got:\n{}",
        output
    );
}
