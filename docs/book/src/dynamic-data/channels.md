# Channels & Ports

Receive messages from external threads with zero-latency wake-up.

```rust
let ch = channel!(cx, String);

effect_once!(cx, {
    let tx = ch.tx();
    move || {
        std::thread::spawn(move || {
            tx.send("hello from thread".to_string()).ok();
        });
        || {}
    }
});

for msg in ch.get() {
    // handle each message this frame
}
```

Run with: `cargo run -p telex-tui --example 34_channels_and_intervals`

## What are channels?

Channels let external threads push data into your component. Unlike streams (which pull from an iterator), channels are push-based — any thread with a sender can deliver messages at any time.

Each frame, the run loop drains the channel and your render code sees a clean batch of messages via `.get()`.

## channel! — inbound messages

```rust
let ch = channel!(cx, MessageType);
```

Creates a typed inbound channel. Returns a `ChannelHandle<T>` with:

- **`ch.tx()`** — Returns a `WakingSender<T>` that can be cloned and sent to any thread
- **`ch.get()`** — Returns a `Vec<T>` of messages received this frame
- **`ch.len()`** — Number of messages this frame
- **`ch.is_empty()`** — Whether any messages arrived

### WakingSender

The sender returned by `ch.tx()` is special — it wakes the event loop immediately when a message is sent. This means near-zero latency between sending and rendering, instead of waiting for the next 16ms poll cycle.

```rust
let tx = ch.tx();

// Clone and send to multiple threads
let tx2 = tx.clone();
std::thread::spawn(move || {
    tx.send("from thread 1".to_string()).ok();
});
std::thread::spawn(move || {
    tx2.send("from thread 2".to_string()).ok();
});
```

## port! — bidirectional communication

```rust
let io = port!(cx, InboundType, OutboundType);
```

Creates a bidirectional port for two-way communication. Returns a `PortHandle<In, Out>` with:

- **`io.rx`** — A `ChannelHandle<In>` for inbound messages (same API as `channel!`)
- **`io.tx()`** — A `Sender<Out>` for outbound messages
- **`io.take_outbound_rx()`** — Takes the outbound receiver (call once, pass to your thread)

### Example: background worker

```rust
let worker = port!(cx, WorkerResult, WorkerCommand);

effect_once!(cx, {
    let tx_in = worker.rx.tx();
    let rx_out = worker.take_outbound_rx();
    move || {
        std::thread::spawn(move || {
            if let Some(rx) = rx_out {
                for cmd in rx {
                    let result = process(cmd);
                    tx_in.send(result).ok();
                }
            }
        });
        || {}
    }
});

// Send commands
worker.tx().send(WorkerCommand::Start).ok();

// Read results
for result in worker.rx.get() {
    // handle result
}
```

## When to use channels vs streams

**Use channels when:**
- External code pushes data to your component
- You need bidirectional communication
- Messages arrive at unpredictable times
- You want zero-latency wake-up

**Use streams when:**
- You're pulling from an iterator (polling, tailing)
- Data flows one direction continuously
- The data source is self-contained

## Tips

**Frame-buffered delivery** — Messages accumulate between frames. `.get()` returns all messages since the last render as a `Vec<T>`. This means you might get 0, 1, or many messages per frame.

**Sender lifetime** — `WakingSender` is `Send + Sync` and can be cloned freely. When all senders are dropped, the channel still works — it just won't receive new messages.

**Order-independent** — Like all macros, `channel!` and `port!` are keyed by call site. Safe in conditionals.

Next: [Intervals](./intervals.md)
