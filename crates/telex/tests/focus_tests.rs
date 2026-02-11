//! Tests for the Focus module.
//!
//! Covers navigation, state management, and widget interactions.
//! Note: Some TextArea tests are limited by TestApp infrastructure.

use telex::prelude::*;
use telex::testing::TestApp;

// ============================================================
// TextInput Tests (fully supported by TestApp)
// ============================================================

#[test]
fn test_text_input_typing() {
    let mut app = TestApp::new(|cx: Scope| {
        let text = cx.use_state(String::new);
        let txt = text.clone();

        View::vstack()
            .child(View::text(format!("Value: {}", text.get())))
            .child(
                View::text_input()
                    .value(text.get())
                    .on_change(move |s| txt.set(s))
                    .build(),
            )
            .build()
    });

    app.focus_next();
    app.type_str("Hello");

    assert!(app.has_text("Value: Hello"));
}

#[test]
fn test_text_input_backspace() {
    let mut app = TestApp::new(|cx: Scope| {
        let text = cx.use_state(|| "Test".to_string());
        let txt = text.clone();

        View::vstack()
            .child(View::text(format!("Value: {}", text.get())))
            .child(
                View::text_input()
                    .value(text.get())
                    .on_change(move |s| txt.set(s))
                    .build(),
            )
            .build()
    });

    app.focus_next();
    app.backspace();

    assert!(app.has_text("Value: Tes"));
}

#[test]
fn test_text_input_empty_backspace() {
    let mut app = TestApp::new(|cx: Scope| {
        let text = cx.use_state(String::new);
        let txt = text.clone();

        View::vstack()
            .child(View::text(format!("Len: {}", text.get().len())))
            .child(
                View::text_input()
                    .value(text.get())
                    .on_change(move |s| txt.set(s))
                    .build(),
            )
            .build()
    });

    app.focus_next();
    app.backspace(); // Should not panic on empty

    assert!(app.has_text("Len: 0"));
}

// ============================================================
// List Navigation Tests
// ============================================================

#[test]
fn test_list_navigation_wraps() {
    let mut app = TestApp::new(|cx: Scope| {
        let selected = cx.use_state(|| 0usize);
        let sel = selected.clone();

        View::vstack()
            .child(View::text(format!("Selected: {}", selected.get())))
            .child(
                View::list()
                    .items(vec!["A".to_string(), "B".to_string(), "C".to_string()])
                    .selected(selected.get())
                    .on_select(move |i| sel.set(i))
                    .build(),
            )
            .build()
    });

    app.focus_next(); // Focus list

    // Navigate down through all items
    app.list_down();
    assert!(app.has_text("Selected: 1"));

    app.list_down();
    assert!(app.has_text("Selected: 2"));

    // Should wrap to beginning
    app.list_down();
    assert!(app.has_text("Selected: 0"), "Should wrap to beginning");
}

#[test]
fn test_list_up_wraps() {
    let mut app = TestApp::new(|cx: Scope| {
        let selected = cx.use_state(|| 0usize);
        let sel = selected.clone();

        View::vstack()
            .child(View::text(format!("Selected: {}", selected.get())))
            .child(
                View::list()
                    .items(vec!["A".to_string(), "B".to_string(), "C".to_string()])
                    .selected(selected.get())
                    .on_select(move |i| sel.set(i))
                    .build(),
            )
            .build()
    });

    app.focus_next();

    // Going up from 0 should wrap to end
    app.list_up();
    assert!(app.has_text("Selected: 2"), "Should wrap to end");
}

#[test]
fn test_list_empty() {
    let mut app = TestApp::new(|cx: Scope| {
        let selected = cx.use_state(|| 0usize);
        let sel = selected.clone();

        View::list()
            .items(vec![])
            .selected(selected.get())
            .on_select(move |i| sel.set(i))
            .build()
    });

    // Empty list should have focusable but not crash on navigation
    assert_eq!(app.focusable_count(), 1);
}

// ============================================================
// Tree Navigation Tests
// ============================================================

#[test]
fn test_tree_navigation_initial() {
    let app = TestApp::new(|cx: Scope| {
        let selected = cx.use_state(|| vec![0usize]);
        let sel = selected.clone();

        let items = vec![
            TreeItem::new("Root 1")
                .expanded(true)
                .child(TreeItem::new("Child 1a"))
                .child(TreeItem::new("Child 1b")),
            TreeItem::new("Root 2"),
        ];

        View::vstack()
            .child(View::text(format!("Selected: {:?}", selected.get())))
            .child(
                View::tree()
                    .items(items)
                    .selected(selected.get().clone())
                    .on_select(move |path| sel.set(path))
                    .build(),
            )
            .build()
    });

    // Initial selection should be [0] (Root 1)
    assert!(app.has_text("Selected: [0]"));
}

#[test]
fn test_tree_focusable_count() {
    let mut app = TestApp::new(|_cx: Scope| {
        let items = vec![TreeItem::new("Root 1"), TreeItem::new("Root 2")];

        View::tree().items(items).selected(vec![0]).build()
    });

    // Tree should be one focusable element
    assert_eq!(app.focusable_count(), 1);
}

// ============================================================
// Table Navigation Tests
// ============================================================

#[test]
fn test_table_navigation_initial() {
    let app = TestApp::new(|cx: Scope| {
        let selected = cx.use_state(|| 0usize);
        let sel = selected.clone();

        let rows = vec![
            vec!["Row 1".to_string()],
            vec!["Row 2".to_string()],
            vec!["Row 3".to_string()],
        ];

        View::vstack()
            .child(View::text(format!("Selected: {}", selected.get())))
            .child(
                View::table()
                    .column("Name")
                    .rows(rows)
                    .selected(selected.get())
                    .on_select(move |i| sel.set(i))
                    .build(),
            )
            .build()
    });

    // Initial selection
    assert!(app.has_text("Selected: 0"));
}

#[test]
fn test_table_focusable_count() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::table()
            .column("Name")
            .rows(vec![vec!["Row 1".to_string()]])
            .build()
    });

    // Table should be one focusable element
    assert_eq!(app.focusable_count(), 1);
}

// ============================================================
// Tabs Tests
// ============================================================

#[test]
fn test_tabs_focusable_count() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::tabs()
            .tab("Tab 1", View::text("Content 1"))
            .tab("Tab 2", View::text("Content 2"))
            .active(0)
            .build()
    });

    // Tabs should be one focusable element
    assert_eq!(app.focusable_count(), 1);
}

#[test]
fn test_tabs_initial_state() {
    let app = TestApp::new(|cx: Scope| {
        let active = cx.use_state(|| 0usize);

        View::vstack()
            .child(View::text(format!("Active: {}", active.get())))
            .child(
                View::tabs()
                    .tab("Tab 1", View::text("Content One"))
                    .tab("Tab 2", View::text("Content Two"))
                    .active(active.get())
                    .build(),
            )
            .build()
    });

    // State should be tracked
    assert!(app.has_text("Active: 0"));
    // Note: has_text doesn't traverse into Tabs children yet
}

// ============================================================
// Focus Order Tests
// ============================================================

#[test]
fn test_focus_order_vstack() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::button().label("First").build())
            .child(View::button().label("Second").build())
            .child(View::button().label("Third").build())
            .build()
    });

    assert_eq!(app.focus_index(), 0);

    app.focus_next();
    assert_eq!(app.focus_index(), 1);

    app.focus_next();
    assert_eq!(app.focus_index(), 2);

    app.focus_next();
    assert_eq!(app.focus_index(), 0, "Should wrap around");
}

#[test]
fn test_focus_order_hstack() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::hstack()
            .child(View::button().label("Left").build())
            .child(View::button().label("Center").build())
            .child(View::button().label("Right").build())
            .build()
    });

    assert_eq!(app.focusable_count(), 3);
}

#[test]
fn test_focus_order_nested() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(
                View::hstack()
                    .child(View::button().label("A").build())
                    .child(View::button().label("B").build())
                    .build(),
            )
            .child(View::button().label("C").build())
            .build()
    });

    // Focus order should be depth-first: A, B, C
    assert_eq!(app.focusable_count(), 3);

    assert_eq!(app.focus_index(), 0); // A
    app.focus_next();
    assert_eq!(app.focus_index(), 1); // B
    app.focus_next();
    assert_eq!(app.focus_index(), 2); // C
}

#[test]
fn test_focus_prev_wraps() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::button().label("A").build())
            .child(View::button().label("B").build())
            .build()
    });

    assert_eq!(app.focus_index(), 0);
    app.focus_prev();
    assert_eq!(app.focus_index(), 1, "Should wrap to last element");
}

// ============================================================
// Button Activation Tests
// ============================================================

#[test]
fn test_button_activation() {
    let mut app = TestApp::new(|cx: Scope| {
        let count = cx.use_state(|| 0);
        let c = count.clone();

        View::vstack()
            .child(View::text(format!("Count: {}", count.get())))
            .child(
                View::button()
                    .label("Click")
                    .on_press(move || c.update(|n| *n += 1))
                    .build(),
            )
            .build()
    });

    assert!(app.has_text("Count: 0"));

    app.focus_next(); // Focus button
    app.activate();

    assert!(app.has_text("Count: 1"));
}

#[test]
fn test_button_press_by_label() {
    let mut app = TestApp::new(|cx: Scope| {
        let clicked = cx.use_state(String::new);
        let c1 = clicked.clone();
        let c2 = clicked.clone();

        View::vstack()
            .child(View::text(format!("Clicked: {}", clicked.get())))
            .child(
                View::button()
                    .label("Save")
                    .on_press(move || c1.set("Save".to_string()))
                    .build(),
            )
            .child(
                View::button()
                    .label("Cancel")
                    .on_press(move || c2.set("Cancel".to_string()))
                    .build(),
            )
            .build()
    });

    app.press_button("Save");
    assert!(app.has_text("Clicked: Save"));

    app.press_button("Cancel");
    assert!(app.has_text("Clicked: Cancel"));
}

// ============================================================
// Checkbox Tests
// ============================================================

#[test]
fn test_checkbox_toggle() {
    let mut app = TestApp::new(|cx: Scope| {
        let checked = cx.use_state(|| false);
        let chk = checked.clone();

        View::vstack()
            .child(View::text(format!("Checked: {}", checked.get())))
            .child(
                View::checkbox()
                    .checked(checked.get())
                    .label("Option")
                    .on_toggle(move |v| chk.set(v))
                    .build(),
            )
            .build()
    });

    assert!(app.has_text("Checked: false"));

    app.focus_next(); // Focus checkbox
    app.activate();

    assert!(app.has_text("Checked: true"));

    app.activate(); // Toggle again
    assert!(app.has_text("Checked: false"));
}

// ============================================================
// Scrollable Box Tests
// ============================================================

#[test]
fn test_scrollable_box_focusable() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .scroll(true)
            .child(View::text("Content"))
            .build()
    });

    // Scrollable box should be focusable
    assert_eq!(app.focusable_count(), 1);
}

#[test]
fn test_scroll_navigation() {
    let mut app = TestApp::new(|_cx: Scope| {
        let long_content = (1..=50)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");

        View::boxed()
            .scroll(true)
            .child(View::text(&long_content))
            .build()
    });

    // Should be able to scroll without panic
    app.scroll_down(5);
    app.scroll_up(2);
}

// ============================================================
// Mixed Focusables Tests
// ============================================================

#[test]
fn test_mixed_focusables() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::button().label("Button").build())
            .child(View::text_input().value(String::new()).build())
            .child(View::checkbox().checked(false).label("Check").build())
            .child(View::list().items(vec!["A".to_string()]).build())
            .build()
    });

    // Should have 4 focusable elements
    assert_eq!(app.focusable_count(), 4);
}

#[test]
fn test_non_focusable_views() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::text("Just text"))
            .child(View::gap(1))
            .child(View::boxed().child(View::text("Box content")).build())
            .build()
    });

    // Text, spacer, and non-scrollable box are not focusable
    assert_eq!(app.focusable_count(), 0);
}

// ============================================================
// Split Pane Focus Tests
// ============================================================

#[test]
fn test_split_focusables_in_panes() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::split()
            .horizontal()
            .first(View::button().label("Left").build())
            .second(View::button().label("Right").build())
            .build()
    });

    // Both buttons in split panes should be focusable
    assert_eq!(app.focusable_count(), 2);
}

// ============================================================
// TextArea Tests
// ============================================================

#[test]
fn test_text_area_typing() {
    let mut app = TestApp::new(|cx: Scope| {
        let text = cx.use_state(String::new);
        let len = cx.use_state(|| 0usize);
        let txt = text.clone();
        let l = len.clone();

        View::vstack()
            .child(View::text(format!("Len: {}", len.get())))
            .child(
                View::text_area()
                    .value(text.get())
                    .on_change(move |s| {
                        l.set(s.len());
                        txt.set(s);
                    })
                    .build(),
            )
            .build()
    });

    app.focus_next(); // Focus text area
    app.type_str("Hello");

    // Test that 5 characters were typed
    assert!(app.has_text("Len: 5"), "Expected 'Len: 5' to be found");
}

#[test]
fn test_text_area_wrap_at_width() {
    // Create a small app (20 columns wide) to test wrapping
    let mut app = TestApp::new(|cx: Scope| {
        let text = cx.use_state(String::new);
        let line_count = cx.use_state(|| 1usize);
        let txt = text.clone();
        let lc = line_count.clone();

        View::vstack()
            .child(View::text(format!("Lines: {}", line_count.get())))
            .child(
                View::text_area()
                    .value(text.get())
                    .on_change(move |s| {
                        let lines = s.lines().count().max(1);
                        lc.set(lines);
                        txt.set(s);
                    })
                    .build(),
            )
            .build()
    })
    .with_size(20, 10);

    app.focus_next(); // Focus text area

    // Type a string longer than the visual wrap width
    // Content should NOT auto-wrap - only Enter creates newlines
    // Visual wrapping is handled by the renderer, not by modifying content
    app.type_str("ABCDEFGHIJKLMNOP"); // 16 chars
    assert!(app.has_text("Lines: 1")); // Still 1 line (no auto-wrap)

    app.type_str("Q"); // 17th char - still no auto-wrap
    assert!(app.has_text("Lines: 1")); // Content stays as 1 line

    // Only Enter creates actual newlines
    app.enter();
    assert!(app.has_text("Lines: 2")); // Now 2 lines from Enter
}

#[test]
fn test_text_area_explicit_wrap_width() {
    let mut app = TestApp::new(|cx: Scope| {
        let text = cx.use_state(String::new);
        let line_count = cx.use_state(|| 1usize);
        let txt = text.clone();
        let lc = line_count.clone();

        View::vstack()
            .child(View::text(format!("Lines: {}", line_count.get())))
            .child(
                View::text_area()
                    .value(text.get())
                    .wrap_width(10) // Explicit wrap at 10 chars
                    .on_change(move |s| {
                        let lines = s.lines().count().max(1);
                        lc.set(lines);
                        txt.set(s);
                    })
                    .build(),
            )
            .build()
    });

    app.focus_next();

    // Type exactly 10 chars
    app.type_str("1234567890");
    assert!(app.has_text("Lines: 1"));

    // 11th char - no auto-wrap, content stays as 1 line
    // (Visual wrapping is handled by renderer, not by modifying content)
    app.type_str("A");
    assert!(app.has_text("Lines: 1")); // Content unchanged

    // Only Enter creates actual newlines
    app.enter();
    assert!(app.has_text("Lines: 2"));
}

/// This test proves the auto-wrap content corruption bug is fixed.
///
/// THE BUG: Previously, typing past wrap_width would insert '\n' into the
/// actual content. This meant:
/// 1. Words got split mid-character (e.g., "sc\nreen")
/// 2. Resizing the terminal wouldn't reflow text properly
/// 3. User's content was silently corrupted
///
/// THE FIX: Only Enter creates newlines. Wrapping is visual-only.
#[test]
fn test_auto_wrap_content_corruption_bug_fixed() {
    let mut app = TestApp::new(|cx: Scope| {
        let text = cx.use_state(String::new);
        let content = cx.use_state(String::new);
        let txt = text.clone();
        let cnt = content.clone();

        View::vstack()
            // Show actual content for debugging
            .child(View::text(format!("Content: [{}]", content.get())))
            // Show if content contains newlines
            .child(View::text(format!(
                "Has newlines: {}",
                content.get().contains('\n')
            )))
            .child(
                View::text_area()
                    .value(text.get())
                    .wrap_width(20) // Force narrow wrap width
                    .on_change(move |s| {
                        cnt.set(s.clone()); // Store raw content
                        txt.set(s);
                    })
                    .build(),
            )
            .build()
    });

    app.focus_next(); // Focus text area

    // Type a sentence that will visually wrap multiple times at width 20
    app.type_str("here is my text before I resize the screen");

    // CRITICAL: Content must NOT contain newlines
    // The old bug would have content like "here is my text bef\nore I resize the sc\nreen"
    assert!(
        app.has_text("Has newlines: false"),
        "Content must not contain newlines from auto-wrap"
    );

    // Type even more - still no newlines
    app.type_str(" and it keeps going with more text");
    assert!(
        app.has_text("Has newlines: false"),
        "Content still must not contain newlines"
    );

    // NOW press Enter - this SHOULD create a newline
    app.enter();
    assert!(
        app.has_text("Has newlines: true"),
        "Enter key should create a newline"
    );
}
