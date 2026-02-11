# Dynamic Data Overview

This section covers three powerful tools for handling dynamic data in your Telex applications. Each serves a different purpose - choose based on your use case.

## Quick Decision Guide

**Use [Streams](./streams.md) when:**
- You have continuous background data that updates over time
- Examples: timers, system monitoring, log tailing, live feeds
- Data keeps flowing, no clear "done" state
- Pattern: Background thread yielding values repeatedly

**Use [Effects](../experimental/effects.md) when:**
- You need to react to state changes with side effects
- Examples: logging, saving to disk, sending analytics, cleanup
- One-time initialization or responding to dependency changes
- Pattern: Run code after render when state changes

**Use [Async Data](./async.md) when:**
- You need to load data once with a clear start and end
- Examples: API calls, database queries, file loading
- Clear loading → ready/error lifecycle
- Pattern: Background task with loading state management

**Use [Channels & Ports](./channels.md) when:**
- External threads need to push data into your component
- You need bidirectional communication with a background system
- Examples: WebSocket handlers, hardware I/O, message queues
- Pattern: Spawn thread with sender, drain messages each frame

**Use [Intervals](./intervals.md) when:**
- You need periodic callbacks on a fixed schedule
- Examples: polling, animation ticks, blinking cursor
- Pattern: Timer fires, callback runs on main thread each frame

## Common Patterns

### Timer/Clock Display
Use **streams** - continuous time updates:
```rust
let elapsed = stream!(cx, || {
    (0..).map(|s| {
        std::thread::sleep(Duration::from_secs(1));
        s
    })
});
```

### Auto-save
Use **effects** - save when document changes:
```rust
effect!(cx, document.get(), |doc| {
    save_to_file("autosave.txt", doc);
    || {}
});
```

### Loading User Profile
Use **async** - one-time fetch with loading state:
```rust
let profile = async_data!(cx, || {
    fetch_user_from_api()
});

match &profile {
    Async::Loading => View::text("Loading..."),
    Async::Ready(user) => View::text(&user.name),
    Async::Error(e) => View::text(e),
}
```

## Combining Approaches

You can use multiple approaches together:

```rust
// Load initial data (async)
let data = async_data!(cx, || fetch_initial_data());

// Save changes (effect)
effect!(cx, data.clone(), |d| {
    if let Async::Ready(items) = d {
        save_to_disk(items);
    }
    || {}
});

// Show live stats (stream)
let stats = stream!(cx, || {
    (0..).map(|_| {
        std::thread::sleep(Duration::from_secs(5));
        get_stats()
    })
});
```

## Next Steps

Start with the chapter that matches your use case:
- [Streams →](./streams.md)
- [Effects →](../experimental/effects.md)
- [Async Data →](./async.md)
- [Channels & Ports →](./channels.md)
- [Intervals →](./intervals.md)
