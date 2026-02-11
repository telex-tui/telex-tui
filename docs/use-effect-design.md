# Effect API Design

> **Status: ✅ Implemented** — See `crates/telex/src/scope.rs` and example `32_effects`

Side effects for Telex components — reacting to state changes, timers, subscriptions, cleanup.

## Quick Start

```rust
use telex::prelude::*;

fn render(&self, cx: Scope) -> View {
    let count = state!(cx, || 0);

    // Runs when count changes
    effect!(cx, count.get(), |&c| {
        println!("Count changed to {}", c);
        || {}  // cleanup
    });

    // Runs once on initialization
    effect_once!(cx, || {
        println!("App started");
        || println!("App cleanup")
    });

    // ...
}
```

---

## The Macros

### `effect!` — Run when dependencies change

```rust
effect!(cx, deps, |&d| {
    // effect body - runs when deps change
    || {
        // cleanup - runs before next effect or on exit
    }
});
```

**Examples:**

```rust
// Single dependency
effect!(cx, count.get(), |&c| {
    println!("Count: {}", c);
    || {}
});

// Multiple dependencies via tuple
effect!(cx, (a.get(), b.get()), |&(a, b)| {
    println!("a={}, b={}", a, b);
    || {}
});

// With state capture using with!
effect!(cx, user_id.get(), with!(api_client => move |&id| {
    api_client.prefetch_user(id);
    || {}
}));
```

### `effect_once!` — Run once on first render

```rust
effect_once!(cx, || {
    // initialization code
    || {
        // cleanup on app exit
    }
});
```

**Examples:**

```rust
// Simple initialization
effect_once!(cx, || {
    println!("App initialized");
    || {}
});

// With cleanup
effect_once!(cx, || {
    let handle = subscribe_to_events();
    move || {
        unsubscribe(handle);
    }
});

// With state capture
effect_once!(cx, with!(config => move || {
    config.set(load_config_from_disk());
    || {}
}));
```

---

## Key Properties

### Order-Independent (Safe in Conditionals)

Unlike React's hooks, the `effect!` and `effect_once!` macros are **keyed by call site**, not by call order. This means they're safe to use in conditionals:

```rust
// SAFE with effect! macro
if feature_enabled {
    effect!(cx, data.get(), |d| {
        log_feature_usage(d);
        || {}
    });
}
```

The macro generates a unique type at each call site, which becomes the key. Same call site = same effect. Different call sites = different effects.

### Cleanup Semantics

Cleanup functions run:
1. **Before the next effect** when dependencies change
2. **On app exit** for all effects

```rust
effect!(cx, url.get(), |url| {
    let connection = connect(url);
    move || {
        connection.close();  // runs before next connect or on exit
    }
});
```

### Effects Run After Render

Effects are scheduled during render but executed after the view is drawn to the terminal. This ensures the UI updates before side effects run.

---

## Cycle Detection

Telex includes automatic detection of infinite loops:

```rust
// BAD: This will panic after ~100 iterations
effect!(cx, count.get(), |_| {
    count.update(|n| *n += 1);  // updates own dependency!
    || {}
});
```

If effects run more than 100 times within 10 frames, Telex panics with a helpful error message explaining the issue.

**The Rule:** Effects should flow *outward* (to external systems) or *sideways* (to different state), never back to their own dependencies.

---

## Legacy API

The index-based APIs still exist for backwards compatibility:

- `cx.use_effect(|| ...)` — runs every render
- `cx.use_effect_once(|| ...)` — runs once
- `cx.use_effect_with(deps, |d| ...)` — runs when deps change

**⚠️ These are order-dependent** and will break if called conditionally. Prefer the macros for new code.

---

## Common Patterns

### Logging State Changes

```rust
effect!(cx, count.get(), |&c| {
    eprintln!("[debug] count = {}", c);
    || {}
});
```

### Syncing to External Storage

```rust
effect!(cx, settings.get(), with!(storage => move |s| {
    storage.save("settings", s);
    || {}
}));
```

### One-Time Setup

```rust
effect_once!(cx, with!(data => move || {
    data.set(load_initial_data());
    || {}
}));
```

### Derived State (prefer computed instead)

```rust
// This works but is verbose:
effect!(cx, items.get(), with!(total => move |items| {
    total.set(items.iter().sum());
    || {}
}));

// Better: compute directly in render
let total: i32 = items.get().iter().sum();
```

---

## Known Limitations

### 1. Sync Only

Effects run synchronously on the main thread. Long-running effects freeze the UI.

```rust
// BAD: blocks UI
effect!(cx, query.get(), |q| {
    let result = expensive_sync_fetch(q);  // UI frozen!
    || {}
});
```

For async work, use `use_async` or `use_stream` instead.

### 2. No Component Lifecycle

There's no "unmount" in Telex (single root component). Cleanup only runs:
- When dependencies change (before next effect)
- On app exit

If you conditionally stop rendering something, its cleanup doesn't automatically run.

### 3. Stale Closures

Closures capture values at creation time:

```rust
effect_once!(cx, || {
    // This closure captures `count` at creation time
    set_interval(1000, || {
        println!("{}", count.get());  // May see stale value!
    });
    || {}
});
```

Fix: Use `effect!` with dependencies so the effect re-runs when values change.

---

## Implementation Notes

### Storage

Keyed effects use `HashMap<TypeId, EffectState>` where the key is generated by the macro:

```rust
// effect!(cx, deps, ...) expands to:
{
    struct __Effect_42;  // unique per call site
    cx.use_effect_keyed::<__Effect_42, _, _, _>(deps, ...)
}
```

### Execution Flow

```
1. Render: component.render(cx) called
   - effect!/effect_once! schedule effects (don't run yet)
   - Compare deps to stored deps
   - If changed, add to pending_keyed_effects

2. Draw: view rendered to terminal

3. Flush: storage.flush_effects() called
   - Run cleanup for effects that will re-run
   - Run pending effects
   - Store new cleanup functions

4. Loop: wait for input, then back to 1
```

---

## See Also

- Example: `cargo run -p telex-tui --example 32_effects`
- Source: `crates/telex/src/scope.rs`
- Macros: `crates/telex-macro/src/lib.rs`
