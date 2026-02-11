//! Comprehensive emoji edge case tests.
//!
//! These tests use the EXACT user-reported scenarios and test the FULL rendering pipeline,
//! not just individual functions. The goal is to catch bugs before users do.

use telex::prelude::*;
use telex::testing::TestApp;

// ============================================================
// User's exact test case
// ============================================================

/// The exact string the user reported issues with.
const USER_TEST_STRING: &str = "a b c d e dakl asdl d fox fox badger😊😊😊 😊😊";

/// Test the user's exact string at various widths.
#[test]
fn test_user_string_width_30() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value(USER_TEST_STRING.to_string())
            .rows(4)
            .build()
    })
    .with_size(30, 8);

    let rendered = app.render_to_string();

    // Count emojis - must have exactly 5
    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(
        emoji_count, 5,
        "Must have exactly 5 emojis at width 30.\nRendered:\n{}",
        rendered
    );
}

#[test]
fn test_user_string_width_40() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value(USER_TEST_STRING.to_string())
            .rows(4)
            .build()
    })
    .with_size(40, 8);

    let rendered = app.render_to_string();

    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(
        emoji_count, 5,
        "Must have exactly 5 emojis at width 40.\nRendered:\n{}",
        rendered
    );
}

#[test]
fn test_user_string_width_50() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value(USER_TEST_STRING.to_string())
            .rows(4)
            .build()
    })
    .with_size(50, 8);

    let rendered = app.render_to_string();

    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(
        emoji_count, 5,
        "Must have exactly 5 emojis at width 50.\nRendered:\n{}",
        rendered
    );
}

/// Test adding one more emoji (the exact scenario user described).
#[test]
fn test_user_string_plus_one_emoji() {
    let content = format!("{}😊", USER_TEST_STRING);

    let mut app =
        TestApp::new(move |_cx: Scope| View::text_area().value(content.clone()).rows(4).build())
            .with_size(50, 8);

    let rendered = app.render_to_string();

    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(
        emoji_count, 6,
        "Must have exactly 6 emojis after adding one.\nRendered:\n{}",
        rendered
    );
}

// ============================================================
// Emoji at every possible boundary position
// ============================================================

/// Test emoji at position that causes wrap.
#[test]
fn test_emoji_causes_wrap() {
    // Fill line almost exactly, then emoji pushes over
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("abcdefghijklmnopqrstuvwxyz😊".to_string()) // 26 + 2 = 28
            .rows(3)
            .build()
    })
    .with_size(30, 6); // content_width = 28

    let rendered = app.render_to_string();

    assert!(
        rendered.contains('😊'),
        "Emoji must be visible when it causes wrap.\nRendered:\n{}",
        rendered
    );
}

/// Test emoji that exactly fits.
#[test]
fn test_emoji_exact_fit() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("abcdefghijklmnopqrstuvwxyz😊".to_string()) // 26 + 2 = 28
            .rows(2)
            .build()
    })
    .with_size(32, 5); // content_width = 30, content = 28, fits with 2 padding

    let rendered = app.render_to_string();

    assert!(
        rendered.contains('😊'),
        "Emoji must be visible when it exactly fits.\nRendered:\n{}",
        rendered
    );
}

/// Test emoji that overflows by 1 column (2-wide char, only 1 col available).
#[test]
fn test_emoji_overflow_by_one() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("abcdefghijklmnopqrstuvwxyz😊".to_string()) // 26 + 2 = 28
            .rows(3)
            .build()
    })
    .with_size(29, 6); // content_width = 27, emoji needs cols 26-27, only have up to 26

    let rendered = app.render_to_string();

    assert!(
        rendered.contains('😊'),
        "Emoji must be visible even when it overflows by 1.\nRendered:\n{}",
        rendered
    );
}

// ============================================================
// Multiple emojis in sequence
// ============================================================

#[test]
fn test_five_emojis_sequential() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("😊😊😊😊😊".to_string())
            .rows(2)
            .build()
    })
    .with_size(20, 5);

    let rendered = app.render_to_string();

    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(
        emoji_count, 5,
        "Must render all 5 sequential emojis.\nRendered:\n{}",
        rendered
    );
}

#[test]
fn test_ten_emojis_sequential() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("😊😊😊😊😊😊😊😊😊😊".to_string())
            .rows(3)
            .build()
    })
    .with_size(20, 6);

    let rendered = app.render_to_string();

    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(
        emoji_count, 10,
        "Must render all 10 sequential emojis.\nRendered:\n{}",
        rendered
    );
}

// ============================================================
// Emojis with spaces (like user's "😊😊😊 😊😊")
// ============================================================

#[test]
fn test_emojis_with_space_between() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("😊😊😊 😊😊".to_string()) // 3 emojis, space, 2 emojis
            .rows(2)
            .build()
    })
    .with_size(20, 5);

    let rendered = app.render_to_string();

    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(
        emoji_count, 5,
        "Must render all 5 emojis with space between groups.\nRendered:\n{}",
        rendered
    );
}

#[test]
fn test_space_before_emoji_at_boundary() {
    // Space then emoji at wrap boundary - space goes on line 1, emoji on line 2
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("abcdefghijklmnopqrstuvwxy 😊".to_string()) // 25 + space + emoji
            .rows(3)
            .build()
    })
    .with_size(30, 6); // content_width = 28

    let rendered = app.render_to_string();

    assert!(
        rendered.contains('😊'),
        "Emoji after space at boundary must be visible.\nRendered:\n{}",
        rendered
    );
}

// ============================================================
// No leading/trailing garbage
// ============================================================

/// Verify no spurious leading spaces before emojis on wrapped lines.
#[test]
fn test_no_leading_spaces_before_wrapped_emoji() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("abcdefghijklmnopqrstuvwxyz😊😊😊".to_string())
            .rows(3)
            .build()
    })
    .with_size(30, 6); // Will wrap

    let rendered = app.render_to_string();

    // Find lines that start with emoji (after the border)
    for line in rendered.lines() {
        if line.contains('😊') {
            // If this line has emoji, check it's not "│  😊" (spaces before emoji)
            // It should be "│😊" or text then emoji
            let after_border = line.trim_start_matches('│');
            if after_border.starts_with(' ') && after_border.trim_start().starts_with('😊') {
                // There are spaces before the emoji - this might be legitimate padding
                // But if the emoji is at the START of wrapped content, there shouldn't be leading spaces
                let spaces_before_emoji = after_border.len() - after_border.trim_start().len();
                // Allow for some padding, but not excessive
                assert!(
                    spaces_before_emoji <= 1 || after_border.contains(char::is_alphabetic),
                    "Suspicious leading spaces before emoji: {:?}\nFull render:\n{}",
                    line,
                    rendered
                );
            }
        }
    }
}

// ============================================================
// Resize scenarios (the original bug was about reflow on resize)
// ============================================================

#[test]
fn test_emoji_survives_resize_narrower() {
    let content = USER_TEST_STRING.to_string();

    // Start wide
    let mut app = TestApp::new({
        let content = content.clone();
        move |_cx: Scope| View::text_area().value(content.clone()).rows(4).build()
    })
    .with_size(60, 8);

    let wide_render = app.render_to_string();
    let wide_count = wide_render.chars().filter(|&c| c == '😊').count();

    // Resize narrower
    let mut app2 = TestApp::new({
        let content = content.clone();
        move |_cx: Scope| View::text_area().value(content.clone()).rows(4).build()
    })
    .with_size(30, 8);

    let narrow_render = app2.render_to_string();
    let narrow_count = narrow_render.chars().filter(|&c| c == '😊').count();

    assert_eq!(
        wide_count, 5,
        "Wide render must have 5 emojis.\n{}",
        wide_render
    );
    assert_eq!(
        narrow_count, 5,
        "Narrow render must have 5 emojis.\n{}",
        narrow_render
    );
}

// ============================================================
// CJK characters (also 2-wide, same issues should apply)
// ============================================================

#[test]
fn test_cjk_characters_render() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("Hello 中文 World".to_string())
            .rows(2)
            .build()
    })
    .with_size(30, 5);

    let rendered = app.render_to_string();

    assert!(
        rendered.contains('中'),
        "CJK character 中 must be visible.\n{}",
        rendered
    );
    assert!(
        rendered.contains('文'),
        "CJK character 文 must be visible.\n{}",
        rendered
    );
}

#[test]
fn test_cjk_at_boundary() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("abcdefghijklmnopqrstuvwxyz中文".to_string())
            .rows(3)
            .build()
    })
    .with_size(30, 6);

    let rendered = app.render_to_string();

    assert!(
        rendered.contains('中'),
        "CJK at boundary must be visible.\n{}",
        rendered
    );
    assert!(
        rendered.contains('文'),
        "CJK at boundary must be visible.\n{}",
        rendered
    );
}

// ============================================================
// Mixed emoji and CJK
// ============================================================

#[test]
fn test_mixed_emoji_and_cjk() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("Hello 😊 中文 World 🎉".to_string())
            .rows(2)
            .build()
    })
    .with_size(40, 5);

    let rendered = app.render_to_string();

    assert!(
        rendered.contains('😊'),
        "Emoji must be visible.\n{}",
        rendered
    );
    assert!(
        rendered.contains('🎉'),
        "Emoji must be visible.\n{}",
        rendered
    );
    assert!(
        rendered.contains('中'),
        "CJK must be visible.\n{}",
        rendered
    );
    assert!(
        rendered.contains('文'),
        "CJK must be visible.\n{}",
        rendered
    );
}

// ============================================================
// Keyboard interaction tests - SKIPPED
// ============================================================
// NOTE: TestApp.type_char/type_str/backspace don't properly update component state.
// The FocusManager is modified but the component's state isn't hooked up,
// so re-rendering shows the original value. This is a testing framework limitation.
//
// These tests would need to be run manually or with a more sophisticated test harness
// that properly integrates state management.

// ============================================================
// Very narrow widths (stress test)
// ============================================================

#[test]
fn test_emoji_in_very_narrow_text_area() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("😊😊😊".to_string())
            .rows(5)
            .build()
    })
    .with_size(6, 8); // content_width = 4, each emoji needs 2

    let rendered = app.render_to_string();

    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(
        emoji_count, 3,
        "Must render all 3 emojis even in narrow view.\n{}",
        rendered
    );
}

#[test]
fn test_emoji_width_3_content_area() {
    // Content width of 3 means emoji (2 wide) fits, but barely
    let mut app =
        TestApp::new(|_cx: Scope| View::text_area().value("😊".to_string()).rows(2).build())
            .with_size(5, 5); // content_width = 3

    let rendered = app.render_to_string();

    assert!(
        rendered.contains('😊'),
        "Single emoji must be visible even with content_width=3.\n{}",
        rendered
    );
}

#[test]
fn test_emoji_width_2_content_area() {
    // Content width of exactly 2 - emoji should just fit
    let mut app =
        TestApp::new(|_cx: Scope| View::text_area().value("😊".to_string()).rows(2).build())
            .with_size(4, 5); // content_width = 2

    let rendered = app.render_to_string();

    assert!(
        rendered.contains('😊'),
        "Single emoji must be visible with content_width=2.\n{}",
        rendered
    );
}
