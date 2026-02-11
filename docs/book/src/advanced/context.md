# Context

Share data across components without prop drilling.

```rust
// Provide (parent)
cx.provide_context(AppConfig {
    theme: "dark",
    api_url: "https://api.example.com",
});

// Consume (anywhere below)
let config = cx.use_context::<AppConfig>();
```

Run with: `cargo run -p telex-tui --example 25_context`

## The Problem: Prop Drilling

Passing data through many layers is tedious:

```rust
fn app() -> View {
    let theme = "dark";
    toolbar(theme)  // pass it down
}

fn toolbar(theme: &str) -> View {
    menu(theme)  // pass it down again
}

fn menu(theme: &str) -> View {
    menu_item(theme)  // and again...
}

fn menu_item(theme: &str) -> View {
    // Finally use it!
}
```

This is **prop drilling** - threading data through components that don't need it, just to get it deeper.

## The Solution: Context

Context lets you provide data at the top and consume it anywhere below:

```rust
fn app(cx: Scope) -> View {
    cx.provide_context("dark");

    toolbar(&cx)  // no props needed!
}

fn menu_item(cx: &Scope) -> View {
    let theme = cx.use_context::<&str>();
    // Use it directly!
}
```

No intermediate functions need to know about the theme.

## Providing Context

Use `provide_context` to make data available to descendants:

```rust
#[derive(Clone)]
struct Config {
    api_url: String,
    timeout: u64,
}

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        cx.provide_context(Config {
            api_url: "https://api.example.com".to_string(),
            timeout: 30,
        });

        // All descendants can access Config
        View::vstack()
            .child(header(&cx))
            .child(content(&cx))
            .build()
    }
}
```

The data must implement `Clone + 'static`.

## Consuming Context

Use `use_context` to retrieve data from ancestors:

```rust
fn header(cx: &Scope) -> View {
    let config = cx.use_context::<Config>();

    match config {
        Some(cfg) => View::text(&cfg.api_url),
        None => View::text("No config"),
    }
}
```

`use_context` returns `Option<T>`:
- `Some(data)` if an ancestor provided it
- `None` if no provider exists

## Multiple Context Types

You can provide multiple types:

```rust
#[derive(Clone)]
struct Theme {
    primary: Color,
    accent: Color,
}

#[derive(Clone)]
struct User {
    name: String,
    role: String,
}

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        cx.provide_context(Theme {
            primary: Color::Blue,
            accent: Color::Cyan,
        });

        cx.provide_context(User {
            name: "Alice".to_string(),
            role: "Admin".to_string(),
        });

        // Both available to descendants
    }
}

fn sidebar(cx: &Scope) -> View {
    let theme = cx.use_context::<Theme>();
    let user = cx.use_context::<User>();

    // Use both
}
```

Each type is stored separately by `TypeId`.

## Real-World Example: Theme System

A complete theme implementation:

```rust
#[derive(Clone, Copy)]
enum AppTheme {
    Light,
    Dark,
    HighContrast,
}

impl AppTheme {
    fn background(&self) -> Color {
        match self {
            AppTheme::Light => Color::White,
            AppTheme::Dark => Color::Black,
            AppTheme::HighContrast => Color::Black,
        }
    }

    fn text(&self) -> Color {
        match self {
            AppTheme::Light => Color::Black,
            AppTheme::Dark => Color::White,
            AppTheme::HighContrast => Color::Yellow,
        }
    }
}

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let theme = state!(cx, || AppTheme::Dark);

        // Provide current theme
        cx.provide_context(theme.get());

        View::vstack()
            .child(theme_picker(&cx, theme))
            .child(content(&cx))
            .build()
    }
}

fn content(cx: &Scope) -> View {
    let theme = cx.use_context::<AppTheme>()
        .unwrap_or(AppTheme::Dark);

    View::boxed()
        .background(theme.background())
        .child(
            View::styled_text("Content")
                .color(theme.text())
                .build()
        )
        .build()
}
```

Change the theme state, and all consumers update automatically.

## Dynamic Context

Context can change over time by providing state values:

```rust
impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let user = state!(cx, || User::guest());

        // Provide current user (clones the value)
        cx.provide_context(user.get());

        // When user.set() is called, next render provides new value
    }
}
```

Each render provides a fresh snapshot of the state.

## Nested Providers

Child providers override parent providers for the same type:

```rust
fn app(cx: Scope) -> View {
    cx.provide_context("dark");  // App-level theme

    View::vstack()
        .child(sidebar(&cx))     // uses "dark"
        .child(special_panel(&cx))  // overrides to "light"
        .build()
}

fn special_panel(cx: &Scope) -> View {
    cx.provide_context("light");  // Override for this subtree

    // All descendants see "light", not "dark"
    panel_content(cx)
}
```

The innermost provider wins.

## Context vs Shared State

Both solve similar problems but have different trade-offs:

**Context (`provide_context` / `use_context`):**
- ✅ Provider/consumer pattern (explicit hierarchy)
- ✅ Scoped to subtree (can override)
- ✅ Returns `Option<T>` (safe if missing)
- ✅ Good for configuration, theming, user sessions
- ❌ Runtime lookup (slight overhead)
- ❌ Immutable snapshots (provide fresh values each render)

**Shared State (`use_state_keyed`):**
- ✅ Compile-time keys (type-safe)
- ✅ Mutable via `State<T>` API
- ✅ Good for frequently changing data
- ❌ Global scope (no override)
- ❌ Must use same key everywhere
- ❌ No hierarchy

**When to use which:**
- **Context:** Themes, user info, app config, dependency injection
- **Shared State:** Global counters, selected items, UI state

See [Shared State](./shared-state.md) for more on `use_state_keyed`.

## Common Patterns

**App configuration:**
```rust
#[derive(Clone)]
struct AppConfig {
    name: String,
    version: String,
    api_base: String,
    features: Vec<String>,
}

// Provide once at app root
cx.provide_context(AppConfig {
    name: "My App".to_string(),
    version: "1.0.0".to_string(),
    api_base: "https://api.example.com".to_string(),
    features: vec!["dark_mode".to_string(), "exports".to_string()],
});

// Use anywhere
fn about_dialog(cx: &Scope) -> View {
    let config = cx.use_context::<AppConfig>().unwrap();
    View::text(format!("{} v{}", config.name, config.version))
}
```

**User session:**
```rust
#[derive(Clone)]
struct Session {
    user_id: Option<u64>,
    token: Option<String>,
    permissions: Vec<String>,
}

impl Session {
    fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    fn can(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }
}

// Provide session
let session = state!(cx, || Session::default());
cx.provide_context(session.get());

// Check permissions anywhere
fn admin_panel(cx: &Scope) -> View {
    let session = cx.use_context::<Session>();

    if !session.map(|s| s.can("admin")).unwrap_or(false) {
        return View::text("Access denied");
    }

    // Admin UI
}
```

**Feature flags:**
```rust
#[derive(Clone)]
struct Features {
    new_ui: bool,
    beta_features: bool,
    debug_mode: bool,
}

// Provide
cx.provide_context(Features {
    new_ui: true,
    beta_features: false,
    debug_mode: cfg!(debug_assertions),
});

// Use
fn experimental_widget(cx: &Scope) -> View {
    let features = cx.use_context::<Features>();

    if !features.map(|f| f.beta_features).unwrap_or(false) {
        return View::empty();
    }

    // Beta UI
}
```

## Default Values

Handle missing context with defaults:

```rust
fn my_component(cx: &Scope) -> View {
    let theme = cx.use_context::<Theme>()
        .unwrap_or(Theme::default());

    // Always has a theme, even if not provided
}
```

Or use a helper function:

```rust
fn get_theme(cx: &Scope) -> Theme {
    cx.use_context::<Theme>()
        .unwrap_or(Theme::Dark)
}
```

## Type Safety

Context is type-safe - you can't accidentally get the wrong type:

```rust
#[derive(Clone)]
struct UserName(String);

#[derive(Clone)]
struct AppName(String);

cx.provide_context(UserName("Alice".to_string()));

// This gets None (different type)
let app_name = cx.use_context::<AppName>();
```

Newtype wrappers give you distinct context slots.

## Dependency Injection

Use context for dependency injection:

```rust
trait Database: Clone {
    fn query(&self, sql: &str) -> Vec<Row>;
}

#[derive(Clone)]
struct PostgresDB { /* ... */ }

impl Database for PostgresDB {
    fn query(&self, sql: &str) -> Vec<Row> {
        // Implementation
    }
}

// Provide
cx.provide_context(PostgresDB::new());

// Inject
fn data_table<D: Database + 'static>(cx: &Scope) -> View {
    let db = cx.use_context::<D>().expect("Database not provided");

    let rows = db.query("SELECT * FROM users");
    // Render rows
}
```

Swap implementations for testing or different environments.

## Debugging

If `use_context` returns `None`:
1. Check that an ancestor calls `provide_context` for that type
2. Verify the type matches exactly (including generics)
3. Ensure the provider is rendered before the consumer
4. Check that context is provided in the same scope or an ancestor

Add debug output:

```rust
let config = cx.use_context::<Config>();
if config.is_none() {
    eprintln!("Warning: Config context not found!");
}
```

## Performance

Context has minimal overhead:
- Providing: O(1) insertion into hash map
- Consuming: O(1) hash map lookup

The clone on provide is the main cost - keep context types lightweight or use `Rc` for large data:

```rust
use std::rc::Rc;

#[derive(Clone)]
struct LargeConfig(Rc<ExpensiveData>);

// Clone is cheap (just increments ref count)
cx.provide_context(LargeConfig(data));
```

## Limitations

**No reactivity:** Context values are snapshots. Changing the original doesn't update consumers until the next render when you call `provide_context` again.

```rust
let config = state!(cx, || Config::default());

// Update config
config.update(|c| c.timeout = 60);

// Must re-provide for descendants to see new value
cx.provide_context(config.get());
```

This is why dynamic context works by providing `state.get()` - each render provides a fresh snapshot.

**No subscriptions:** Consumers don't subscribe to updates. They read during render. If context changes, the component must re-render for them to see it.

## Tips

**Provide early** - Call `provide_context` near the top of your render function, before returning the view tree.

**Small types** - Keep context types small. Large data should be in `Rc` or `Arc`.

**Newtype for clarity** - Wrap primitives: `struct UserId(u64)` instead of bare `u64`.

**Document requirements** - If a component needs context, document which types it expects.

**Provide defaults** - Always unwrap with a sensible default: `.unwrap_or_default()`.

**One provider per tree** - Don't provide the same type multiple times in a single component. Override in children if needed.

**Clone is mandatory** - Context types must be `Clone`. If your type can't clone, wrap it in `Rc`.

## Comparison Table

| Feature | Context | Shared State | Props |
|---------|---------|--------------|-------|
| Scope | Subtree | Global | Direct child |
| Mutability | Immutable snapshots | Mutable | Immutable |
| Type safety | Runtime (`Option<T>`) | Compile-time | Compile-time |
| Override | Yes (nested providers) | No | Yes (pass different value) |
| Best for | Config, theme, session | UI state, selections | Local data |

Next: [Troubleshooting](../reference/troubleshooting.md)
