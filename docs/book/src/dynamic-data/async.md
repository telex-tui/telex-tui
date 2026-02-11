# Async Data

Load data asynchronously with automatic loading and error state management.

```rust
let user = cx.use_async(|| {
    std::thread::sleep(Duration::from_secs(2));
    fetch_user_from_api()
});

match &user {
    Async::Loading => View::text("Loading..."),
    Async::Ready(data) => View::text(format!("Welcome, {}!", data.name)),
    Async::Error(e) => View::styled_text(format!("Error: {}", e)).color(Color::Red).build(),
}
```

Run with: `cargo run -p telex-tui --example 24_async_data`

## What is use_async?

`use_async` runs a closure in a background thread and returns an `Async<T>` enum representing the current state:

```rust
pub enum Async<T> {
    Loading,           // Task is running
    Ready(T),          // Task succeeded
    Error(String),     // Task failed
}
```

This handles the common pattern of showing loading spinners, displaying data when ready, and showing errors when things fail.

## Basic usage

```rust
let data = cx.use_async(|| {
    // This runs in a background thread
    std::thread::sleep(Duration::from_secs(1));

    // Return Result<T, String>
    Ok("Data loaded successfully".to_string())
});

// Pattern match on the state
match &data {
    Async::Loading => View::text("Loading..."),
    Async::Ready(value) => View::text(value),
    Async::Error(err) => View::text(format!("Error: {}", err)),
}
```

The closure must return `Result<T, String>` where `T` is your data type.

## Lifecycle

1. **Start:** When the component first renders, the async task starts immediately
2. **Loading:** `Async::Loading` until the task completes
3. **Complete:** Once finished, becomes either `Async::Ready(data)` or `Async::Error(msg)`
4. **Cleanup:** If the component unmounts while loading, the task is cancelled

## Helper methods

Instead of pattern matching, you can use helper methods:

```rust
let data = cx.use_async(|| fetch_data());

// Check state
if data.is_loading() {
    // Show spinner
}

if data.is_error() {
    // Show error banner
}

// Get value if ready
if let Some(value) = data.as_ref().ok() {
    // Use value
}
```

## Multiple async operations

Load multiple things in parallel:

```rust
let profile = cx.use_async(|| {
    std::thread::sleep(Duration::from_secs(2));
    Ok(fetch_user_profile())
});

let stats = cx.use_async(|| {
    std::thread::sleep(Duration::from_secs(1));
    Ok(fetch_user_stats())
});

let posts = cx.use_async(|| {
    std::thread::sleep(Duration::from_secs(3));
    Ok(fetch_user_posts())
});

// Each loads independently
// They don't wait for each other
```

Each `use_async` runs in its own thread. They start simultaneously and complete at different times.

## Error handling

Return `Err` to indicate failure:

```rust
let data = cx.use_async(|| {
    match fetch_from_api() {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("API error: {}", e)),
    }
});

match &data {
    Async::Error(msg) => View::styled_text(msg).color(Color::Red).build(),
    _ => // ... handle loading and ready states
}
```

The error message is stored as a `String`.

## Loading UI patterns

**Simple spinner:**
```rust
match &data {
    Async::Loading => View::text("Loading..."),
    Async::Ready(d) => render_data(d),
    Async::Error(e) => render_error(e),
}
```

**Progress bar:**
```rust
match &profile {
    Async::Loading => View::vstack()
        .child(View::text("Loading profile..."))
        .child(View::text("[==========>          ]"))
        .build(),
    Async::Ready(p) => render_profile(p),
    Async::Error(e) => render_error(e),
}
```

**Overall status:**
```rust
let all_loaded = !profile.is_loading() && !stats.is_loading() && !posts.is_loading();
let any_error = profile.is_error() || stats.is_error() || posts.is_error();

let status = if any_error {
    "Some requests failed"
} else if all_loaded {
    "All data loaded"
} else {
    "Loading..."
};

View::text(format!("Status: {}", status))
```

## Fetching from APIs

Real-world HTTP example:

```rust
let user_data = cx.use_async(|| {
    // Using reqwest or similar
    match reqwest::blocking::get("https://api.example.com/user/123") {
        Ok(response) => match response.json::<User>() {
            Ok(user) => Ok(user),
            Err(e) => Err(format!("Parse error: {}", e)),
        },
        Err(e) => Err(format!("Network error: {}", e)),
    }
});
```

## Database queries

Loading from a database:

```rust
let records = cx.use_async(|| {
    match db::fetch_all_records() {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("Database error: {}", e)),
    }
});

match &records {
    Async::Loading => View::text("Querying database..."),
    Async::Ready(rows) => View::list().items(rows.clone()).build(),
    Async::Error(e) => View::text(format!("Query failed: {}", e)),
}
```

## File I/O

Loading configuration files:

```rust
let config = cx.use_async(|| {
    match std::fs::read_to_string("config.json") {
        Ok(contents) => match serde_json::from_str(&contents) {
            Ok(cfg) => Ok(cfg),
            Err(e) => Err(format!("Invalid JSON: {}", e)),
        },
        Err(e) => Err(format!("File error: {}", e)),
    }
});
```

## Dependent loading

If one load depends on another, use `effect!` to trigger the second:

```rust
let user_id = state!(cx, || None);
let user_profile = state!(cx, || Async::Loading);

// Load user list
let users = cx.use_async(|| Ok(fetch_users()));

// When a user is selected, load their profile
effect!(cx, user_id.get(), with!(user_profile => move |id| {
    if let Some(id) = id {
        // Trigger profile load
        let profile = load_profile(*id);
        user_profile.set(Async::Ready(profile));
    }
    || {}
}));
```

This is more complex. For simpler cases, load everything upfront.

## Refreshing data

To reload, you need to trigger a re-creation of the async task:

```rust
let refresh_key = state!(cx, || 0);

let data = cx.use_async({
    let key = refresh_key.get();
    move || {
        let _ = key;  // capture key to make closure depend on it
        fetch_data()
    }
});

// Button to refresh
View::button()
    .label("Refresh")
    .on_press(with!(refresh_key => move || {
        refresh_key.update(|k| *k += 1);  // triggers new async task
    }))
    .build()
```

Changing the closure's captured values causes `use_async` to restart.

**Note:** This pattern is a workaround - the dummy `refresh_key` is captured just to force the async task to re-run. A more ergonomic API for refreshing async data is planned for a future version.

## Async vs Streams

**Use use_async when:**
- One-time data fetch (API call, database query, file load)
- Clear start and end (loading → ready/error)
- User-triggered or component mount triggered

**Use use_stream when:**
- Continuous data updates (polling, tailing, monitoring)
- No clear "done" state
- Data keeps flowing over time

See [Streams](./streams.md) for continuous data.

## Performance tips

**Parallel is automatic** - Multiple `use_async` calls run in parallel without extra work.

**Blocking is fine** - Async tasks run in background threads, so blocking operations (network, disk, sleep) don't freeze the UI.

**Cancellation is automatic** - If the component unmounts, background threads are cleaned up.

**Don't overuse** - Each `use_async` is a thread. Dozens is fine, hundreds is excessive.

## Common patterns

**API with timeout:**
```rust
let data = cx.use_async(|| {
    let handle = std::thread::spawn(|| fetch_from_api());

    match handle.join_timeout(Duration::from_secs(10)) {
        Ok(Ok(data)) => Ok(data),
        Ok(Err(e)) => Err(format!("Request failed: {}", e)),
        Err(_) => Err("Request timed out".to_string()),
    }
});
```

**Retry logic:**
```rust
let data = cx.use_async(|| {
    for attempt in 1..=3 {
        match fetch_data() {
            Ok(data) => return Ok(data),
            Err(e) if attempt == 3 => return Err(format!("Failed after 3 attempts: {}", e)),
            Err(_) => std::thread::sleep(Duration::from_secs(1)),
        }
    }
    unreachable!()
});
```

**Graceful degradation:**
```rust
match &data {
    Async::Loading => render_skeleton(),  // placeholder UI
    Async::Ready(d) => render_full_ui(d),
    Async::Error(_) => render_cached_data(),  // fallback to stale data
}
```

## Tips

**Return Ok/Err explicitly** - The closure must return `Result<T, String>`.

**Errors are strings** - Convert your error types to strings: `Err(format!("...", e))`.

**Loading is the initial state** - Even before the thread starts, the state is `Async::Loading`.

**No retry mechanism** - If the task fails, it stays failed. Implement retry logic inside the closure.

**Pattern match or use helpers** - Either `match &async_data` or `.is_loading()` / `.is_error()`.

**Clone for rendering** - `Async<T>` derefs to `&T` when Ready, but pattern matching gives you references. Clone if needed for rendering.

**Async data is not reactive** - Unlike `use_state`, modifying the value inside `Async::Ready` doesn't trigger re-renders. Async tasks run once.

Next: [Tables](../widgets/tables.md)
