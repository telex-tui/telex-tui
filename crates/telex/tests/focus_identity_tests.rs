//! Tests for stable focus identity — see `docs/focus-identity-design.md`.
//!
//! Tests marked `// FAILED before 0.4` pin down behaviour the design changed; the rest
//! guard behaviour that must not change.
//!
//! Note: the first Tab only *reveals* focus (`FocusManager::focus_next`), it doesn't move
//! it — event sequences below account for that.

use telex::prelude::*;
use telex::testing::TestApp;
use telex::{Event, KeyCode, KeyEvent, KeyModifiers};

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn key_mod(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn char_key(c: char) -> Event {
    key(KeyCode::Char(c))
}

fn numbered_lines(prefix: &str, n: usize) -> String {
    (1..=n).map(|i| format!("{prefix} {:03}", i)).collect::<Vec<_>>().join("\n")
}

// ============================================================
// 1. Focus requests re-apply on screen change
// ============================================================

#[test]
fn test_button_focus_request_applies_on_screen_change() {
    // FAILED before 0.4: buttons can't request focus, and focus stays at index 0
    let output = run_headless(
        |cx: Scope| {
            let on_b = state!(cx, || false);
            let pressed = state!(cx, String::new);
            let press = |label: &'static str| with!(pressed => move || pressed.set(label.to_string()));

            let screen = if !on_b.get() {
                View::vstack()
                    .child(View::button().label("Go B").on_press(with!(on_b => move || on_b.set(true))).build())
                    .build()
            } else {
                View::vstack()
                    .child(View::button().label("B one").on_press(press("B one")).build())
                    .child(View::button().label("B two").on_press(press("B two")).build())
                    .child(
                        View::button()
                            .key("b-target")
                            .focused(true)
                            .label("B target")
                            .on_press(press("B target"))
                            .build(),
                    )
                    .build()
            };
            View::vstack()
                .child(screen)
                .child(View::text(format!("pressed: {}", pressed.get())))
                .build()
        },
        40,
        10,
        vec![key(KeyCode::Enter), key(KeyCode::Enter)],
    );
    assert!(output.contains("pressed: B target"), "got:\n{output}");
}

#[test]
fn test_text_input_focus_request_applies_on_second_screen() {
    // FAILED before 0.4: TextInput's request is honoured once per app
    let output = run_headless(
        |cx: Scope| {
            let on_b = state!(cx, || false);
            let b2 = state!(cx, String::new);

            let screen = if !on_b.get() {
                View::vstack()
                    .child(View::button().label("unused").build())
                    .child(
                        View::text_input()
                            .focused(true)
                            .on_submit(with!(on_b => move || on_b.set(true)))
                            .build(),
                    )
                    .build()
            } else {
                View::vstack()
                    .child(View::text_input().build())
                    .child(View::button().label("between").build())
                    .child(
                        View::text_input()
                            .key("b2")
                            .focused(true)
                            .value(b2.get())
                            .on_change(with!(b2 => move |v: String| b2.set(v)))
                            .build(),
                    )
                    .build()
            };
            View::vstack()
                .child(screen)
                .child(View::text(format!("b2=[{}]", b2.get())))
                .build()
        },
        40,
        10,
        vec![key(KeyCode::Enter), char_key('x')],
    );
    assert!(output.contains("b2=[x]"), "got:\n{output}");
}

// ============================================================
// 2. Scroll state doesn't leak between widgets
// ============================================================

/// Screen A: a scroll box (focused, index 0) + "Next". Screen B: a different scroll box
/// at the same index. Keys optional so the same view serves keyed and positional tests.
fn two_scroll_screens(keyed: bool) -> TestApp<impl Component> {
    TestApp::new(move |cx: Scope| {
        let on_b = state!(cx, || false);
        let (prefix, k) = if on_b.get() { ("B", "box-b") } else { ("A", "box-a") };
        let mut scroll_box = View::boxed().scroll(true);
        if keyed {
            scroll_box = scroll_box.key(k);
        }
        View::vstack()
            .child(scroll_box.max_height(6).child(View::text(numbered_lines(prefix, 50))).build())
            .child(View::button().label("Next").on_press(with!(on_b => move || on_b.set(true))).build())
            .build()
    })
    .with_size(30, 12)
}

#[test]
fn test_keyed_scroll_state_does_not_leak_to_next_screen() {
    // FAILED before 0.4: offset is stored per index and carries over
    let mut app = two_scroll_screens(true);
    app.scroll_down(20);
    app.assert_not_visible("A 001");
    assert!(app.press_button("Next"));
    app.assert_visible("B 001");
}

#[test]
fn test_unkeyed_scroll_state_is_positional() {
    // Documents today's positional behaviour, which the design keeps for unkeyed widgets
    let mut app = two_scroll_screens(false);
    app.scroll_down(20);
    assert!(app.press_button("Next"));
    app.assert_not_visible("B 001");
}

// ============================================================
// 3. Focus follows the key when the tree shifts
// ============================================================

#[test]
fn test_focus_follows_key_when_widget_inserted_above() {
    // FAILED before 0.4: focus stays at the index, which now holds "A"
    let output = run_headless(
        |cx: Scope| {
            let extra = state!(cx, || false);
            let target_count = state!(cx, || 0);
            let a_count = state!(cx, || 0);

            let mut list = View::vstack();
            if extra.get() {
                list = list.child(View::button().label("Extra").build());
            }
            list.child(View::button().label("A").on_press(with!(a_count => move || a_count.update(|n| *n += 1))).build())
                .child(
                    View::button()
                        .key("target")
                        .label("Target")
                        .on_press(with!(target_count, extra => move || {
                            target_count.update(|n| *n += 1);
                            extra.set(true);
                        }))
                        .build(),
                )
                .child(View::text(format!("target={} a={}", target_count.get(), a_count.get())))
                .build()
        },
        40,
        10,
        // reveal, move to Target, press (inserts Extra above), press again
        vec![key(KeyCode::Tab), key(KeyCode::Tab), key(KeyCode::Enter), key(KeyCode::Enter)],
    );
    assert!(output.contains("target=2 a=0"), "got:\n{output}");
}

// ============================================================
// 4. A persistent request doesn't trap focus
// ============================================================

#[test]
fn test_persistent_focus_request_does_not_trap_tab() {
    // FAILED before 0.4: today the request is ignored, so Tab only reveals focus on
    // "First". Implemented wrongly (re-applied every frame) it would press "Second".
    let output = run_headless(
        |cx: Scope| {
            let pressed = state!(cx, String::new);
            let press = |label: &'static str| with!(pressed => move || pressed.set(label.to_string()));
            View::vstack()
                .child(View::button().label("First").on_press(press("First")).build())
                .child(View::button().key("second").focused(true).label("Second").on_press(press("Second")).build())
                .child(View::button().label("Third").on_press(press("Third")).build())
                .child(View::text(format!("pressed: {}", pressed.get())))
                .build()
        },
        40,
        10,
        vec![key(KeyCode::Tab), key(KeyCode::Enter)],
    );
    assert!(output.contains("pressed: Third"), "got:\n{output}");
}

// ============================================================
// 5. Visibility
// ============================================================

/// A bordered scroll box draws double-line corners (`╔`) only when focused *and* focus
/// is visible (`draw_border_focused`).
fn bordered_box(request_focus: bool) -> String {
    run_headless(
        move |_cx: Scope| {
            View::boxed()
                .key("log")
                .focused(request_focus)
                .border(true)
                .scroll(true)
                .child(View::text("log line"))
                .build()
        },
        30,
        8,
        vec![],
    )
}

#[test]
fn test_applied_focus_request_is_visible() {
    // FAILED before 0.4
    let output = bordered_box(true);
    assert!(output.contains('╔'), "requested focus should be visible, got:\n{output}");
}

#[test]
fn test_default_focus_stays_hidden() {
    let output = bordered_box(false);
    assert!(!output.contains('╔'), "default focus should stay hidden, got:\n{output}");
}

// ============================================================
// 6. Key dispatch order
// ============================================================

#[test]
fn test_focused_text_input_beats_plain_letter_command() {
    // FAILED before 0.4: today the 's' command fires and the input never sees it
    let output = run_headless(
        |cx: Scope| {
            let value = state!(cx, String::new);
            let fired = state!(cx, || false);
            cx.use_command(KeyBinding::key(KeyCode::Char('s')), with!(fired => move || fired.set(true)));
            View::vstack()
                .child(
                    View::text_input()
                        .focused(true)
                        .value(value.get())
                        .on_change(with!(value => move |v: String| value.set(v)))
                        .build(),
                )
                .child(View::text(format!("value=[{}] fired={}", value.get(), fired.get())))
                .build()
        },
        40,
        10,
        vec![char_key('s')],
    );
    assert!(output.contains("value=[s] fired=false"), "got:\n{output}");
}

#[test]
fn test_focused_button_beats_enter_command() {
    // FAILED before 0.4: today the Enter command wins
    let output = run_headless(
        |cx: Scope| {
            let who = state!(cx, String::new);
            cx.use_command(KeyBinding::key(KeyCode::Enter), with!(who => move || who.set("command".to_string())));
            View::vstack()
                .child(View::button().label("Go").on_press(with!(who => move || who.set("button".to_string()))).build())
                .child(View::text(format!("who={}", who.get())))
                .build()
        },
        40,
        10,
        vec![key(KeyCode::Enter)],
    );
    assert!(output.contains("who=button"), "got:\n{output}");
}

#[test]
fn test_enter_on_focused_list_reaches_command() {
    // Guard: List has no Enter handler, so Enter falls through to the command
    let output = run_headless(
        |cx: Scope| {
            let fired = state!(cx, || false);
            cx.use_command(KeyBinding::key(KeyCode::Enter), with!(fired => move || fired.set(true)));
            View::vstack()
                .child(View::list().items(vec!["one".to_string(), "two".to_string()]).build())
                .child(View::text(format!("fired={}", fired.get())))
                .build()
        },
        40,
        10,
        vec![key(KeyCode::Enter)],
    );
    assert!(output.contains("fired=true"), "got:\n{output}");
}

#[test]
fn test_ctrl_chord_command_beats_focused_text_input() {
    // Guard: Ctrl/Alt chords always go to commands first
    let output = run_headless(
        |cx: Scope| {
            let fired = state!(cx, || false);
            cx.use_command(KeyBinding::ctrl('s'), with!(fired => move || fired.set(true)));
            View::vstack()
                .child(View::text_input().focused(true).build())
                .child(View::text(format!("fired={}", fired.get())))
                .build()
        },
        40,
        10,
        vec![key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL)],
    );
    assert!(output.contains("fired=true"), "got:\n{output}");
}

// ============================================================
// 7. Duplicate keys keep separate state
// ============================================================

#[test]
fn test_duplicate_keys_keep_separate_scroll_state() {
    // Guard: a duplicate key must not make two boxes share scroll state
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::boxed().key("dup").scroll(true).max_height(4).child(View::text(numbered_lines("X", 30))).build())
            .child(View::boxed().key("dup").scroll(true).max_height(4).child(View::text(numbered_lines("Y", 30))).build())
            .build()
    })
    .with_size(30, 12);

    app.scroll_down(10); // first box (index 0) is focused
    app.assert_not_visible("X 001");
    app.assert_visible("Y 001");
}

// ============================================================
// 8. Modal restores focus by id
// ============================================================

#[test]
fn test_modal_close_restores_focus_to_same_keyed_widget() {
    // FAILED before 0.4: restore uses the saved index, which now holds "A"
    let output = run_headless(
        |cx: Scope| {
            let open = state!(cx, || false);
            let extra = state!(cx, || false);
            let target_count = state!(cx, || 0);
            let a_count = state!(cx, || 0);

            let mut list = View::vstack();
            if extra.get() {
                list = list.child(View::button().label("Extra").build());
            }
            list.child(View::button().label("A").on_press(with!(a_count => move || a_count.update(|n| *n += 1))).build())
                .child(
                    View::button()
                        .key("target")
                        .label("Target")
                        .on_press(with!(target_count, open, extra => move || {
                            target_count.update(|n| *n += 1);
                            open.set(true);
                            extra.set(true);
                        }))
                        .build(),
                )
                .child(View::text(format!("target={} a={}", target_count.get(), a_count.get())))
                .child(
                    View::modal()
                        .visible(open.get())
                        .title("Modal")
                        .child(View::button().label("Close").on_press(with!(open => move || open.set(false))).build())
                        .build(),
                )
                .build()
        },
        50,
        16,
        // reveal, move to Target, press (opens modal, inserts Extra), Close, press again
        vec![
            key(KeyCode::Tab),
            key(KeyCode::Tab),
            key(KeyCode::Enter),
            key(KeyCode::Enter),
            key(KeyCode::Enter),
        ],
    );
    assert!(output.contains("target=2 a=0"), "got:\n{output}");
}

#[test]
fn test_duplicate_keys_are_reported() {
    let mut app = TestApp::new(|_cx: Scope| {
        View::vstack()
            .child(View::button().key("same").label("One").build())
            .child(View::button().key("same").label("Two").build())
            .child(View::button().key("other").label("Three").build())
            .build()
    });
    assert_eq!(app.duplicate_focus_keys(), vec!["same".to_string()]);
}

// ============================================================
// Further dispatch and request cases
// ============================================================

#[test]
fn test_plain_letter_command_fires_when_focused_widget_ignores_it() {
    // A button doesn't handle letters, so the command still gets them
    let output = run_headless(
        |cx: Scope| {
            let fired = state!(cx, || false);
            cx.use_command(KeyBinding::key(KeyCode::Char('s')), with!(fired => move || fired.set(true)));
            View::vstack()
                .child(View::button().focused(true).label("Go").build())
                .child(View::text(format!("fired={}", fired.get())))
                .build()
        },
        40,
        10,
        vec![char_key('s')],
    );
    assert!(output.contains("fired=true"), "got:\n{output}");
}

#[test]
fn test_focus_request_inside_modal_applies_on_open() {
    let output = run_headless(
        |cx: Scope| {
            let open = state!(cx, || false);
            let pressed = state!(cx, String::new);
            let press = |label: &'static str| with!(pressed => move || pressed.set(label.to_string()));
            View::vstack()
                .child(View::button().label("Open").on_press(with!(open => move || open.set(true))).build())
                .child(View::text(format!("pressed: {}", pressed.get())))
                .child(
                    View::modal()
                        .visible(open.get())
                        .title("Modal")
                        .child(
                            View::vstack()
                                .child(View::button().label("Cancel").on_press(press("Cancel")).build())
                                .child(View::button().key("ok").focused(true).label("OK").on_press(press("OK")).build())
                                .build(),
                        )
                        .build(),
                )
                .build()
        },
        50,
        16,
        vec![key(KeyCode::Enter), key(KeyCode::Enter)],
    );
    assert!(output.contains("pressed: OK"), "got:\n{output}");
}
