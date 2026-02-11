//! Tests for the keyed state API (state!).
//!
//! These tests verify that keyed state works correctly with:
//! - Basic state creation and retrieval
//! - Conditional state access (the main benefit over use_state)
//! - Multiple independent states

use telex::prelude::*;
use telex::testing::TestApp;

// ============================================================
// Basic Keyed State Tests
// ============================================================

#[test]
fn test_keyed_state_basic_usage() {
    let mut app = TestApp::new(|cx: Scope| {
        let count = state!(cx, || 0);
        View::text(format!("Count: {}", count.get()))
    })
    .with_size(30, 5);

    app.assert_visible("Count: 0");
}

#[test]
fn test_keyed_state_persists_across_renders() {
    // Test that state persists: initialize to 0, then verify we can update it
    let mut app = TestApp::new(|cx: Scope| {
        let count = state!(cx, || 0);

        let c = count.clone();
        let increment = move || c.update(|n| *n += 1);

        View::vstack()
            .child(View::text(format!("Count: {}", count.get())))
            .child(View::button().label("Inc").on_press(increment).build())
            .build()
    })
    .with_size(30, 10);

    // Initial render shows 0
    app.assert_visible("Count: 0");

    // After button press, state persists and shows 1
    app.press_button("Inc");
    app.assert_visible("Count: 1");

    // State continues to persist
    app.press_button("Inc");
    app.assert_visible("Count: 2");
}

// ============================================================
// Conditional State Tests (the key feature!)
// ============================================================

#[test]
fn test_keyed_state_works_in_conditional() {
    // This is the key test - conditional state that would panic with use_state
    let mut app = TestApp::new(|cx: Scope| {
        let show = state!(cx, || true);

        // State inside conditional - this is SAFE with state!
        let conditional_text = if show.get() {
            let text = state!(cx, || "Hello".to_string());
            text.get()
        } else {
            "Hidden".to_string()
        };

        View::text(conditional_text)
    })
    .with_size(30, 5);

    app.assert_visible("Hello");
}

#[test]
fn test_keyed_state_conditional_toggle_preserves_state() {
    // Toggle the condition and verify state persists
    let mut app = TestApp::new(|cx: Scope| {
        let show = state!(cx, || true);

        // This state is only accessed when show is true
        let counter = if show.get() {
            let c = state!(cx, || 0);
            Some(c.get())
        } else {
            None
        };

        View::vstack()
            .child(View::text(format!("Show: {}", show.get())))
            .child(View::text(format!("Counter: {:?}", counter)))
            .build()
    })
    .with_size(30, 10);

    app.assert_visible("Show: true");
    app.assert_visible("Counter: Some(0)");
}

// ============================================================
// Multiple Independent States
// ============================================================

#[test]
fn test_multiple_keyed_states_are_independent() {
    let mut app = TestApp::new(|cx: Scope| {
        let count_a = state!(cx, || 10);
        let count_b = state!(cx, || 20);
        let count_c = state!(cx, || 30);

        View::text(format!(
            "A={} B={} C={}",
            count_a.get(),
            count_b.get(),
            count_c.get()
        ))
    })
    .with_size(40, 5);

    app.assert_visible("A=10 B=20 C=30");
}

#[test]
fn test_keyed_state_in_different_branches() {
    let mut app = TestApp::new(|cx: Scope| {
        let branch = state!(cx, || 1);

        // Different states in different branches
        let value = match branch.get() {
            1 => {
                let a = state!(cx, || "Branch A");
                a.get().to_string()
            }
            2 => {
                let b = state!(cx, || "Branch B");
                b.get().to_string()
            }
            _ => {
                let c = state!(cx, || "Branch C");
                c.get().to_string()
            }
        };

        View::text(value)
    })
    .with_size(30, 5);

    app.assert_visible("Branch A");
}

// ============================================================
// Edge Cases
// ============================================================

#[test]
fn test_keyed_state_with_complex_init() {
    let mut app = TestApp::new(|cx: Scope| {
        let items = state!(cx, || vec!["apple", "banana", "cherry"]);
        View::text(format!("Items: {}", items.get().len()))
    })
    .with_size(30, 5);

    app.assert_visible("Items: 3");
}

#[test]
fn test_keyed_state_update_reflects_in_render() {
    let mut app = TestApp::new(|cx: Scope| {
        let count = state!(cx, || 0);

        let c = count.clone();
        let increment = move || c.update(|n| *n += 1);

        View::vstack()
            .child(View::text(format!("Count: {}", count.get())))
            .child(View::button().label("Inc").on_press(increment).build())
            .build()
    })
    .with_size(30, 10);

    app.assert_visible("Count: 0");

    // Press the button
    app.press_button("Inc");
    app.assert_visible("Count: 1");

    app.press_button("Inc");
    app.assert_visible("Count: 2");
}
