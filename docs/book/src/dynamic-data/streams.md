# Streams

Background data sources that update the UI automatically over time.

```rust
let counter = stream!(cx, || {
    (0..).map(|i| {
        std::thread::sleep(Duration::from_secs(1));
        i
    })
});

View::text(format!("Count: {}", counter.get()))
```

Run with: `cargo run -p telex-tui --example 04_timer`

## What are streams?

Streams let you integrate background data sources into your UI. Each value the stream yields triggers a re-render with the new data.

Common uses:
- Timers and clocks
- System monitoring (CPU, memory, network)
- Log tailing
- Live data feeds
- Progress tracking

## When to Use Streams

**Use streams when:**
- Data updates continuously over time
- You're polling, monitoring, or tailing
- The data source is external (files, system, network)

**Use effects instead when:**
- You need to react to state changes
- You're doing one-time initialization
- You need cleanup logic (timers, subscriptions)

**Use async instead when:**
- One-time data fetch (API call, database query)
- Clear start and end (loading → ready/error)

See [Overview](./overview.md) for more guidance on choosing the right approach.

## Creating a stream

Use the `stream!` macro with a closure that returns an iterator:

```rust
let elapsed = stream!(cx, || {
    (0u64..).map(|s| {
        std::thread::sleep(Duration::from_secs(1));
        s
    })
});
```

The stream runs in a background thread. Each time the iterator yields a value, Telex triggers a re-render.

## Reading stream values

Use `.get()` to read the latest value:

```rust
let seconds = elapsed.get();
View::text(format!("Elapsed: {}s", seconds))
```

`.get()` returns the most recent value yielded by the stream.

## Checking stream status

Use `.is_loading()` to check if the stream is still active:

```rust
let is_running = stream.is_loading();

if is_running {
    View::styled_text(" ●").color(Color::Green).build()  // active
} else {
    View::styled_text(" ○").dim().build()  // stopped
}
```

A stream is "loading" while it's producing values. Once the iterator ends or the component unmounts, `is_loading()` returns false.

## Multiple streams

You can create multiple independent streams:

```rust
let cpu = stream!(cx, || {
    (0..).map(|_| {
        std::thread::sleep(Duration::from_millis(500));
        get_cpu_usage()
    })
});

let memory = stream!(cx, || {
    (0..).map(|_| {
        std::thread::sleep(Duration::from_millis(800));
        get_memory_usage()
    })
});

let network = stream!(cx, || {
    (0..).map(|_| {
        std::thread::sleep(Duration::from_millis(300));
        get_network_traffic()
    })
});
```

Run with: `cargo run -p telex-tui --example 08_system_monitor`

Each stream runs independently in its own thread. They update at different rates and don't block each other.

## Stream lifecycle

**Creation:** The stream starts when the component first renders and calls `stream!`.

**Updates:** Each yielded value triggers a re-render. The UI shows the latest value.

**Cleanup:** When the component unmounts, the stream is automatically stopped and the background thread is cleaned up.

You don't manually start, stop, or manage stream threads.

## Infinite vs finite streams

**Infinite streams** run until the component unmounts:

```rust
let timer = stream!(cx, || {
    (0..).map(|i| {  // infinite iterator
        std::thread::sleep(Duration::from_secs(1));
        i
    })
});
```

**Finite streams** stop after producing all values:

```rust
let countdown = stream!(cx, || {
    (0..=10).rev().map(|i| {  // 10, 9, 8, ..., 0
        std::thread::sleep(Duration::from_secs(1));
        i
    })
});

// After 11 seconds, is_loading() returns false
```

## Reactive updates

Stream values are reactive. Render automatically when they change:

```rust
let cpu_val = cpu.get();

let color = if cpu_val > 80 {
    Color::Red
} else if cpu_val > 50 {
    Color::Yellow
} else {
    Color::Green
};

View::styled_text(format!("CPU: {}%", cpu_val))
    .color(color)
    .build()
```

Each time `cpu` yields a new value, the color recalculates and the UI updates.

## Error handling

If your stream can fail, yield `Result` values:

```rust
let data = stream!(cx, || {
    (0..).map(|_| {
        std::thread::sleep(Duration::from_secs(1));
        fetch_data()  // returns Result<T, E>
    })
});

match data.get() {
    Ok(value) => View::text(format!("Data: {}", value)),
    Err(e) => View::styled_text(format!("Error: {}", e)).color(Color::Red).build(),
}
```

## Progress indicators

Combine finite streams with progress bars:

```rust
let progress = stream!(cx, || {
    (0..=100).map(|i| {
        std::thread::sleep(Duration::from_millis(50));
        i
    })
});

let pct = progress.get();
let is_done = !progress.is_loading();

View::vstack()
    .child(View::text(format!("Progress: {}%", pct)))
    .child(progress_bar(pct))
    .child(if is_done {
        View::text("Complete!")
    } else {
        View::text("Working...")
    })
    .build()
```

## Real-world patterns

**Log viewer:**
```rust
let logs = stream!(cx, || {
    std::fs::File::open("/var/log/app.log")
        .and_then(|file| {
            std::io::BufReader::new(file)
                .lines()
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default()
        .into_iter()
});
```

**System stats:**
```rust
let stats = stream!(cx, || {
    (0..).map(|_| {
        std::thread::sleep(Duration::from_secs(2));
        SystemStats {
            cpu: get_cpu(),
            memory: get_memory(),
            disk: get_disk(),
        }
    })
});
```

**Periodic API polling:**
```rust
let api_data = stream!(cx, || {
    (0..).map(|_| {
        std::thread::sleep(Duration::from_secs(30));
        fetch_from_api()
    })
});
```

## Performance considerations

**Update frequency:** Don't yield faster than the UI can render. Sleeping for at least 100-300ms between yields is reasonable.

**Thread overhead:** Each stream is a thread. Dozens of streams is fine, hundreds might be excessive.

**Data size:** Stream values are cloned when accessed with `.get()`. Keep them reasonably sized, or wrap large data in `Rc<T>`.

**Blocking operations:** Stream closures run in background threads, so blocking operations (file I/O, network calls) are fine.

## Tips

**Sleep on the first iteration** - Streams yield their first value immediately. If your stream sleeps 1 second between yields, put the sleep at the start to avoid an instant first value followed by a delay.

**Finite streams for progress** - If your task has a known endpoint, use a finite iterator and check `is_loading()` to detect completion.

**Channels for external events** - If you need to push data from outside (another thread, a library callback), use the `channel!` macro instead of wrapping a channel in a stream:

```rust
let ch = channel!(cx, String);
let tx = ch.tx();  // WakingSender — wakes event loop instantly
// Pass tx to external code, read with ch.get() in render
```

See [Channels & Ports](./channels.md) for full details.

**Don't use streams for user input** - User actions should go through state and callbacks, not streams. Streams are for external data sources.

Next: [Channels & Ports](./channels.md)
