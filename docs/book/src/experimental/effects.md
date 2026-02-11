# Effects

Side effects that run after rendering, reacting to state changes or initialization.

```rust
// Run once on mount
effect_once!(cx, || {
    println!("Component mounted!");
    || {}  // cleanup function
});

// Run when dependency changes
effect!(cx, count.get(), |&val| {
    println!("Count changed to {}", val);
    || {}  // cleanup
});
```

Run with: `cargo run -p telex-tui --example 32_effects`

## What are effects?

Effects let you run code in response to component lifecycle events or state changes. They're for **side effects** - things that aren't directly rendering:

- Logging or analytics
- Saving to local storage
- Starting timers or intervals
- Subscribing to external events
- Updating window title
- Sending metrics

## The Macros

### `effect!` — Run when dependencies change

```rust
effect!(cx, deps, |&d| {
    // effect body
    || {}  // cleanup
});
```

### `effect_once!` — Run once on first render

```rust
effect_once!(cx, || {
    // initialization
    || {}  // cleanup
});
```

Both macros are **order-independent** — safe to use in conditionals, loops, or any order.

## When to Use Effects

**Use effects when:**
- Reacting to state changes (logging, saving, notifications)
- One-time initialization (loading config, setting up)
- Need cleanup logic (timers, subscriptions, resources)

**Use streams instead when:**
- Continuous data source (polling, monitoring, tailing)
- Data drives the UI (timers, live feeds, progress)
- External events (files, system stats, network)

**Use async instead when:**
- One-time data fetch with loading state
- API calls, database queries, file loading

## Effect timing

Effects run **after** the render completes. The rendering cycle:

1. State changes
2. `render()` is called, returns a `View`
3. View is displayed to the screen
4. Effects run

This ensures effects see the updated UI state and don't block rendering.

## One-time effects

Use `effect_once!` for initialization that should happen exactly once:

```rust
let initialized = state!(cx, || false);

effect_once!(cx, with!(initialized => move || {
    // Runs only on first render
    println!("App started!");
    load_config();
    initialized.set(true);

    || {}  // cleanup (runs on unmount)
}));
```

Common uses:
- Loading initial data
- Setting up subscriptions
- Logging app start
- Initializing third-party libraries

## Reactive effects

Use `effect!` to run code when a dependency changes:

```rust
let count = state!(cx, || 0);

effect!(cx, count.get(), with!(count => move |&value| {
    println!("Count is now: {}", value);
    save_count_to_disk(value);
    || {}  // cleanup
}));
```

The effect runs:
- Once when the component first renders
- Every time the dependency value changes

## Dependencies

The second parameter to `effect!` is the dependency. When it changes (via `PartialEq`), the effect runs:

```rust
// Single dependency
effect!(cx, count.get(), |&c| {
    log_count(c);
    || {}
});

// String dependency
effect!(cx, name.get(), |n: &String| {
    save_name(n.clone());
    || {}
});

// Tuple for multiple dependencies
effect!(cx, (count.get(), name.get()), |(c, n)| {
    log_event(c, n);
    || {}
});
```

The effect closure receives a reference to the dependency value.

## Cleanup functions

Effects return a cleanup closure that runs when:
- The app exits
- Before the effect runs again (if the dependency changed)

```rust
effect!(cx, interval_ms.get(), move |&ms| {
    let (tx, rx) = std::sync::mpsc::channel();

    // Start a timer
    let handle = std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(ms));
            if tx.send(()).is_err() {
                break;  // receiver dropped
            }
        }
    });

    // Cleanup: stop the timer
    move || {
        drop(rx);  // This will cause the thread to exit
        let _ = handle.join();
    }
});
```

If you don't need cleanup, return an empty closure: `|| {}`.

## Comparing effect types

| Type | When it runs | Use for |
|------|--------------|---------|
| `effect_once!` | Once on mount | Initialization, setup |
| `effect!` | When dependency changes | Reacting to state, saving data |

## Safe in conditionals

Unlike React hooks, the `effect!` and `effect_once!` macros are keyed by call site, not call order:

```rust
// SAFE: effect in conditional
if feature_enabled {
    effect!(cx, data.get(), |d| {
        log_feature_usage(d);
        || {}
    });
}
```

This works because each macro invocation gets a unique key based on its location in the source code.

## Logging example

Track when state changes:

```rust
let count = state!(cx, || 0);

effect!(cx, count.get(), move |&c| {
    eprintln!("[{}] Count changed to {}", chrono::Utc::now(), c);
    || {}
});
```

## Local storage example

Persist state to disk:

```rust
let todos = state!(cx, Vec::new);

effect!(cx, todos.get(), move |items| {
    let json = serde_json::to_string(items).unwrap();
    std::fs::write("todos.json", json).ok();
    || {}
});
```

Every time `todos` changes, the list is saved.

## Timer example

Start a repeating timer:

```rust
let ticks = state!(cx, || 0);

effect_once!(cx, with!(ticks => move || {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            if tx.send(()).is_err() { break; }
        }
    });

    std::thread::spawn(with!(ticks => move || {
        while rx.recv().is_ok() {
            ticks.update(|t| *t += 1);
        }
    }));

    move || drop(rx)  // cleanup stops both threads
}));
```

Note: For simpler cases, use `stream!` instead. Effects are better when you need cleanup logic.

## Multiple effects

You can have multiple effects in one component:

```rust
// Initialize
effect_once!(cx, || {
    load_settings();
    || {}
});

// Save count
effect!(cx, count.get(), |&c| {
    save_count(c);
    || {}
});

// Save name
effect!(cx, name.get(), |n| {
    save_name(n);
    || {}
});
```

Each effect is independent. They don't interfere with each other.

## Avoiding infinite loops

Don't update the same state you're watching:

```rust
// ❌ BAD: Infinite loop (will panic with cycle detection)
effect!(cx, count.get(), with!(count => move |&c| {
    count.set(c + 1);  // triggers effect again!
    || {}
}));

// ✅ GOOD: Update different state
effect!(cx, count.get(), with!(double => move |&c| {
    double.set(c * 2);  // updates different state
    || {}
}));
```

Telex includes automatic cycle detection — if an effect runs more than 100 times in 10 frames, it panics with a helpful error message.

## Tips

**Return cleanup even if empty** - Always return `|| {}` to match the expected signature.

**Clone state for effects** - Use `with!` to capture state handles in effect closures.

**Effects don't block rendering** - They run after the UI updates, so heavy work in effects won't freeze the UI.

**One effect per concern** - Don't cram multiple unrelated side effects into one effect. Separate them for clarity.

**Dependencies must be values** - Pass `state.get()`, not the `State<T>` handle itself. The effect watches the value, not the handle.

**Cleanup runs before re-running** - If your dependency changes rapidly, cleanup runs each time. Keep cleanup fast.

**Effects can't return values** - If you need computed values from state, do that during render, not in effects. Effects are for side effects only.

## Debugging effects

Add logging to see when effects run:

```rust
effect!(cx, count.get(), |&c| {
    eprintln!("Effect running: count = {}", c);
    do_work(c);
    move || eprintln!("Cleanup running")
});
```

This helps track effect timing and dependencies.

Next: [Async Data](./async.md)
