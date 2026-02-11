# Shared State

Multiple components sharing the same state via explicit keys.

```rust
struct SharedCounterKey;

// Both get the SAME state
let count_a = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);
let count_b = cx.use_state_keyed::<SharedCounterKey, _>(|| 0);

count_a.update(|n| *n += 1);  // Both see the update!
```

Run with: `cargo run -p telex-tui --example 28_shared_state`

## The Concept

In [Keyed State](./keyed-state.md), we saw that `state!` creates **unique keys** for each call site, giving you independent state. Shared state is the opposite: **same key = same state**.

By using `use_state_keyed` with an explicit key type, multiple parts of your code can access the same underlying value.

## Defining a Key

Create a zero-sized type to use as a key:

```rust
struct MySharedKey;
```

That's it. The type name becomes the key. Any code that uses `MySharedKey` will share state.

## Using the Key

Access shared state with `use_state_keyed`:

```rust
struct CounterKey;

fn render(&self, cx: Scope) -> View {
    // First access - creates the state
    let count = cx.use_state_keyed::<CounterKey, _>(|| 0);

    // Later, elsewhere - gets the SAME state
    let count2 = cx.use_state_keyed::<CounterKey, _>(|| 0);

    // count and count2 point to the same value
    count.get() == count2.get()  // always true
}
```

The initializer `|| 0` only runs once, when the key is first encountered. Subsequent calls retrieve the existing state.

## Multiple Components Sharing State

Different parts of your UI can share state:

```rust
struct SelectedTabKey;

fn tab_bar(cx: Scope) -> View {
    let selected = cx.use_state_keyed::<SelectedTabKey, _>(|| 0);

    // Render tab buttons that modify selected
}

fn tab_content(cx: Scope) -> View {
    let selected = cx.use_state_keyed::<SelectedTabKey, _>(|| 0);

    // Render content based on selected tab
}

// Both functions access the same selected tab index
```

When the tab bar changes the selection, the content updates automatically.

## Shared State Across Renders

State persists across renders, so all access points see the same value:

```rust
struct AppModeKey;

fn render(&self, cx: Scope) -> View {
    let mode = cx.use_state_keyed::<AppModeKey, _>(|| Mode::View);

    View::vstack()
        .child(toolbar(cx))       // reads mode
        .child(main_content(cx))  // reads mode
        .child(status_bar(cx))    // reads mode
        .build()
}

fn toolbar(cx: Scope) -> View {
    let mode = cx.use_state_keyed::<AppModeKey, _>(|| Mode::View);
    // mode reflects current app mode set anywhere else
}
```

## Real-World Example: Theme Switcher

A global theme setting:

```rust
struct ThemeKey;

fn app(cx: Scope) -> View {
    View::vstack()
        .child(header(cx))
        .child(content(cx))
        .child(settings_panel(cx))
        .build()
}

fn header(cx: Scope) -> View {
    let theme = cx.use_state_keyed::<ThemeKey, _>(|| "dark");

    let bg_color = match theme.get().as_str() {
        "dark" => Color::DarkGrey,
        "light" => Color::White,
        _ => Color::Blue,
    };

    View::boxed()
        .background(bg_color)
        .child(View::text("Header"))
        .build()
}

fn settings_panel(cx: Scope) -> View {
    let theme = cx.use_state_keyed::<ThemeKey, _>(|| "dark");

    View::button()
        .label("Toggle Theme")
        .on_press(with!(theme => move || {
            theme.update(|t| {
                *t = if *t == "dark" { "light".to_string() } else { "dark".to_string() }
            });
        }))
        .build()
}
```

Clicking the button in settings updates the theme everywhere.

## Multiple Shared States

You can have as many shared states as you need:

```rust
struct UserNameKey;
struct UserEmailKey;
struct UserAvatarKey;

fn profile_editor(cx: Scope) -> View {
    let name = cx.use_state_keyed::<UserNameKey, _>(String::new);
    let email = cx.use_state_keyed::<UserEmailKey, _>(String::new);
    let avatar = cx.use_state_keyed::<UserAvatarKey, _>(|| "default.png");

    // Edit UI
}

fn profile_display(cx: Scope) -> View {
    let name = cx.use_state_keyed::<UserNameKey, _>(String::new);
    let email = cx.use_state_keyed::<UserEmailKey, _>(String::new);
    let avatar = cx.use_state_keyed::<UserAvatarKey, _>(|| "default.png");

    // Display UI - sees changes from editor
}
```

Each key is independent - changing name doesn't affect email.

## Shared State in Loops

Shared keys work inside loops:

```rust
struct ListSelectionKey;

for item in items.get() {
    let selected = cx.use_state_keyed::<ListSelectionKey, _>(|| 0);

    // All iterations see the same selection
    let is_selected = selected.get() == item.id;
}
```

This is useful for coordinating behavior across list items.

## When to Use Shared State

**Use shared state when:**
- Multiple UI components need the same data
- You want global app-level state (theme, user session, etc.)
- Avoiding prop drilling through many layers
- Coordinating state across distant components

**Use regular state when:**
- State is local to one component
- You want isolation and independence
- State doesn't need to be accessed elsewhere

## Shared State vs Context

Both solve similar problems - sharing data across components. The difference:

**Shared State (`use_state_keyed`):**
- Explicit key types
- Compile-time enforcement
- State is mutable via `State<T>` API
- Good for frequently changing data

**Context (`use_context`):**
- Provider/consumer pattern
- Runtime lookup (returns `Option<T>`)
- Data is typically immutable
- Good for configuration and DI

See [Context](./context.md) for details.

## Type Safety

Keys are type-safe. Different keys can't accidentally collide:

```rust
struct CountKey;
struct NameKey;

let count = cx.use_state_keyed::<CountKey, _>(|| 0i32);
let name = cx.use_state_keyed::<NameKey, _>(String::new);

// These are completely independent
// No runtime conflicts possible
```

## Initialization

The initializer runs **only once**, when the key is first used:

```rust
struct ExpensiveKey;

let data = cx.use_state_keyed::<ExpensiveKey, _>(|| {
    println!("Initializing!");  // Prints once
    load_from_disk()
});

let data2 = cx.use_state_keyed::<ExpensiveKey, _>(|| {
    println!("Initializing!");  // Never prints
    load_from_disk()             // Never runs
});
```

## Sharing with Explicit Keys

You can parameterize keys for fine-grained sharing:

```rust
// Instead of a single shared counter:
struct CounterKey;

// Use multiple keys for independent counters:
struct CounterAKey;
struct CounterBKey;
struct CounterCKey;

let a = cx.use_state_keyed::<CounterAKey, _>(|| 0);
let b = cx.use_state_keyed::<CounterBKey, _>(|| 0);
let c = cx.use_state_keyed::<CounterCKey, _>(|| 0);
```

Or use a generic key with phantom data:

```rust
use std::marker::PhantomData;

struct CounterKey<T>(PhantomData<T>);

struct A;
struct B;

let counter_a = cx.use_state_keyed::<CounterKey<A>, _>(|| 0);
let counter_b = cx.use_state_keyed::<CounterKey<B>, _>(|| 0);
```

## Common Patterns

**Global loading state:**
```rust
struct LoadingKey;

fn anywhere(cx: Scope) {
    let loading = cx.use_state_keyed::<LoadingKey, _>(|| false);
    loading.set(true);  // Shows loading spinner globally
}
```

**Sidebar open/closed:**
```rust
struct SidebarOpenKey;

fn sidebar_toggle(cx: Scope) -> View {
    let open = cx.use_state_keyed::<SidebarOpenKey, _>(|| true);

    View::button()
        .label(if open.get() { "Close" } else { "Open" })
        .on_press(with!(open => move || open.update(|b| *b = !*b)))
        .build()
}

fn sidebar(cx: Scope) -> View {
    let open = cx.use_state_keyed::<SidebarOpenKey, _>(|| true);

    if !open.get() {
        return View::empty();
    }

    // Sidebar content
}
```

**Selected item ID:**
```rust
struct SelectedItemKey;

fn item_list(cx: Scope, items: Vec<Item>) -> View {
    let selected = cx.use_state_keyed::<SelectedItemKey, _>(|| None::<usize>);

    View::list()
        .items(items.iter().map(|i| i.name.clone()).collect())
        .selected(selected.get().unwrap_or(0))
        .on_select(with!(selected => move |idx| selected.set(Some(idx))))
        .build()
}

fn item_details(cx: Scope, items: Vec<Item>) -> View {
    let selected = cx.use_state_keyed::<SelectedItemKey, _>(|| None::<usize>);

    match selected.get() {
        Some(idx) => View::text(&items[idx].description),
        None => View::text("No item selected"),
    }
}
```

## Debugging

If you see unexpected sharing, check that your key types are distinct:

```rust
// BAD - Same key name in different modules
mod panel_a {
    struct SelectedKey;
}

mod panel_b {
    struct SelectedKey;  // Different type, but if imported wrong...
}

// GOOD - Unique names
struct PanelASelectedKey;
struct PanelBSelectedKey;
```

## Performance

Shared state has the same performance as regular state:
- O(1) key lookup (hash map)
- O(1) read/write via `State<T>`

The only difference is the key is provided explicitly instead of generated.

## Limitations

**No scoping:** Shared state is global to the component tree. You can't have "scoped" shared state that only exists in part of the tree.

**No hierarchies:** Keys are flat - you can't nest or inherit keys.

**Manual coordination:** You're responsible for ensuring the right components use the right keys.

For scoped sharing, use [Context](./context.md) instead.

## Tips

**Name keys clearly** - `struct ThemeKey` is better than `struct Key1`.

**One key per concept** - Don't reuse keys for unrelated data.

**Document key purpose** - Add comments explaining what each key is for:
```rust
/// Shared state for currently selected tab index
struct SelectedTabKey;
```

**Prefer context for config** - Use shared state for mutable data, context for immutable config.

**Group related keys** - Put keys in a module:
```rust
mod app_state {
    pub struct ThemeKey;
    pub struct UserKey;
    pub struct LoadingKey;
}
```

Next: [Context](./context.md)
