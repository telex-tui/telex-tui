//! Tests that background streams wake the event loop.
//!
//! These tests verify that `stream!` and `text_stream!` data triggers
//! re-renders without any user input — the bug that caused streaming
//! content to only appear when the user typed the next keystroke.
//!
//! The key assertion: stream content must appear in a rendered frame
//! BEFORE the final quit event. Without the wake fix, only 2 frames
//! render (initial + quit), and content only appears in the quit frame.

use std::time::Duration;
use telex::prelude::*;

#[test]
fn test_text_stream_wakes_event_loop() {
    struct StreamApp;
    impl Component for StreamApp {
        fn render(&self, cx: Scope) -> View {
            let data = text_stream!(cx, || {
                std::thread::sleep(Duration::from_millis(20));
                vec!["hello".to_string(), " world".to_string()]
                    .into_iter()
            });
            View::text(data.get())
        }
    }

    let frames = run_headless_timed(StreamApp, 40, 10, Duration::from_millis(200));

    // Without the wake fix, only 2 frames: initial (empty) + quit (has content).
    // With the fix, wake-triggered frames appear in between.
    assert!(
        frames.len() > 2,
        "Stream should wake event loop for intermediate renders. Only got {} frames (no wake)",
        frames.len()
    );

    // Content should appear in a frame BEFORE the final quit-triggered frame.
    let pre_quit_frames = &frames[..frames.len() - 1];
    let has_content_before_quit = pre_quit_frames
        .iter()
        .any(|f| f.contains("hello"));
    assert!(
        has_content_before_quit,
        "Stream content should appear before quit event (wake-triggered render).\n\
         Frames: {:#?}",
        frames
    );
}

#[test]
fn test_stream_wakes_event_loop() {
    struct CounterApp;
    impl Component for CounterApp {
        fn render(&self, cx: Scope) -> View {
            let counter = stream!(cx, || {
                (1..=3).inspect(|_| std::thread::sleep(Duration::from_millis(20)))
            });
            View::text(format!("count: {}", counter.get()))
        }
    }

    let frames = run_headless_timed(CounterApp, 40, 10, Duration::from_millis(200));

    assert!(
        frames.len() > 2,
        "Stream should wake event loop for intermediate renders. Only got {} frames",
        frames.len()
    );

    // At least one pre-quit frame should show a non-zero count
    let pre_quit_frames = &frames[..frames.len() - 1];
    let has_progress = pre_quit_frames
        .iter()
        .any(|f| f.contains("count: 1") || f.contains("count: 2") || f.contains("count: 3"));
    assert!(
        has_progress,
        "Stream values should appear before quit event.\nFrames: {:#?}",
        frames
    );
}
