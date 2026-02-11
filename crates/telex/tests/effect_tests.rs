//! Tests for the effect API (both legacy index-based and new keyed macros).

use std::cell::RefCell;
use std::rc::Rc;
use telex::prelude::*;

/// Helper to track effect calls.
#[derive(Default, Clone)]
struct EffectTracker {
    effect_count: Rc<RefCell<u32>>,
    cleanup_count: Rc<RefCell<u32>>,
    values: Rc<RefCell<Vec<i32>>>,
}

impl EffectTracker {
    fn new() -> Self {
        Self::default()
    }

    fn effect_calls(&self) -> u32 {
        *self.effect_count.borrow()
    }

    fn cleanup_calls(&self) -> u32 {
        *self.cleanup_count.borrow()
    }

    fn values(&self) -> Vec<i32> {
        self.values.borrow().clone()
    }

    fn record_effect(&self) {
        *self.effect_count.borrow_mut() += 1;
    }

    fn record_cleanup(&self) {
        *self.cleanup_count.borrow_mut() += 1;
    }

    fn record_value(&self, v: i32) {
        self.values.borrow_mut().push(v);
    }
}

// ========== use_effect tests ==========

#[test]
fn test_use_effect_runs_every_render() {
    let tracker = EffectTracker::new();
    let tracker_clone = tracker.clone();

    struct TestComponent {
        tracker: EffectTracker,
    }

    impl Component for TestComponent {
        fn render(&self, cx: Scope) -> View {
            let t = self.tracker.clone();
            cx.use_effect(move || {
                t.record_effect();
                || {}
            });
            View::text("test")
        }
    }

    let mut app = telex::testing::TestApp::new(TestComponent {
        tracker: tracker_clone.clone(),
    });

    // First render
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 1);

    // Second render (simulated by rendering again)
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 2);

    // Third render
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 3);
}

#[test]
fn test_use_effect_cleanup_runs_before_next_effect() {
    let tracker = EffectTracker::new();
    let tracker_clone = tracker.clone();

    struct TestComponent {
        tracker: EffectTracker,
    }

    impl Component for TestComponent {
        fn render(&self, cx: Scope) -> View {
            let t = self.tracker.clone();
            let t2 = self.tracker.clone();
            cx.use_effect(move || {
                t.record_effect();
                move || {
                    t2.record_cleanup();
                }
            });
            View::text("test")
        }
    }

    let mut app = telex::testing::TestApp::new(TestComponent {
        tracker: tracker_clone.clone(),
    });

    // First render - effect runs, no cleanup yet
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 1);
    assert_eq!(tracker_clone.cleanup_calls(), 0);

    // Second render - cleanup runs, then effect
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 2);
    assert_eq!(tracker_clone.cleanup_calls(), 1);

    // Third render - cleanup runs, then effect
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 3);
    assert_eq!(tracker_clone.cleanup_calls(), 2);
}

// ========== use_effect_once tests ==========

#[test]
fn test_use_effect_once_runs_only_once() {
    let tracker = EffectTracker::new();
    let tracker_clone = tracker.clone();

    struct TestComponent {
        tracker: EffectTracker,
    }

    impl Component for TestComponent {
        fn render(&self, cx: Scope) -> View {
            let t = self.tracker.clone();
            cx.use_effect_once(move || {
                t.record_effect();
                || {}
            });
            View::text("test")
        }
    }

    let mut app = telex::testing::TestApp::new(TestComponent {
        tracker: tracker_clone.clone(),
    });

    // First render - effect runs
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 1);

    // Second render - effect does NOT run
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 1);

    // Third render - effect still doesn't run
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 1);
}

// ========== use_effect_with tests ==========

#[test]
fn test_use_effect_with_runs_on_dep_change() {
    let tracker = EffectTracker::new();
    let tracker_clone = tracker.clone();

    struct TestComponent {
        tracker: EffectTracker,
    }

    impl Component for TestComponent {
        fn render(&self, cx: Scope) -> View {
            let count = cx.use_state(|| 0);
            let t = self.tracker.clone();

            cx.use_effect_with(count.get(), move |&c| {
                t.record_value(c);
                || {}
            });

            View::vstack()
                .child(View::text(format!("Count: {}", count.get())))
                .child(
                    View::button()
                        .label("+")
                        .on_press(move || count.update(|n| *n += 1))
                        .build(),
                )
                .build()
        }
    }

    let mut app = telex::testing::TestApp::new(TestComponent {
        tracker: tracker_clone.clone(),
    });

    // First render - effect runs with initial value
    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![0]);

    // Re-render without state change - effect does NOT run
    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![0]);

    // Press button to change state
    app.press_button("+");

    // Render with new state - effect runs
    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![0, 1]);

    // Re-render without state change - effect does NOT run
    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![0, 1]);
}

#[test]
fn test_use_effect_with_tuple_deps() {
    let tracker = EffectTracker::new();
    let tracker_clone = tracker.clone();

    struct TestComponent {
        tracker: EffectTracker,
    }

    impl Component for TestComponent {
        fn render(&self, cx: Scope) -> View {
            let a = cx.use_state(|| 1);
            let b = cx.use_state(|| 2);
            let t = self.tracker.clone();

            cx.use_effect_with((a.get(), b.get()), move |&(a, b)| {
                t.record_value(a + b);
                || {}
            });

            View::hstack()
                .child(
                    View::button()
                        .label("A+")
                        .on_press(move || a.update(|n| *n += 1))
                        .build(),
                )
                .child(
                    View::button()
                        .label("B+")
                        .on_press(move || b.update(|n| *n += 1))
                        .build(),
                )
                .build()
        }
    }

    let mut app = telex::testing::TestApp::new(TestComponent {
        tracker: tracker_clone.clone(),
    });

    // First render
    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![3]); // 1 + 2

    // Change a
    app.press_button("A+");
    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![3, 4]); // 2 + 2

    // Change b
    app.press_button("B+");
    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![3, 4, 5]); // 2 + 3
}

// ========== Cycle detection tests ==========

#[test]
#[should_panic(expected = "Effect Cycle Detected")]
fn test_effect_cycle_detection_panics() {
    // This test verifies that the cycle detection catches infinite loops.
    // An effect that updates state on every render will trigger the guard.

    struct InfiniteLoopComponent;

    impl Component for InfiniteLoopComponent {
        fn render(&self, cx: Scope) -> View {
            let count = cx.use_state(|| 0);

            // BAD: This effect updates state every render, causing infinite loop
            let c = count.clone();
            cx.use_effect(move || {
                c.update(|n| *n += 1);
                || {}
            });

            View::text(format!("Count: {}", count.get()))
        }
    }

    let mut app = telex::testing::TestApp::new(InfiniteLoopComponent);

    // Render many times to trigger the cycle detection
    // The threshold is 100 effect runs per 10 frames
    for _ in 0..150 {
        app.render_to_string();
    }
}

// ========== Multiple effects tests ==========

#[test]
fn test_multiple_effects_run_in_order() {
    let tracker = EffectTracker::new();
    let tracker_clone = tracker.clone();

    struct TestComponent {
        tracker: EffectTracker,
    }

    impl Component for TestComponent {
        fn render(&self, cx: Scope) -> View {
            let t1 = self.tracker.clone();
            let t2 = self.tracker.clone();
            let t3 = self.tracker.clone();

            cx.use_effect(move || {
                t1.record_value(1);
                || {}
            });

            cx.use_effect(move || {
                t2.record_value(2);
                || {}
            });

            cx.use_effect(move || {
                t3.record_value(3);
                || {}
            });

            View::text("test")
        }
    }

    let mut app = telex::testing::TestApp::new(TestComponent {
        tracker: tracker_clone.clone(),
    });

    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![1, 2, 3]);
}

// ========== Keyed effect macro tests ==========

#[test]
fn test_effect_once_macro_runs_only_once() {
    let tracker = EffectTracker::new();
    let tracker_clone = tracker.clone();

    struct TestComponent {
        tracker: EffectTracker,
    }

    impl Component for TestComponent {
        fn render(&self, cx: Scope) -> View {
            let t = self.tracker.clone();
            effect_once!(cx, move || {
                t.record_effect();
                || {}
            });
            View::text("test")
        }
    }

    let mut app = telex::testing::TestApp::new(TestComponent {
        tracker: tracker_clone.clone(),
    });

    // First render - effect runs
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 1);

    // Second render - effect does NOT run
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 1);

    // Third render - effect still doesn't run
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 1);
}

#[test]
fn test_effect_macro_runs_on_dep_change() {
    let tracker = EffectTracker::new();
    let tracker_clone = tracker.clone();

    struct TestComponent {
        tracker: EffectTracker,
    }

    impl Component for TestComponent {
        fn render(&self, cx: Scope) -> View {
            let count = state!(cx, || 0);
            let t = self.tracker.clone();

            effect!(cx, count.get(), move |&c| {
                t.record_value(c);
                || {}
            });

            View::vstack()
                .child(View::text(format!("Count: {}", count.get())))
                .child(
                    View::button()
                        .label("+")
                        .on_press(with!(count => move || count.update(|n| *n += 1)))
                        .build(),
                )
                .build()
        }
    }

    let mut app = telex::testing::TestApp::new(TestComponent {
        tracker: tracker_clone.clone(),
    });

    // First render - effect runs with initial value
    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![0]);

    // Re-render without state change - effect does NOT run
    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![0]);

    // Press button to change state
    app.press_button("+");

    // Render with new state - effect runs
    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![0, 1]);

    // Re-render without state change - effect does NOT run
    app.render_to_string();
    assert_eq!(tracker_clone.values(), vec![0, 1]);
}

#[test]
fn test_effect_macro_safe_in_conditional() {
    // This test verifies that effect! macro is safe in conditionals
    // (unlike the index-based use_effect which would break)

    use std::cell::RefCell as StdRefCell;
    use std::rc::Rc as StdRc;

    let tracker = EffectTracker::new();
    let tracker_clone = tracker.clone();

    struct ConditionalEffectComponent {
        tracker: EffectTracker,
        show: StdRc<StdRefCell<bool>>,
    }

    impl Component for ConditionalEffectComponent {
        fn render(&self, cx: Scope) -> View {
            let show = *self.show.borrow();
            let t = self.tracker.clone();

            // This is SAFE with effect! macro (keyed, order-independent)
            if show {
                effect!(cx, (), move |_| {
                    t.record_effect();
                    || {}
                });
            }

            View::text(format!("show={}", show))
        }
    }

    let show = StdRc::new(StdRefCell::new(true));
    let mut app = telex::testing::TestApp::new(ConditionalEffectComponent {
        tracker: tracker_clone.clone(),
        show: show.clone(),
    })
    .with_size(30, 5);

    // First render with show=true - effect runs
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 1);

    // Toggle condition
    *show.borrow_mut() = false;

    // Second render with show=false - effect doesn't run (and no panic!)
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 1);

    // Toggle back
    *show.borrow_mut() = true;

    // Third render with show=true - effect runs again (deps unchanged, but keyed effect remembers)
    // Actually for effect! with deps=(), it won't re-run since deps didn't change
    app.render_to_string();
    assert_eq!(tracker_clone.effect_calls(), 1);
}
