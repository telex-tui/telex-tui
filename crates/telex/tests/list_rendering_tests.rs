//! Tests for list rendering and height calculations.
//!
//! These tests help verify that numbered and bulleted lists
//! render correctly with proper text wrapping and scrolling.

use telex::markdown;
use telex::prelude::*;
use telex::testing::TestApp;

/// Generate a numbered list markdown string.
fn make_numbered_list(items: &[&str]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| format!("{}. {}", i + 1, item))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate a bulleted list markdown string.
fn make_bulleted_list(items: &[&str]) -> String {
    items
        .iter()
        .map(|item| format!("- {}", item))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_short_numbered_list_renders_all_items() {
    let md = make_numbered_list(&["Apple", "Banana", "Cherry"]);
    let view = markdown::render(&md);

    let mut app = TestApp::new(|_cx: Scope| view.clone()).with_size(40, 20);

    // All items should be present - using new assertion methods
    app.assert_visible("1.");
    app.assert_visible("Apple");
    app.assert_visible("2.");
    app.assert_visible("Banana");
    app.assert_visible("3.");
    app.assert_visible("Cherry");
}

#[test]
fn test_numbered_list_finds_all_text() {
    let md = make_numbered_list(&["Apple", "Banana", "Cherry", "Date", "Elderberry"]);
    let view = markdown::render(&md);

    let app = TestApp::new(|_cx: Scope| view.clone());
    let texts = app.find_all_text();

    // Should find all item texts
    assert!(
        texts.iter().any(|t| t.contains("Apple")),
        "Should find Apple"
    );
    assert!(
        texts.iter().any(|t| t.contains("Banana")),
        "Should find Banana"
    );
    assert!(
        texts.iter().any(|t| t.contains("Cherry")),
        "Should find Cherry"
    );
    assert!(texts.iter().any(|t| t.contains("Date")), "Should find Date");
    assert!(
        texts.iter().any(|t| t.contains("Elderberry")),
        "Should find Elderberry"
    );
}

#[test]
fn test_long_numbered_list_has_correct_item_count() {
    // Similar to the English counties list (47 items)
    let counties: Vec<&str> = vec![
        "Bedfordshire",
        "Berkshire",
        "Buckinghamshire",
        "Cambridgeshire",
        "Cheshire",
        "Cornwall",
        "Cumberland",
        "Derbyshire",
        "Devon",
        "Dorset",
        "Durham",
        "Essex",
        "Gloucestershire",
        "Hampshire",
        "Herefordshire",
        "Hertfordshire",
        "Huntingdonshire",
        "Kent",
        "Lancashire",
        "Leicestershire",
        "Lincolnshire",
        "Middlesex",
        "Norfolk",
        "Northamptonshire",
        "Northumberland",
        "Nottinghamshire",
        "Oxfordshire",
        "Rutland",
        "Shropshire",
        "Somerset",
        "Staffordshire",
        "Suffolk",
        "Surrey",
        "Sussex",
        "Warwickshire",
        "Westmorland",
        "Wiltshire",
        "Worcestershire",
        "Yorkshire",
        "London",
        "Isle of Wight",
        "Monmouthshire",
        "Hereford and Worcester",
        "Cleveland",
        "Avon",
        "Humberside",
        "Tyne and Wear",
    ];

    let md = make_numbered_list(&counties);
    let view = markdown::render(&md);

    let app = TestApp::new(|_cx: Scope| view.clone());
    let texts = app.find_all_text();

    // Verify first and last items are in the view tree
    assert!(
        texts.iter().any(|t| t.contains("Bedfordshire")),
        "Should find first item"
    );
    assert!(
        texts.iter().any(|t| t.contains("Tyne and Wear")),
        "Should find last item"
    );

    // Count how many county names we find
    let found_count = counties
        .iter()
        .filter(|county| texts.iter().any(|t| t.contains(*county)))
        .count();

    assert_eq!(
        found_count,
        counties.len(),
        "Should find all {} counties",
        counties.len()
    );
}

#[test]
fn test_list_item_text_wrapping() {
    // Create a list with long items that need to wrap
    let md = make_numbered_list(&[
        "This is a very long list item that should wrap to multiple lines when rendered in a narrow terminal window",
        "Another long item here",
        "Short",
    ]);
    let view = markdown::render(&md);

    let mut app = TestApp::new(|_cx: Scope| view.clone()).with_size(30, 20);
    let rendered = app.render_to_string();

    // The text should be present (wrapped across multiple lines)
    assert!(
        rendered.contains("very long"),
        "Should contain wrapped text"
    );
    assert!(rendered.contains("Short"), "Should contain short item");
}

#[test]
fn test_bulleted_list_renders_correctly() {
    let md = make_bulleted_list(&["First", "Second", "Third"]);
    let view = markdown::render(&md);

    let mut app = TestApp::new(|_cx: Scope| view.clone()).with_size(40, 20);
    let rendered = app.render_to_string();

    // Should have bullet markers
    assert!(
        rendered.contains("•") || rendered.contains("-"),
        "Should contain bullet marker"
    );
    assert!(rendered.contains("First"), "Should contain First");
    assert!(rendered.contains("Second"), "Should contain Second");
    assert!(rendered.contains("Third"), "Should contain Third");
}

#[test]
fn test_list_in_scrollable_box() {
    let counties: Vec<&str> = vec![
        "Bedfordshire",
        "Berkshire",
        "Buckinghamshire",
        "Cambridgeshire",
        "Cheshire",
        "Cornwall",
        "Cumberland",
        "Derbyshire",
        "Devon",
        "Dorset",
        "Durham",
        "Essex",
        "Gloucestershire",
        "Hampshire",
        "Herefordshire",
        "Hertfordshire",
        "Huntingdonshire",
        "Kent",
        "Lancashire",
        "Leicestershire",
    ];

    let md = make_numbered_list(&counties);

    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .auto_scroll_bottom(true)
            .child(markdown::render(&md))
            .build()
    })
    .with_size(60, 10);

    // Check focusable count - should have 1 focusable (the scrollable box)
    let focusable = app.focusable_count();
    assert_eq!(focusable, 1, "Should have 1 focusable (scrollable box)");

    // With auto_scroll_bottom, LAST item should be visible, FIRST should not
    app.assert_visible("Leicestershire"); // Last item
    app.assert_not_visible("Bedfordshire"); // First item should be scrolled away
}

#[test]
fn test_manual_scroll_box_with_list() {
    let items: Vec<&str> = (0..20)
        .map(|i| match i {
            0 => "Item Zero",
            1 => "Item One",
            2 => "Item Two",
            3 => "Item Three",
            4 => "Item Four",
            5 => "Item Five",
            6 => "Item Six",
            7 => "Item Seven",
            8 => "Item Eight",
            9 => "Item Nine",
            10 => "Item Ten",
            11 => "Item Eleven",
            12 => "Item Twelve",
            13 => "Item Thirteen",
            14 => "Item Fourteen",
            15 => "Item Fifteen",
            16 => "Item Sixteen",
            17 => "Item Seventeen",
            18 => "Item Eighteen",
            _ => "Item Nineteen",
        })
        .collect();

    let md = make_numbered_list(&items);

    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .scroll(true)
            .child(markdown::render(&md))
            .build()
    })
    .with_size(40, 8);

    // Initial render - manual scroll starts at top
    app.assert_visible("Item Zero"); // First item visible
    app.assert_not_visible("Item Nineteen"); // Last item not visible

    // Check which items are visible initially
    let visible = app.visible_items(&items);
    assert!(
        visible.contains(&"Item Zero".to_string()),
        "First item should be in visible list"
    );
}

#[test]
fn test_empty_list() {
    let md = "";
    let view = markdown::render(md);

    let app = TestApp::new(|_cx: Scope| view.clone());
    let texts = app.find_all_text();

    // Empty markdown should produce no text
    assert!(
        texts.is_empty() || texts.iter().all(|t| t.is_empty()),
        "Empty markdown should produce empty view"
    );
}

#[test]
fn test_mixed_content_with_list() {
    let md = r#"# Counties of England

Here is a list of some English counties:

1. Bedfordshire
2. Berkshire
3. Buckinghamshire

These are the traditional counties."#;

    let view = markdown::render(md);
    let app = TestApp::new(|_cx: Scope| view.clone());
    let texts = app.find_all_text();

    // Should find header
    assert!(
        texts.iter().any(|t| t.contains("Counties of England")),
        "Should find header"
    );

    // Should find list items
    assert!(
        texts.iter().any(|t| t.contains("Bedfordshire")),
        "Should find Bedfordshire"
    );
    assert!(
        texts.iter().any(|t| t.contains("Buckinghamshire")),
        "Should find Buckinghamshire"
    );

    // Should find paragraph text
    assert!(
        texts.iter().any(|t| t.contains("traditional")),
        "Should find paragraph text"
    );
}

/// Diagnostic test that prints detailed rendering info.
/// Run with: cargo test -p telex --test list_rendering_tests diagnostic -- --nocapture
#[test]
fn diagnostic_list_rendering() {
    let counties: Vec<&str> = vec![
        "Bedfordshire",
        "Berkshire",
        "Buckinghamshire",
        "Cambridgeshire",
        "Cheshire",
        "Cornwall",
        "Cumberland",
        "Derbyshire",
        "Devon",
        "Dorset",
    ];

    let md = make_numbered_list(&counties);
    println!("Input markdown:\n{}\n", md);

    let view = markdown::render(&md);
    println!("View tree structure: {:?}\n", view_debug(&view));

    let mut app = TestApp::new(|_cx: Scope| view.clone()).with_size(50, 15);
    let rendered = app.render_to_string();

    println!("Rendered output (50x15):");
    println!("{}", rendered);
    println!("\nFocusable count: {}", app.focusable_count());

    // Count visible items
    let visible_count = counties
        .iter()
        .filter(|county| rendered.contains(*county))
        .count();
    println!("Visible items: {}/{}", visible_count, counties.len());
}

/// Diagnostic test for auto-scroll box with list (mimics telex-ai chat).
/// Run with: cargo test -p telex-tui --test list_rendering_tests diagnostic_auto_scroll -- --nocapture
#[test]
fn diagnostic_auto_scroll() {
    let counties: Vec<&str> = vec![
        "Bedfordshire",
        "Berkshire",
        "Buckinghamshire",
        "Cambridgeshire",
        "Cheshire",
        "Cornwall",
        "Cumberland",
        "Derbyshire",
        "Devon",
        "Dorset",
        "Durham",
        "Essex",
        "Gloucestershire",
        "Hampshire",
        "Herefordshire",
        "Hertfordshire",
        "Huntingdonshire",
        "Kent",
        "Lancashire",
        "Leicestershire",
        "Lincolnshire",
        "Middlesex",
        "Norfolk",
        "Northamptonshire",
        "Northumberland",
        "Nottinghamshire",
        "Oxfordshire",
        "Rutland",
        "Shropshire",
        "Somerset",
        "Staffordshire",
        "Suffolk",
        "Surrey",
        "Sussex",
        "Warwickshire",
        "Westmorland",
        "Wiltshire",
        "Worcestershire",
        "Yorkshire",
        "London",
        "Isle of Wight",
        "Monmouthshire",
        "Hereford and Worcester",
        "Cleveland",
        "Avon",
        "Humberside",
        "Tyne and Wear",
    ];

    let md = make_numbered_list(&counties);
    let view = markdown::render(&md);

    println!("=== Test: 47 counties in auto_scroll_bottom box ===");
    println!("Total items: {}", counties.len());
    println!("View structure: {:?}", view_debug(&view));

    // Simulate a small visible area (like a chat window)
    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .auto_scroll_bottom(true)
            .border(true)
            .child(view.clone())
            .build()
    })
    .with_size(60, 12); // Small viewport

    let rendered = app.render_to_string();
    println!("\nRendered (60x12 viewport with auto_scroll_bottom):");
    println!("{}", rendered);
    println!("\nFocusable count: {}", app.focusable_count());

    // Count which items are visible
    let visible: Vec<_> = counties
        .iter()
        .enumerate()
        .filter(|(_, county)| rendered.contains(*county))
        .collect();
    println!("\nVisible items: {}/{}", visible.len(), counties.len());
    if !visible.is_empty() {
        println!(
            "First visible: {}. {}",
            visible.first().unwrap().0 + 1,
            visible.first().unwrap().1
        );
        println!(
            "Last visible: {}. {}",
            visible.last().unwrap().0 + 1,
            visible.last().unwrap().1
        );
    }

    // With auto_scroll_bottom, we expect to see the LAST items, not the first
    // Let's verify this behavior
    let expected_last = "Tyne and Wear";
    if rendered.contains(expected_last) {
        println!(
            "\nAuto-scroll working: last item '{}' is visible",
            expected_last
        );
    } else {
        println!(
            "\nAuto-scroll issue: last item '{}' is NOT visible",
            expected_last
        );
        println!("This suggests the scroll calculation or rendering isn't showing the bottom");
    }
}

/// Test that compares auto-scroll vs manual scroll behavior.
/// Run with: cargo test -p telex-tui --test list_rendering_tests diagnostic_scroll_comparison -- --nocapture
#[test]
fn diagnostic_scroll_comparison() {
    let items: Vec<&str> = (1..=30)
        .map(|i| match i {
            1 => "Item_001",
            2 => "Item_002",
            3 => "Item_003",
            4 => "Item_004",
            5 => "Item_005",
            6 => "Item_006",
            7 => "Item_007",
            8 => "Item_008",
            9 => "Item_009",
            10 => "Item_010",
            11 => "Item_011",
            12 => "Item_012",
            13 => "Item_013",
            14 => "Item_014",
            15 => "Item_015",
            16 => "Item_016",
            17 => "Item_017",
            18 => "Item_018",
            19 => "Item_019",
            20 => "Item_020",
            21 => "Item_021",
            22 => "Item_022",
            23 => "Item_023",
            24 => "Item_024",
            25 => "Item_025",
            26 => "Item_026",
            27 => "Item_027",
            28 => "Item_028",
            29 => "Item_029",
            _ => "Item_030",
        })
        .collect();

    let md = make_numbered_list(&items);
    let view = markdown::render(&md);

    println!("=== Scroll Comparison Test ===");
    println!("30 items, viewport height 8 (excluding border)");

    // Test auto_scroll_bottom
    println!("\n--- auto_scroll_bottom box ---");
    let mut auto_app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .auto_scroll_bottom(true)
            .border(true)
            .child(view.clone())
            .build()
    })
    .with_size(40, 10);

    let auto_rendered = auto_app.render_to_string();
    println!("{}", auto_rendered);

    // Test manual scroll box (starts at top)
    println!("\n--- manual scroll box (at top) ---");
    let mut manual_app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .scroll(true)
            .border(true)
            .child(view.clone())
            .build()
    })
    .with_size(40, 10);

    let manual_rendered = manual_app.render_to_string();
    println!("{}", manual_rendered);

    // Compare what's visible
    println!("\n--- Comparison ---");
    let auto_visible: Vec<_> = items
        .iter()
        .filter(|item| auto_rendered.contains(*item))
        .collect();
    let manual_visible: Vec<_> = items
        .iter()
        .filter(|item| manual_rendered.contains(*item))
        .collect();

    println!("auto_scroll_bottom visible items: {:?}", auto_visible);
    println!("manual scroll visible items: {:?}", manual_visible);
}

/// Helper to create a simple debug representation of the view tree.
fn view_debug(view: &View) -> String {
    match view {
        View::Text(n) => format!("Text({:?})", truncate(&n.content, 20)),
        View::VStack(n) => format!("VStack[{}]", n.children.len()),
        View::HStack(n) => format!("HStack[{}]", n.children.len()),
        View::Box(n) => {
            let attrs = [
                n.scroll.then_some("scroll"),
                n.auto_scroll_bottom.then_some("auto_scroll"),
                n.border.then_some("border"),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(",");
            format!("Box({})", attrs)
        }
        View::Empty => "Empty".to_string(),
        _ => "Other".to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

// ============================================================
// Tests informed by examples
// ============================================================

/// Test the log viewer pattern: scroll + auto_scroll_bottom combined.
/// From example 06_log_viewer.
#[test]
fn test_log_viewer_pattern() {
    let log_lines: Vec<String> = (1..=30)
        .map(|i| format!("[INFO] Log entry {}", i))
        .collect();
    let log_text = log_lines.join("\n");

    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .scroll(true)
            .auto_scroll_bottom(true)
            .min_height(10)
            .max_height(10)
            .child(View::text(&log_text))
            .build()
    })
    .with_size(50, 12);

    // Should show the END of the log (auto_scroll_bottom)
    app.assert_visible("Log entry 30");
    app.assert_visible("Log entry 29");
    // Use "Log entry 5" to avoid substring matching (e.g., "Log entry 1" matches "Log entry 19")
    app.assert_not_visible("Log entry 5");
}

/// Test min_height constraint.
#[test]
fn test_min_height_constraint() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(
                View::boxed()
                    .border(true)
                    .min_height(5)
                    .child(View::text("Short"))
                    .build(),
            )
            .child(View::text("Below"))
            .build()
    })
    .with_size(30, 20);

    let rendered = app.render_to_string();
    println!("min_height test:\n{}", rendered);

    // Both should be visible
    app.assert_visible("Short");
    app.assert_visible("Below");

    // "Below" should be at least 5 lines down (min_height of box)
    let short_line = app.find_line_containing("Short");
    let below_line = app.find_line_containing("Below");
    assert!(
        below_line.unwrap() >= short_line.unwrap() + 4,
        "Below should be at least 4 lines after Short due to min_height"
    );
}

/// Test max_height constraint clips content.
#[test]
fn test_max_height_constraint() {
    let long_text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8";

    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(
                View::boxed()
                    .border(true)
                    .max_height(5) // Only room for ~3 lines of content
                    .child(View::text(long_text))
                    .build(),
            )
            .child(View::text("After box"))
            .build()
    })
    .with_size(30, 20);

    // "After box" should be visible (not pushed off screen by unbounded box)
    app.assert_visible("After box");
}

/// Test modal visibility.
#[test]
fn test_modal_rendering() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::text("Background content"))
            .child(
                View::modal()
                    .visible(true)
                    .title("Test Modal")
                    .width(50)
                    .height(30)
                    .child(View::text("Modal body text"))
                    .build(),
            )
            .build()
    })
    .with_size(60, 20);

    // Modal content should be visible
    app.assert_visible("Test Modal");
    app.assert_visible("Modal body text");
}

/// Test modal hidden.
#[test]
fn test_modal_hidden() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::text("Background content"))
            .child(
                View::modal()
                    .visible(false)
                    .title("Hidden Modal")
                    .child(View::text("Should not see this"))
                    .build(),
            )
            .build()
    })
    .with_size(60, 20);

    // Background should be visible, modal should not
    app.assert_visible("Background content");
    app.assert_not_visible("Hidden Modal");
    app.assert_not_visible("Should not see this");
}

/// Test text input placeholder rendering.
#[test]
fn test_text_input_placeholder() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::text_input()
            .value(String::new())
            .placeholder("Type something here...")
            .build()
    })
    .with_size(40, 3);

    // Placeholder should be visible when value is empty
    app.assert_visible("Type something");
}

/// Test styled text rendering (bold, dim, color).
#[test]
fn test_styled_text_rendering() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::styled_text("Bold text").bold().build())
            .child(View::styled_text("Dim text").dim().build())
            .child(View::styled_text("Normal text").build())
            .build()
    })
    .with_size(30, 10);

    // All text variants should render
    app.assert_visible("Bold text");
    app.assert_visible("Dim text");
    app.assert_visible("Normal text");
}

/// Test HStack with mixed fixed and flex children (system monitor pattern).
#[test]
fn test_hstack_mixed_flex() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::hstack()
            .child(View::text("Label: ")) // Fixed width
            .child(
                View::boxed()
                    .flex(1)
                    .child(View::text("[##########----------]")) // Flex fills remaining
                    .build(),
            )
            .child(View::text(" 50%")) // Fixed width
            .build()
    })
    .with_size(50, 3);

    // All parts should be visible in the HStack
    app.assert_visible("Label:");
    app.assert_visible("###"); // Part of the progress bar
    app.assert_visible("50%");
}

/// Test that auto_scroll_bottom shows the END of content, not the beginning.
/// This is the core auto-scroll bug.
#[test]
fn test_auto_scroll_shows_end_not_beginning() {
    // Create content that's definitely taller than viewport
    let mut lines = Vec::new();
    for i in 1..=50 {
        lines.push(format!("Line {}: This is line number {}", i, i));
    }
    let content = lines.join("\n");

    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .auto_scroll_bottom(true)
            .border(true)
            .child(View::text(&content))
            .build()
    })
    .with_size(50, 10);

    // With auto_scroll_bottom, we should see the END (lines 45-50ish)
    // NOT the beginning (lines 1-10)
    app.assert_visible("Line 50"); // Last line must be visible
    app.assert_not_visible("Line 1:"); // First line must NOT be visible

    println!("Rendered:\n{}", app.render_to_string());
}

/// Test auto-scroll with markdown content (like telex-ai uses).
/// This mimics the real app structure more closely.
#[test]
fn test_auto_scroll_with_markdown_content() {
    // Create markdown similar to what Gemini returns
    let mut items = Vec::new();
    for i in 1..=40 {
        items.push(format!("**Country {}** - Capital {}", i, i));
    }
    let md_content = items.join("\n");

    let mut app = TestApp::new(|_cx: Scope| {
        // Mimic telex-ai structure: scrollable box containing a VStack with messages
        View::boxed()
            .auto_scroll_bottom(true)
            .border(true)
            .child(
                View::vstack()
                    .child(View::styled_text("Assistant").bold().build())
                    .child(markdown::render(&md_content))
                    .build(),
            )
            .build()
    })
    .with_size(60, 12);

    let rendered = app.render_to_string();
    println!("Rendered:\n{}", rendered);

    // Should see the END of the list, not the beginning
    app.assert_visible("Country 40"); // Last item
    app.assert_not_visible("Country 1 "); // First item (with space to avoid matching Country 10, 11, etc)
}

/// Test based on actual Gemini response - European capitals.
/// This is the exact bug report scenario.
#[test]
fn test_european_capitals_auto_scroll() {
    // This is the actual content structure from Gemini (simplified)
    let content = r#"Of course! Here is a list of European capitals, organized by country:

**Albania** - Tirana
**Andorra** - Andorra la Vella
**Armenia** - Yerevan
**Austria** - Vienna
**Azerbaijan** - Baku
**Belarus** - Minsk
**Belgium** - Brussels
**Bosnia and Herzegovina** - Sarajevo
**Bulgaria** - Sofia
**Croatia** - Zagreb
**Cyprus** - Nicosia
**Czech Republic** - Prague
**Denmark** - Copenhagen
**Estonia** - Tallinn
**Finland** - Helsinki
**France** - Paris
**Georgia** - Tbilisi
**Germany** - Berlin
**Greece** - Athens
**Hungary** - Budapest
**Iceland** - Reykjavik
**Ireland** - Dublin
**Italy** - Rome
**Kazakhstan** - Nur-Sultan
**Kosovo** - Pristina
**Latvia** - Riga
**Liechtenstein** - Vaduz
**Lithuania** - Vilnius
**Luxembourg** - Luxembourg City
**Malta** - Valletta
**Moldova** - Chisinau
**Monaco** - Monaco
**Montenegro** - Podgorica
**Netherlands** - Amsterdam
**North Macedonia** - Skopje
**Norway** - Oslo
**Poland** - Warsaw
**Portugal** - Lisbon
**Romania** - Bucharest
**Russia** - Moscow
**San Marino** - San Marino
**Serbia** - Belgrade
**Slovakia** - Bratislava
**Slovenia** - Ljubljana
**Spain** - Madrid
**Sweden** - Stockholm
**Switzerland** - Bern
**Turkey** - Ankara
**Ukraine** - Kyiv
**United Kingdom** - London
**Vatican City** - Vatican City

I hope this list is helpful!"#;

    // Mimic telex-ai structure exactly
    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .auto_scroll_bottom(true)
            .border(true)
            .flex(1)
            .padding(1)
            .child(
                View::vstack()
                    .child(
                        View::vstack()
                            .child(View::styled_text("Assistant").bold().build())
                            .child(markdown::render(content))
                            .child(View::text(""))
                            .build(),
                    )
                    .build(),
            )
            .build()
    })
    .with_size(80, 15); // Typical terminal size

    let rendered = app.render_to_string();
    println!("European capitals test:\n{}", rendered);

    // The END of the response should be visible, not the beginning
    app.assert_visible("Vatican City"); // Last country
    app.assert_visible("hope this list is helpful"); // Final line
    app.assert_not_visible("Albania"); // First country should be scrolled away
    app.assert_not_visible("Of course!"); // Opening should be scrolled away
}

/// Test auto-scroll with multiple messages (closer to real chat).
#[test]
fn test_auto_scroll_multiple_messages() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::boxed()
            .auto_scroll_bottom(true)
            .border(true)
            .child(
                View::vstack()
                    // Message 1
                    .child(View::vstack()
                        .child(View::styled_text("You").bold().build())
                        .child(View::text("First question"))
                        .build())
                    // Message 2 - long response
                    .child(View::vstack()
                        .child(View::styled_text("Assistant").bold().build())
                        .child(View::text("Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15\nLine 16\nLine 17\nLine 18\nLine 19\nLine 20"))
                        .build())
                    .build()
            )
            .build()
    }).with_size(40, 10);

    let rendered = app.render_to_string();
    println!("Rendered:\n{}", rendered);

    // Should see the end of the response, not "First question"
    app.assert_visible("Line 20");
    app.assert_not_visible("First question");
}

/// Test that single newlines in markdown are preserved as line breaks.
/// This is the "European capitals" bug - LLM sends items separated by \n
/// but they render as one long paragraph.
#[test]
fn test_soft_breaks_preserved() {
    // This is what Gemini sends - items separated by single newlines
    let md = "**Albania** - Tirana\n**Andorra** - Andorra la Vella\n**Austria** - Vienna";
    let view = markdown::render(md);

    let mut app = TestApp::new(|_cx: Scope| view.clone()).with_size(60, 10);
    let lines = app.rendered_lines();

    println!("Rendered lines:");
    for (i, line) in lines.iter().enumerate() {
        println!("  {}: {:?}", i, line);
    }

    // Each country should be on its own line, NOT all on one line
    // If this fails, soft breaks are being collapsed to spaces
    let rendered = app.render_to_string();

    // These should NOT all be on the same line
    let albania_line = app.find_line_containing("Albania");
    let andorra_line = app.find_line_containing("Andorra");
    let austria_line = app.find_line_containing("Austria");

    println!(
        "\nLine positions: Albania={:?}, Andorra={:?}, Austria={:?}",
        albania_line, andorra_line, austria_line
    );

    // They should be on different lines
    assert_ne!(
        albania_line, andorra_line,
        "Albania and Andorra should be on different lines!\nRendered:\n{}",
        rendered
    );
    assert_ne!(
        andorra_line, austria_line,
        "Andorra and Austria should be on different lines!\nRendered:\n{}",
        rendered
    );
}

/// Test that HStack items with wrapped text have correct heights.
/// This was a previous bug where list items showed only one line.
#[test]
fn test_hstack_wrapped_height() {
    let long_text = "This is a very long text that should wrap to multiple lines when rendered";

    // Create an HStack with a marker and long text (like a list item)
    let view = View::hstack()
        .child(View::text("1. "))
        .child(View::boxed().flex(1).child(View::text(long_text)).build())
        .build();

    let mut app = TestApp::new(|_cx: Scope| view.clone()).with_size(30, 10);
    let rendered = app.render_to_string();

    println!("HStack with wrapped text (30 wide):\n{}", rendered);

    // The text should be wrapped and visible
    assert!(
        rendered.contains("This is a"),
        "Should contain start of text"
    );
    assert!(
        rendered.contains("wrap") || rendered.contains("wra"),
        "Should contain 'wrap' or wrapped version"
    );
}

/// Verify the view tree structure for a numbered list.
#[test]
fn test_list_view_structure() {
    let md = "1. First\n2. Second\n3. Third";
    let view = markdown::render(md);

    // Should be a VStack
    match &view {
        View::VStack(vstack) => {
            println!("VStack with {} children", vstack.children.len());
            for (i, child) in vstack.children.iter().enumerate() {
                println!("  Child {}: {}", i, view_debug(child));
            }
            // Each list item should be an HStack
            // Plus potential spacing/empty views
            assert!(
                vstack.children.len() >= 3,
                "Should have at least 3 children for 3 items"
            );
        }
        _ => panic!("Expected VStack at root, got {:?}", view_debug(&view)),
    }
}

/// Test rendering with very narrow width.
#[test]
fn test_narrow_width_rendering() {
    let md = make_numbered_list(&["Apple", "Banana", "Cherry"]);
    let view = markdown::render(&md);

    let mut app = TestApp::new(|_cx: Scope| view.clone()).with_size(15, 20);
    let rendered = app.render_to_string();

    println!("Narrow width render (15 wide):\n{}", rendered);

    // Items should still be present even in narrow width
    assert!(
        rendered.contains("Appl") || rendered.contains("Apple"),
        "Should show Apple or truncated"
    );
}
