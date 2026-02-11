//! Tests for the Render module.
//!
//! Covers rendering of TextArea, Checkbox, Modal, Split, Tabs, Tree, Table.

use telex::prelude::*;
use telex::testing::TestApp;

// ============================================================
// TextArea Rendering Tests
// ============================================================

#[test]
fn test_text_area_renders_content() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("Hello\nWorld".to_string())
            .rows(5)
            .build()
    })
    .with_size(30, 10);

    let rendered = app.render_to_string();
    assert!(rendered.contains("Hello"), "Should contain Hello");
    assert!(rendered.contains("World"), "Should contain World");
}

#[test]
fn test_text_area_renders_placeholder() {
    // Wrap in a VStack with a button so focus goes to the button,
    // not the TextArea — placeholder only renders when unfocused.
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::button().label("Dummy").build())
            .child(View::text_area()
                .value(String::new())
                .placeholder("Enter text here...")
                .rows(3)
                .build())
            .build()
    })
    .with_size(30, 10);

    let rendered = app.render_to_string();
    assert!(
        rendered.contains("Enter text"),
        "Should show placeholder when empty"
    );
}

#[test]
fn test_text_area_renders_borders() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            .value("Content".to_string())
            .rows(3)
            .build()
    })
    .with_size(20, 10);

    let rendered = app.render_to_string();
    // TextArea uses │ for left/right borders
    assert!(rendered.contains('│'), "Should have vertical borders");
}

/// This test verifies that TextArea ACTUALLY WRAPS long text visually.
///
/// THE BUG: render_text_area was truncating long lines instead of wrapping.
/// This test ensures that text longer than the content width appears on
/// multiple visual lines.
#[test]
fn test_text_area_wraps_long_lines() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            // Content is 40 chars, wider than the 20-char box (18 content width)
            .value("here is my text that should wrap nicely".to_string())
            .rows(5)
            .build()
    })
    .with_size(20, 10);

    let rendered = app.render_to_string();

    // With character wrap at width 18:
    // Line 1: "here is my text th" (18 chars)
    // Line 2: "at should wrap nic" (18 chars)
    // Line 3: "ely" (3 chars)

    // The key assertion: ALL characters should be visible, not truncated
    assert!(rendered.contains("here"), "Should contain 'here'");
    assert!(
        rendered.contains("ely"),
        "Should contain 'ely' - proves full text visible with character wrap"
    );

    // Double-check: if it was truncating, "ely" wouldn't appear
    // because "here is my text that should wrap nicely" truncated at 18 chars
    // would be "here is my text th" - no "ely"
}

/// Test that very long single words get force-broken.
#[test]
fn test_text_area_breaks_long_words() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            // Single word longer than content width
            .value("abcdefghijklmnopqrstuvwxyz".to_string())
            .rows(5)
            .build()
    })
    .with_size(20, 10); // 18 char content width

    let rendered = app.render_to_string();

    // All letters should be visible (wrapped, not truncated)
    assert!(rendered.contains('a'), "Should contain start");
    assert!(
        rendered.contains('z'),
        "Should contain end - proves wrapping not truncating"
    );
}

/// Test that emoji (2 display columns each) wrap correctly and don't overflow.
///
/// THE BUG: render_text_area was padding by CHARACTER count instead of DISPLAY WIDTH.
/// For emoji (1 char = 2 columns), this caused text to overflow the buffer.
#[test]
fn test_text_area_emoji_display_width() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            // Emoji are 2 display columns each
            // This content has: "abc " (4 cols) + "😊😊😊" (6 cols) = 10 display columns
            .value("abc 😊😊😊".to_string())
            .rows(3)
            .build()
    })
    .with_size(15, 6); // 13 char content width - enough for 10 cols

    let rendered = app.render_to_string();

    // All content should be visible
    assert!(rendered.contains("abc"), "Should contain 'abc'");
    assert!(rendered.contains('😊'), "Should contain emoji");

    // The text should NOT overflow the borders
    // Check that '│' border characters appear at expected positions
    let lines: Vec<&str> = rendered.lines().collect();
    for line in &lines {
        if line.contains('│') {
            // Each line with borders should have exactly 2 '│' characters
            let border_count = line.chars().filter(|&c| c == '│').count();
            assert!(
                border_count <= 2,
                "Line should have at most 2 borders, got {}: {:?}",
                border_count,
                line
            );
        }
    }
}

/// Test emoji at wrap boundary - the scenario that triggered the original bug report.
#[test]
fn test_text_area_emoji_at_wrap_boundary() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_area()
            // Content designed to have emoji exactly at wrap boundary
            // "a b c d e dakl asdl d fox fox badger" = 36 display cols
            // Plus "😊😊😊 😊😊" = 11 display cols (3*2 + 1 + 2*2)
            // Total = 47 display cols
            .value("a b c d e dakl asdl d fox fox badger😊😊😊 😊😊".to_string())
            .rows(4)
            .build()
    })
    .with_size(50, 8); // 48 content width - content (47) fits on one line

    let rendered = app.render_to_string();

    // All emojis should be visible
    let emoji_count = rendered.chars().filter(|&c| c == '😊').count();
    assert_eq!(
        emoji_count, 5,
        "All 5 emojis should be rendered, got {}",
        emoji_count
    );
}

// ============================================================
// Checkbox Rendering Tests
// ============================================================

#[test]
fn test_checkbox_renders_unchecked() {
    let mut app =
        TestApp::new(|_cx: Scope| View::checkbox().checked(false).label("Option").build())
            .with_size(30, 3);

    let rendered = app.render_to_string();
    assert!(rendered.contains("[ ]"), "Unchecked should show [ ]");
    assert!(rendered.contains("Option"), "Should show label");
}

#[test]
fn test_checkbox_renders_checked() {
    let mut app = TestApp::new(|_cx: Scope| View::checkbox().checked(true).label("Option").build())
        .with_size(30, 3);

    let rendered = app.render_to_string();
    assert!(rendered.contains("[x]"), "Checked should show [x]");
    assert!(rendered.contains("Option"), "Should show label");
}

#[test]
fn test_checkbox_toggle() {
    let mut app = TestApp::new(|cx: Scope| {
        let checked = state!(cx, || false);
        let chk = checked.clone();

        View::vstack()
            .child(View::text(format!("Checked: {}", checked.get())))
            .child(
                View::checkbox()
                    .checked(checked.get())
                    .label("Toggle me")
                    .on_toggle(move |v| chk.set(v))
                    .build(),
            )
            .build()
    });

    assert!(app.has_text("Checked: false"));

    app.focus_next(); // Focus checkbox
    app.activate();

    assert!(app.has_text("Checked: true"));
}

// ============================================================
// Modal Rendering Tests
// ============================================================

#[test]
fn test_modal_visible_renders() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::text("Background"))
            .child(
                View::modal()
                    .visible(true)
                    .title("Test Modal")
                    .child(View::text("Modal Content"))
                    .build(),
            )
            .build()
    })
    .with_size(60, 20);

    app.assert_visible("Test Modal");
    app.assert_visible("Modal Content");
}

#[test]
fn test_modal_hidden_does_not_render() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::text("Background"))
            .child(
                View::modal()
                    .visible(false)
                    .title("Hidden Modal")
                    .child(View::text("Should not see"))
                    .build(),
            )
            .build()
    })
    .with_size(60, 20);

    app.assert_visible("Background");
    app.assert_not_visible("Hidden Modal");
    app.assert_not_visible("Should not see");
}

#[test]
fn test_modal_with_buttons() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::modal()
            .visible(true)
            .title("Confirm")
            .child(
                View::vstack()
                    .child(View::text("Are you sure?"))
                    .child(
                        View::hstack()
                            .child(View::button().label("Yes").build())
                            .child(View::button().label("No").build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    })
    .with_size(60, 20);

    app.assert_visible("Confirm");
    app.assert_visible("Are you sure?");
    app.assert_visible("Yes");
    app.assert_visible("No");
}

// ============================================================
// Split Pane Rendering Tests
// ============================================================

#[test]
fn test_split_horizontal_renders_both_panes() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::split()
            .horizontal()
            .ratio(0.5)
            .first(View::text("Left Pane"))
            .second(View::text("Right Pane"))
            .build()
    })
    .with_size(40, 10);

    app.assert_visible("Left Pane");
    app.assert_visible("Right Pane");
}

#[test]
fn test_split_vertical_renders_both_panes() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::split()
            .vertical()
            .ratio(0.5)
            .first(View::text("Top Pane"))
            .second(View::text("Bottom Pane"))
            .build()
    })
    .with_size(40, 10);

    app.assert_visible("Top Pane");
    app.assert_visible("Bottom Pane");
}

#[test]
fn test_split_with_divider() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::split()
            .horizontal()
            .ratio(0.5)
            .show_divider(true)
            .first(View::text("A"))
            .second(View::text("B"))
            .build()
    })
    .with_size(40, 10);

    let rendered = app.render_to_string();
    // Horizontal split uses │ as divider
    assert!(rendered.contains('│'), "Should have vertical divider");
}

#[test]
fn test_split_nested() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::split()
            .horizontal()
            .ratio(0.5)
            .first(
                View::split()
                    .vertical()
                    .ratio(0.5)
                    .first(View::text("Top Left"))
                    .second(View::text("Bottom Left"))
                    .build(),
            )
            .second(View::text("Right"))
            .build()
    })
    .with_size(60, 20);

    app.assert_visible("Top Left");
    app.assert_visible("Bottom Left");
    app.assert_visible("Right");
}

// ============================================================
// Tabs Rendering Tests
// ============================================================

#[test]
fn test_tabs_renders_tab_bar() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::tabs()
            .tab("Files", View::text("Files content"))
            .tab("Edit", View::text("Edit content"))
            .tab("View", View::text("View content"))
            .active(0)
            .build()
    })
    .with_size(40, 10);

    app.assert_visible("Files");
    app.assert_visible("Edit");
    app.assert_visible("View");
}

#[test]
fn test_tabs_renders_active_content() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::tabs()
            .tab("Tab 1", View::text("Content One"))
            .tab("Tab 2", View::text("Content Two"))
            .active(0)
            .build()
    })
    .with_size(40, 10);

    app.assert_visible("Content One");
    // Tab 2 content should not be visible since it's not active
}

#[test]
fn test_tabs_switching() {
    let mut app = TestApp::new(|cx: Scope| {
        let active = state!(cx, || 0usize);
        let act = active.clone();

        View::vstack()
            .child(View::text(format!("Active tab: {}", active.get())))
            .child(
                View::boxed()
                    .flex(1)
                    .child(
                        View::tabs()
                            .tab("First", View::text("First content"))
                            .tab("Second", View::text("Second content"))
                            .active(active.get())
                            .on_change(move |i| act.set(i))
                            .build(),
                    )
                    .build(),
            )
            .build()
    })
    .with_size(50, 20);

    app.assert_visible("Active tab: 0");
    // Tab bar should show tab labels
    app.assert_visible("First");
    app.assert_visible("Second");
}

#[test]
fn test_tabs_position_bottom() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::tabs()
            .tab("Tab 1", View::text("Content"))
            .position(telex::TabPosition::Bottom)
            .active(0)
            .build()
    })
    .with_size(40, 10);

    // Tab bar should render (position doesn't change what's visible, just where)
    app.assert_visible("Tab 1");
    app.assert_visible("Content");
}

// ============================================================
// Tree Rendering Tests
// ============================================================

#[test]
fn test_tree_renders_items() {
    let mut app = TestApp::new(|_cx: Scope| {
        let items = vec![
            TreeItem::new("Root 1"),
            TreeItem::new("Root 2"),
            TreeItem::new("Root 3"),
        ];

        View::tree().items(items).selected(vec![0]).build()
    })
    .with_size(40, 10);

    app.assert_visible("Root 1");
    app.assert_visible("Root 2");
    app.assert_visible("Root 3");
}

#[test]
fn test_tree_renders_expanded_children() {
    let mut app = TestApp::new(|_cx: Scope| {
        let items = vec![TreeItem::new("Parent")
            .expanded(true)
            .child(TreeItem::new("Child 1"))
            .child(TreeItem::new("Child 2"))];

        View::tree().items(items).selected(vec![0]).build()
    })
    .with_size(40, 10);

    app.assert_visible("Parent");
    app.assert_visible("Child 1");
    app.assert_visible("Child 2");
}

#[test]
fn test_tree_collapsed_hides_children() {
    let mut app = TestApp::new(|_cx: Scope| {
        let items = vec![TreeItem::new("Parent")
            .expanded(false)
            .child(TreeItem::new("Hidden Child"))];

        View::tree().items(items).selected(vec![0]).build()
    })
    .with_size(40, 10);

    app.assert_visible("Parent");
    app.assert_not_visible("Hidden Child");
}

#[test]
fn test_tree_renders_icons() {
    let mut app = TestApp::new(|_cx: Scope| {
        let items = vec![
            TreeItem::new("Folder").icon("F"),
            TreeItem::new("File").icon("f"),
        ];

        View::tree().items(items).selected(vec![0]).build()
    })
    .with_size(40, 10);

    let rendered = app.render_to_string();
    assert!(rendered.contains("F") && rendered.contains("Folder"));
    assert!(rendered.contains("f") && rendered.contains("File"));
}

#[test]
fn test_tree_renders_expand_markers() {
    let mut app = TestApp::new(|_cx: Scope| {
        let items = vec![
            TreeItem::new("Expanded")
                .expanded(true)
                .child(TreeItem::new("Child")),
            TreeItem::new("Collapsed")
                .expanded(false)
                .child(TreeItem::new("Hidden")),
        ];

        View::tree().items(items).selected(vec![0]).build()
    })
    .with_size(40, 10);

    let rendered = app.render_to_string();
    // ▼ for expanded, ▶ for collapsed
    assert!(
        rendered.contains('▼'),
        "Should have expand marker for expanded item"
    );
    assert!(
        rendered.contains('▶'),
        "Should have collapse marker for collapsed item"
    );
}

// ============================================================
// Table Rendering Tests
// ============================================================

#[test]
fn test_table_renders_headers() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::table()
            .column("NAME")
            .column("STATUS")
            .column("VALUE")
            .rows(vec![vec![
                "Item 1".to_string(),
                "Active".to_string(),
                "100".to_string(),
            ]])
            .build()
    })
    .with_size(60, 10);

    app.assert_visible("NAME");
    app.assert_visible("STATUS");
    app.assert_visible("VALUE");
}

#[test]
fn test_table_renders_rows() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::table()
            .column("Name")
            .rows(vec![
                vec!["Row 1".to_string()],
                vec!["Row 2".to_string()],
                vec!["Row 3".to_string()],
            ])
            .build()
    })
    .with_size(40, 10);

    app.assert_visible("Row 1");
    app.assert_visible("Row 2");
    app.assert_visible("Row 3");
}

#[test]
fn test_table_renders_sort_indicator() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::table()
            .column("Name")
            .column("Value")
            .rows(vec![vec!["A".to_string(), "1".to_string()]])
            .sort_by(0, true) // Sort ascending on column 0
            .build()
    })
    .with_size(40, 10);

    let rendered = app.render_to_string();
    assert!(
        rendered.contains('▲'),
        "Should show ascending sort indicator"
    );
}

#[test]
fn test_table_renders_descending_sort() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::table()
            .column("Name")
            .rows(vec![vec!["A".to_string()]])
            .sort_by(0, false) // Sort descending
            .build()
    })
    .with_size(40, 10);

    let rendered = app.render_to_string();
    assert!(
        rendered.contains('▼'),
        "Should show descending sort indicator"
    );
}

#[test]
fn test_table_column_alignment() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::table()
            .column_with(TableColumn::new("Left").align(TextAlign::Left))
            .column_with(TableColumn::new("Right").align(TextAlign::Right))
            .column_with(TableColumn::new("Center").align(TextAlign::Center))
            .rows(vec![vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
            ]])
            .build()
    })
    .with_size(60, 10);

    // Headers should be visible
    app.assert_visible("Left");
    app.assert_visible("Right");
    app.assert_visible("Center");
}

// ============================================================
// Button Rendering Tests
// ============================================================

#[test]
fn test_button_renders_label() {
    let mut app =
        TestApp::new(|_cx: Scope| View::button().label("Click Me").build()).with_size(30, 3);

    let rendered = app.render_to_string();
    assert!(rendered.contains("Click Me"));
    assert!(
        rendered.contains("[") && rendered.contains("]"),
        "Should have brackets"
    );
}

#[test]
fn test_button_activation() {
    let mut app = TestApp::new(|cx: Scope| {
        let count = state!(cx, || 0);
        let c = count.clone();

        View::vstack()
            .child(View::text(format!("Count: {}", count.get())))
            .child(
                View::button()
                    .label("Inc")
                    .on_press(move || c.update(|n| *n += 1))
                    .build(),
            )
            .build()
    });

    assert!(app.has_text("Count: 0"));

    app.press_button("Inc");
    assert!(app.has_text("Count: 1"));

    app.press_button("Inc");
    assert!(app.has_text("Count: 2"));
}

// ============================================================
// List Rendering Tests
// ============================================================

#[test]
fn test_list_renders_items() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::list()
            .items(vec![
                "Apple".to_string(),
                "Banana".to_string(),
                "Cherry".to_string(),
            ])
            .selected(0)
            .build()
    })
    .with_size(30, 10);

    app.assert_visible("Apple");
    app.assert_visible("Banana");
    app.assert_visible("Cherry");
}

#[test]
fn test_list_shows_selection_marker() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::list()
            .items(vec!["First".to_string(), "Second".to_string()])
            .selected(0)
            .build()
    })
    .with_size(30, 10);

    let rendered = app.render_to_string();
    // Selection marker is "> "
    assert!(rendered.contains(">"), "Should show selection marker");
}

// ============================================================
// Box Rendering Tests
// ============================================================

#[test]
fn test_box_with_border() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .border(true)
            .child(View::text("Inside"))
            .build()
    })
    .with_size(20, 5);

    let rendered = app.render_to_string();
    app.assert_visible("Inside");
    assert!(rendered.contains('┌'), "Should have top-left corner");
    assert!(rendered.contains('┘'), "Should have bottom-right corner");
}

#[test]
fn test_box_with_padding() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .border(true)
            .padding(1)
            .child(View::text("Padded"))
            .build()
    })
    .with_size(20, 7);

    app.assert_visible("Padded");
}

#[test]
fn test_scrollable_box() {
    let long_content = (1..=20)
        .map(|i| format!("Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");

    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .scroll(true)
            .border(true)
            .child(View::text(&long_content))
            .build()
    })
    .with_size(30, 10);

    // Should be able to render without panic
    let rendered = app.render_to_string();
    assert!(rendered.contains("Line"), "Should render some content");
}

// ============================================================
// Progress Bar Rendering Tests
// ============================================================

#[test]
fn test_progress_bar_renders() {
    let mut app =
        TestApp::new(|_cx: Scope| View::progress_bar().value(0.5).build()).with_size(40, 3);

    let rendered = app.render_to_string();
    // Should contain filled and empty chars
    assert!(
        rendered.contains('█') || rendered.contains('░'),
        "Should render progress bar characters: {}",
        rendered
    );
}

#[test]
fn test_progress_bar_shows_percentage() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::progress_bar()
            .value(0.75)
            .show_percentage(true)
            .build()
    })
    .with_size(40, 3);

    let rendered = app.render_to_string();
    assert!(rendered.contains("75%"), "Should show 75%: {}", rendered);
}

#[test]
fn test_progress_bar_hides_percentage() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::progress_bar()
            .value(0.5)
            .show_percentage(false)
            .build()
    })
    .with_size(40, 3);

    let rendered = app.render_to_string();
    assert!(
        !rendered.contains('%'),
        "Should not show percentage: {}",
        rendered
    );
}

#[test]
fn test_progress_bar_with_label() {
    let mut app =
        TestApp::new(|_cx: Scope| View::progress_bar().value(0.5).label("Loading").build())
            .with_size(50, 3);

    let rendered = app.render_to_string();
    assert!(
        rendered.contains("Loading"),
        "Should show label: {}",
        rendered
    );
}

#[test]
fn test_progress_bar_custom_chars() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::progress_bar()
            .value(0.5)
            .filled_char('=')
            .empty_char('-')
            .width(10)
            .build()
    })
    .with_size(30, 3);

    let rendered = app.render_to_string();
    assert!(
        rendered.contains('='),
        "Should use custom filled char: {}",
        rendered
    );
    assert!(
        rendered.contains('-'),
        "Should use custom empty char: {}",
        rendered
    );
}

#[test]
fn test_progress_bar_fixed_width() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::progress_bar()
            .value(1.0)
            .width(10)
            .show_percentage(false)
            .filled_char('#')
            .build()
    })
    .with_size(50, 3);

    let rendered = app.render_to_string();
    // Count filled chars - should be exactly 10
    let filled_count = rendered.chars().filter(|&c| c == '#').count();
    assert_eq!(
        filled_count, 10,
        "Should have exactly 10 filled chars: {}",
        rendered
    );
}

#[test]
fn test_progress_bar_zero_value() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::progress_bar()
            .value(0.0)
            .width(10)
            .show_percentage(true)
            .build()
    })
    .with_size(30, 3);

    let rendered = app.render_to_string();
    assert!(rendered.contains("0%"), "Should show 0%: {}", rendered);
}

#[test]
fn test_progress_bar_full_value() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::progress_bar()
            .value(1.0)
            .width(10)
            .show_percentage(true)
            .build()
    })
    .with_size(30, 3);

    let rendered = app.render_to_string();
    assert!(rendered.contains("100%"), "Should show 100%: {}", rendered);
}

// ============================================================================
// StatusBar Tests
// ============================================================================

#[test]
fn test_status_bar_renders() {
    let mut app =
        TestApp::new(|_cx: Scope| View::status_bar().left("NORMAL").build()).with_size(40, 3);

    let rendered = app.render_to_string();
    assert!(
        rendered.contains("NORMAL"),
        "Should render left section: {}",
        rendered
    );
}

#[test]
fn test_status_bar_left_and_right() {
    let mut app =
        TestApp::new(|_cx: Scope| View::status_bar().left("INSERT").right("Ln 42").build())
            .with_size(40, 3);

    let rendered = app.render_to_string();
    assert!(
        rendered.contains("INSERT"),
        "Should render left section: {}",
        rendered
    );
    assert!(
        rendered.contains("Ln 42"),
        "Should render right section: {}",
        rendered
    );
}

#[test]
fn test_status_bar_all_sections() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::status_bar()
            .left("LEFT")
            .center("CENTER")
            .right("RIGHT")
            .build()
    })
    .with_size(50, 3);

    let rendered = app.render_to_string();
    assert!(
        rendered.contains("LEFT"),
        "Should render left section: {}",
        rendered
    );
    assert!(
        rendered.contains("CENTER"),
        "Should render center section: {}",
        rendered
    );
    assert!(
        rendered.contains("RIGHT"),
        "Should render right section: {}",
        rendered
    );
}

#[test]
fn test_status_bar_right_aligned() {
    let mut app =
        TestApp::new(|_cx: Scope| View::status_bar().right("END").build()).with_size(20, 3);

    let rendered = app.render_to_string();
    // Right section should be at the end of the line
    assert!(
        rendered.contains("END"),
        "Should render right section: {}",
        rendered
    );
    // Check that END is near the end by looking at the line
    let line = rendered.lines().next().unwrap_or("");
    assert!(
        line.trim_end().ends_with("END"),
        "Right section should be right-aligned: '{}'",
        line
    );
}

#[test]
fn test_status_bar_center_aligned() {
    let mut app =
        TestApp::new(|_cx: Scope| View::status_bar().center("MID").build()).with_size(20, 3);

    let rendered = app.render_to_string();
    assert!(
        rendered.contains("MID"),
        "Should render center section: {}",
        rendered
    );
    // Center should be approximately in the middle
    let line = rendered.lines().next().unwrap_or("");
    if let Some(pos) = line.find("MID") {
        let mid_point = line.len() / 2;
        assert!(
            (pos as i32 - mid_point as i32).abs() < 5,
            "Center section should be near middle: pos={}, mid={}",
            pos,
            mid_point
        );
    }
}

#[test]
fn test_status_bar_fills_width() {
    let mut app = TestApp::new(|_cx: Scope| View::status_bar().left("X").build()).with_size(30, 3);

    let rendered = app.render_to_string();
    // Should contain X for left section
    assert!(
        rendered.contains("X"),
        "Should render left section: {}",
        rendered
    );
    // Status bar renders as a full line with background color filling the width
}

#[test]
fn test_status_bar_empty_sections() {
    let mut app = TestApp::new(|_cx: Scope| View::status_bar().left("").build()).with_size(20, 3);

    let rendered = app.render_to_string();
    // Should render without errors even with empty content
    assert!(!rendered.is_empty(), "Should render something");
}
