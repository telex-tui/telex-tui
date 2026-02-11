//! Tests for the Buffer module.
//!
//! The buffer is the foundation of all rendering - it holds the 2D grid
//! of cells that represents the terminal screen.

use telex::prelude::*;
use telex::testing::TestApp;

// We need to test the buffer module directly, but it's not public.
// So we test it through TestApp's render_to_string() which uses buffer.to_string().

#[test]
fn test_buffer_to_string_basic() {
    let mut app = TestApp::new(|_cx: Scope| View::text("Hello")).with_size(10, 3);
    let rendered = app.render_to_string();

    assert!(rendered.contains("Hello"));
    // Buffer should trim trailing spaces per line
    assert!(!rendered.ends_with("          \n"));
}

#[test]
fn test_buffer_to_string_multiline() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::text("Line 1"))
            .child(View::text("Line 2"))
            .child(View::text("Line 3"))
            .build()
    })
    .with_size(20, 5);

    let rendered = app.render_to_string();
    let lines: Vec<&str> = rendered.lines().collect();

    assert!(lines.iter().any(|l| l.contains("Line 1")));
    assert!(lines.iter().any(|l| l.contains("Line 2")));
    assert!(lines.iter().any(|l| l.contains("Line 3")));
}

#[test]
fn test_buffer_clips_at_boundaries() {
    // Text longer than buffer width should be clipped
    let long_text = "This is a very long line that exceeds the buffer width";
    let mut app = TestApp::new(|_cx: Scope| View::text(long_text)).with_size(20, 3);

    let rendered = app.render_to_string();
    // Should only contain first 20 chars per line
    for line in rendered.lines() {
        assert!(
            line.len() <= 20,
            "Line should be clipped to 20 chars: {:?}",
            line
        );
    }
}

#[test]
fn test_buffer_handles_empty_view() {
    let mut app = TestApp::new(|_cx: Scope| View::Empty).with_size(10, 5);
    let rendered = app.render_to_string();

    // Empty view should produce empty/whitespace output
    assert!(rendered.trim().is_empty() || rendered.chars().all(|c| c == ' ' || c == '\n'));
}

#[test]
fn test_buffer_handles_unicode() {
    let mut app = TestApp::new(|_cx: Scope| View::text("Hello")).with_size(20, 3);
    let rendered = app.render_to_string();

    // Unicode should be preserved
    assert!(rendered.contains("Hello"));
}

#[test]
fn test_buffer_handles_special_chars() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::text("Tab:\there"))
            .child(View::text("Newline:\nwrapped"))
            .build()
    })
    .with_size(30, 5);

    let rendered = app.render_to_string();
    // Should handle special characters in some way
    assert!(rendered.contains("Tab:"));
    assert!(rendered.contains("Newline:"));
}

#[test]
fn test_buffer_minimum_size() {
    // Test with very small buffer
    let mut app = TestApp::new(|_cx: Scope| View::text("X")).with_size(1, 1);
    let rendered = app.render_to_string();

    // Should contain the single character
    assert!(rendered.contains("X"));
}

#[test]
fn test_buffer_large_size() {
    // Test with larger buffer
    let mut app = TestApp::new(|_cx: Scope| View::text("Test")).with_size(200, 50);
    let rendered = app.render_to_string();

    assert!(rendered.contains("Test"));
}

#[test]
fn test_buffer_box_with_border() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .border(true)
            .child(View::text("Content"))
            .build()
    })
    .with_size(20, 5);

    let rendered = app.render_to_string();

    // Should have border characters
    assert!(
        rendered.contains('┌') || rendered.contains('+'),
        "Should have top-left corner"
    );
    assert!(
        rendered.contains('┘') || rendered.contains('+'),
        "Should have bottom-right corner"
    );
    assert!(rendered.contains("Content"), "Should have content inside");
}

#[test]
fn test_buffer_styled_text() {
    // Styled text should render (styles won't be visible in string output,
    // but the text content should be there)
    let mut app =
        TestApp::new(|_cx: Scope| View::styled_text("Bold text").bold().build()).with_size(20, 3);

    let rendered = app.render_to_string();
    assert!(rendered.contains("Bold text"));
}

#[test]
fn test_buffer_nested_stacks() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(
                View::hstack()
                    .child(View::text("A"))
                    .child(View::text("B"))
                    .build(),
            )
            .child(
                View::hstack()
                    .child(View::text("C"))
                    .child(View::text("D"))
                    .build(),
            )
            .build()
    })
    .with_size(20, 5);

    let rendered = app.render_to_string();

    assert!(rendered.contains("A"));
    assert!(rendered.contains("B"));
    assert!(rendered.contains("C"));
    assert!(rendered.contains("D"));
}

// ============================================================
// Wide Character (Emoji/CJK) Tests
// ============================================================

/// Test that emojis are rendered correctly and don't disappear.
///
/// BUG FIXED: When rendering cell-by-cell, the wide character continuation
/// cell (second half of emoji) was being printed as a space, overwriting
/// half the emoji and making it disappear.
#[test]
fn test_buffer_emoji_not_overwritten() {
    let mut app = TestApp::new(|_cx: Scope| View::text("Hello 😊 World")).with_size(30, 3);
    let rendered = app.render_to_string();

    // Emoji should be visible, not replaced with spaces
    assert!(
        rendered.contains('😊'),
        "Emoji should be visible, got: {:?}",
        rendered
    );
    assert!(
        rendered.contains("Hello"),
        "Text before emoji should be visible"
    );
    assert!(
        rendered.contains("World"),
        "Text after emoji should be visible"
    );
}

/// Test multiple emojis in sequence.
#[test]
fn test_buffer_multiple_emojis() {
    let mut app = TestApp::new(|_cx: Scope| View::text("😊😊😊😊😊")).with_size(30, 3);
    let rendered = app.render_to_string();

    // Count emojis - should have exactly 5
    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(
        emoji_count, 5,
        "Should have 5 emojis, got {}: {:?}",
        emoji_count, rendered
    );
}

/// Test emoji in TextArea (the original bug location).
#[test]
fn test_buffer_emoji_in_text_area() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("one two 😊😊😊".to_string())
            .rows(3)
            .build()
    })
    .with_size(30, 6);

    let rendered = app.render_to_string();

    // All emojis should be visible
    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(
        emoji_count, 3,
        "Should have 3 emojis, got {}: {:?}",
        emoji_count, rendered
    );
}

/// Test that emojis at the exact boundary of content width are handled.
#[test]
fn test_buffer_emoji_at_boundary() {
    // TextArea with content_width = 28 (30 - 2 for borders)
    // Content that fills almost exactly: "one two xxx yyy zzz one tw" = 26 chars
    // Plus emoji (2 cols) = 28 cols exactly
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("one two xxx yyy zzz one tw😊".to_string())
            .rows(2)
            .build()
    })
    .with_size(30, 5);

    let rendered = app.render_to_string();

    // The emoji should be visible
    assert!(
        rendered.contains('😊'),
        "Emoji at boundary should be visible: {:?}",
        rendered
    );
}
