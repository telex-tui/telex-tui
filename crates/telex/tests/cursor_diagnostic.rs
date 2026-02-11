//! Diagnostic test for cursor rendering

use telex::prelude::*;
use telex::testing::TestApp;

#[test]
fn test_text_input_cursor_visible() {
    let mut app = TestApp::new(|cx: Scope| {
        let text = cx.use_state(|| "hello".to_string());
        View::text_input()
            .value(text.get())
            .build()
    }).with_size(40, 3);

    // Focus the text input
    app.focus_next();
    
    let rendered = app.render_to_string();
    println!("=== Rendered output ===");
    println!("{}", rendered);
    println!("=== End ===");
    
    // Check for cursor character (▌ when at end/space position)
    assert!(rendered.contains('█'), "Block cursor '█' should be in output.\nRendered:\n{}", rendered);
}

#[test]
fn test_text_input_cursor_not_visible_when_unfocused() {
    let mut app = TestApp::new(|cx: Scope| {
        let text = cx.use_state(|| "hello".to_string());
        View::vstack()
            .child(View::button().label("btn").build())  // index 0
            .child(View::text_input().value(text.get()).build())  // index 1
            .build()
    }).with_size(40, 5);

    // Initial focus is at index 0 (button)
    // Do NOT call focus_next() - we want button focused, not text_input

    let rendered = app.render_to_string();
    println!("=== Unfocused text input (focus on button at idx 0) ===");
    println!("{}", rendered);

    // Cursor should NOT appear when text_input is unfocused
    assert!(!rendered.contains('▋'), "Block cursor '▋' should NOT be in unfocused output.\nRendered:\n{}", rendered);
}
