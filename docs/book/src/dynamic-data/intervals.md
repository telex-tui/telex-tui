# Intervals

Periodic callbacks that run on a fixed schedule.

```rust
let ticks = state!(cx, || 0u64);

interval!(cx, Duration::from_secs(1), with!(ticks => move || {
    ticks.update(|n| *n += 1);
}));

View::text(format!("Ticks: {}", ticks.get()))
```

## What is interval!?

`interval!` sets up a repeating timer that fires a callback on the main thread. Under the hood, it spawns a background thread with a `WakingSender` channel — the timer thread sends a wake-up signal, and the callback runs during the next frame.

```rust
interval!(cx, duration, callback);
```

- **`duration`** — A `std::time::Duration` for the interval period
- **`callback`** — A closure that runs on the main thread each frame the timer fires

## Common uses

### Polling

```rust
let data = state!(cx, || None);

interval!(cx, Duration::from_secs(5), with!(data => move || {
    // Runs every 5 seconds on the main thread
    if let Ok(fresh) = quick_fetch() {
        data.set(Some(fresh));
    }
}));
```

### Animation

```rust
let frame = state!(cx, || 0usize);
let sprites = ["| ", "/ ", "- ", "\\ "];

interval!(cx, Duration::from_millis(100), with!(frame => move || {
    frame.update(|f| *f = (*f + 1) % 4);
}));

View::text(sprites[frame.get()])
```

### Blinking cursor

```rust
let visible = state!(cx, || true);

interval!(cx, Duration::from_millis(500), with!(visible => move || {
    visible.update(|v| *v = !*v);
}));
```

## Interval vs stream

**Use interval! when:**
- You need periodic side effects (polling, animation)
- The callback is simple and runs on the main thread
- You want the timer to fire even if no data changed

**Use stream! when:**
- You need to yield data values over time
- The work runs in a background thread
- You want loading/completion tracking with `.is_loading()`

## Tips

**Main thread execution** — The callback runs on the main thread during the render cycle. Keep it fast — don't do heavy computation or blocking I/O in the callback.

**Order-independent** — Like all macros, `interval!` is keyed by call site. Safe in conditionals and loops.

**Automatic cleanup** — The timer thread stops when the component unmounts.

Next: [Tables](../widgets/tables.md)
