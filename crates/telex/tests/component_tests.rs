//! Example tests demonstrating Telex's testing utilities.

use telex::prelude::*;
use telex::testing::TestApp;

#[test]
fn test_counter_increments() {
    let mut app = TestApp::new(|cx: Scope| {
        let count = cx.use_state(|| 0);
        let c1 = count.clone();
        let c2 = count.clone();

        View::vstack()
            .child(View::text(format!("Count: {}", count.get())))
            .child(
                View::hstack()
                    .child(
                        View::button()
                            .label("-")
                            .on_press(move || c1.update(|n| *n -= 1))
                            .build(),
                    )
                    .child(
                        View::button()
                            .label("+")
                            .on_press(move || c2.update(|n| *n += 1))
                            .build(),
                    )
                    .build(),
            )
            .build()
    });

    // Initial state
    assert!(app.has_text("Count: 0"));

    // Press + button
    assert!(app.press_button("+"));
    assert!(app.has_text("Count: 1"));

    // Press + again
    assert!(app.press_button("+"));
    assert!(app.has_text("Count: 2"));

    // Press - button
    assert!(app.press_button("-"));
    assert!(app.has_text("Count: 1"));
}

#[test]
fn test_finds_all_text() {
    let app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::text("Hello"))
            .child(View::text("World"))
            .child(View::text("Telex"))
            .build()
    });

    let texts = app.find_all_text();
    assert_eq!(texts.len(), 3);
    assert!(texts.contains(&"Hello".to_string()));
    assert!(texts.contains(&"World".to_string()));
    assert!(texts.contains(&"Telex".to_string()));
}

#[test]
fn test_finds_buttons() {
    let app = TestApp::new(|_cx: Scope| {
        View::hstack()
            .child(View::button().label("Save").build())
            .child(View::button().label("Cancel").build())
            .child(View::button().label("Help").build())
            .build()
    });

    let buttons = app.find_all_buttons();
    assert_eq!(buttons.len(), 3);
    assert!(app.find_button("Save").is_some());
    assert!(app.find_button("Cancel").is_some());
    assert!(app.find_button("Help").is_some());
    assert!(app.find_button("Delete").is_none());
}

#[test]
fn test_focus_navigation() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::button().label("First").build())
            .child(View::button().label("Second").build())
            .child(View::button().label("Third").build())
            .build()
    });

    assert_eq!(app.focusable_count(), 3);
    assert_eq!(app.focus_index(), 0);

    app.focus_next();
    assert_eq!(app.focus_index(), 1);

    app.focus_next();
    assert_eq!(app.focus_index(), 2);

    app.focus_next(); // Wraps around
    assert_eq!(app.focus_index(), 0);

    app.focus_prev(); // Wraps to end
    assert_eq!(app.focus_index(), 2);
}

#[test]
fn test_checkbox_toggle() {
    let mut app = TestApp::new(|cx: Scope| {
        let checked = cx.use_state(|| false);
        let chk = checked.clone();

        View::vstack()
            .child(View::text(format!(
                "Status: {}",
                if checked.get() { "ON" } else { "OFF" }
            )))
            .child(
                View::checkbox()
                    .checked(checked.get())
                    .label("Toggle me")
                    .on_toggle(move |v| chk.set(v))
                    .build(),
            )
            .build()
    });

    // Initial state
    assert!(app.has_text("Status: OFF"));

    // Focus and activate checkbox
    app.focus_next(); // Move to checkbox
    app.activate();

    assert!(app.has_text("Status: ON"));

    // Toggle again
    app.activate();
    assert!(app.has_text("Status: OFF"));
}

#[test]
fn test_list_selection() {
    let mut app = TestApp::new(|cx: Scope| {
        let selected = cx.use_state(|| 0);
        let sel = selected.clone();

        View::vstack()
            .child(View::text(format!("Selected: {}", selected.get())))
            .child(
                View::list()
                    .items(vec![
                        "Apple".to_string(),
                        "Banana".to_string(),
                        "Cherry".to_string(),
                    ])
                    .selected(selected.get())
                    .on_select(move |i| sel.set(i))
                    .build(),
            )
            .build()
    });

    // Initial state
    assert!(app.has_text("Selected: 0"));

    // Focus list and navigate
    app.focus_next(); // Move to list
    app.list_down();
    assert!(app.has_text("Selected: 1"));

    app.list_down();
    assert!(app.has_text("Selected: 2"));

    app.list_up();
    assert!(app.has_text("Selected: 1"));
}

#[test]
fn test_text_input() {
    let mut app = TestApp::new(|cx: Scope| {
        let text = cx.use_state(String::new);
        let txt = text.clone();

        View::vstack()
            .child(View::text(format!("Input: {}", text.get())))
            .child(
                View::text_input()
                    .value(text.get())
                    .placeholder("Type here")
                    .on_change(move |s| txt.set(s))
                    .build(),
            )
            .build()
    });

    // Focus text input
    app.focus_next();

    // Type some text
    app.type_str("Hello");
    assert!(app.has_text("Input: Hello"));

    // Type more
    app.type_str(" World");
    assert!(app.has_text("Input: Hello World"));

    // Backspace
    app.backspace();
    app.backspace();
    assert!(app.has_text("Input: Hello Wor"));
}

#[test]
fn test_render_to_string() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::text("Header"))
            .child(View::text("Content"))
            .build()
    })
    .with_size(20, 5);

    let rendered = app.render_to_string();
    assert!(rendered.contains("Header"));
    assert!(rendered.contains("Content"));
}
