# Tabs

Organize multiple screens into a tabbed interface with keyboard navigation.

```rust
let active = state!(cx, || 0);

View::tabs()
    .tab("Overview", overview_content)
    .tab("Settings", settings_content)
    .tab("About", about_content)
    .active(active.get())
    .on_change(with!(active => move |idx| active.set(idx)))
    .build()
```

Run with: `cargo run -p telex-tui --example 14_tabs`

## Basic tabs

Tabs display one of several screens based on which tab is active:

```rust
View::tabs()
    .tab("Home", View::text("Welcome home!"))
    .tab("Profile", View::text("Your profile"))
    .tab("Settings", View::text("App settings"))
    .build()
```

The first tab (`"Home"`) is active by default.

## Controlling active tab

Track which tab is visible with state:

```rust
let active_tab = state!(cx, || 0);  // 0 = first tab

View::tabs()
    .tab("Tab 1", content1)
    .tab("Tab 2", content2)
    .tab("Tab 3", content3)
    .active(active_tab.get())
    .on_change(with!(active_tab => move |idx: usize| {
        active_tab.set(idx);
    }))
    .build()
```

When the user switches tabs, `on_change` fires with the new index.

## Tab content

Each tab's content is a full `View`. You can put anything inside:

```rust
View::tabs()
    .tab("List", View::list().items(items).build())
    .tab("Table", View::table().rows(rows).build())
    .tab("Form",
        View::vstack()
            .child(View::text_input()...)
            .child(View::button()...)
            .build()
    )
    .active(active.get())
    .on_change(on_change)
    .build()
```

## Keyboard navigation

Tabs support multiple navigation methods:
- **←/→** - Previous/next tab
- **[** and **]** - Previous/next tab (alternative)
- **1**, **2**, **3**, ... - Jump to specific tab by number

All methods wrap around (last → first, first → last).

## Per-tab state

State persists even when tabs aren't visible:

```rust
let active_tab = state!(cx, || 0);

// These states persist across tab switches
let settings_notify = state!(cx, || true);
let settings_theme = state!(cx, || "dark".to_string());
let profile_name = state!(cx, || String::new());

View::tabs()
    .tab("Profile",
        View::text_input()
            .value(profile_name.get())
            .on_change(with!(profile_name => move |s| profile_name.set(s)))
            .build()
    )
    .tab("Settings",
        View::vstack()
            .child(View::checkbox()
                .label("Notifications")
                .checked(settings_notify.get())
                .on_toggle(with!(settings_notify => move |c| settings_notify.set(c)))
                .build())
            .build()
    )
    .active(active_tab.get())
    .on_change(with!(active_tab => move |idx| active_tab.set(idx)))
    .build()
```

When you switch to Settings and back to Profile, the input value is still there.

## Dynamic tab content

Rebuild tab content based on state:

```rust
let data = state!(cx, Vec::new);

View::tabs()
    .tab("Data", View::list().items(data.get()).build())
    .tab("Add",
        View::button()
            .label("Add Item")
            .on_press(with!(data => move || {
                data.update(|v| v.push("New".into()));
            }))
            .build()
    )
    .active(active.get())
    .on_change(on_change)
    .build()
```

Adding an item in the "Add" tab updates the list shown in the "Data" tab.

## Programmatic tab switching

Switch tabs from code:

```rust
// Switch to second tab
active_tab.set(1);

// Button that switches tabs
View::button()
    .label("Go to Settings")
    .on_press(with!(active_tab => move || active_tab.set(2)))
    .build()
```

## Tab bar styling

Tab labels appear in a row at the top. The active tab is highlighted.

The widget handles all styling automatically - you just provide labels.

## Number of tabs

You can have as many tabs as you want, but consider usability:
- **2-5 tabs** - Ideal, easy to navigate
- **6-8 tabs** - Acceptable, fits most terminals
- **9+ tabs** - Gets crowded, consider nested tabs or a different UI

## Empty tabs

You can have an empty tab:

```rust
.tab("Coming Soon", View::styled_text("This feature is under development").dim().build())
```

## Real-world example

Settings screen with multiple categories:

```rust
let active_tab = state!(cx, || 0);
let notifications = state!(cx, || true);
let dark_mode = state!(cx, || true);
let auto_save = state!(cx, || true);

View::tabs()
    .tab("Overview",
        View::vstack()
            .child(View::styled_text("Welcome!").bold().build())
            .child(View::text("Configure your app using the tabs above"))
            .build()
    )
    .tab("Settings",
        View::vstack()
            .child(View::checkbox()
                .label("Enable notifications")
                .checked(notifications.get())
                .on_toggle(with!(notifications => move |c| notifications.set(c)))
                .build())
            .child(View::checkbox()
                .label("Dark mode")
                .checked(dark_mode.get())
                .on_toggle(with!(dark_mode => move |c| dark_mode.set(c)))
                .build())
            .child(View::checkbox()
                .label("Auto-save")
                .checked(auto_save.get())
                .on_toggle(with!(auto_save => move |c| auto_save.set(c)))
                .build())
            .build()
    )
    .tab("About",
        View::vstack()
            .child(View::text("My App v1.0"))
            .child(View::text("Built with Telex"))
            .build()
    )
    .active(active_tab.get())
    .on_change(with!(active_tab => move |idx| active_tab.set(idx)))
    .build()
```

## Tabs in modals

Tabs work inside modals:

```rust
View::modal()
    .visible(show_settings.get())
    .title("Settings")
    .child(
        View::tabs()
            .tab("General", general_settings)
            .tab("Advanced", advanced_settings)
            .active(settings_tab.get())
            .on_change(on_tab_change)
            .build()
    )
    .build()
```

## Conditional tabs

Show different tabs based on state:

```rust
let is_admin = state!(cx, || false);

let mut tabs_view = View::tabs()
    .tab("Home", home_content)
    .tab("Profile", profile_content);

if is_admin.get() {
    tabs_view = tabs_view.tab("Admin", admin_content);
}

tabs_view
    .active(active.get())
    .on_change(on_change)
    .build()
```

Note: Be careful with conditional tabs and `active` index - ensure the index stays valid.

## Tips

**Active is zero-indexed** - First tab is 0, second is 1, etc.

**State persists** - All tabs render simultaneously (only one is visible). State in inactive tabs is preserved.

**Labels should be short** - Keep tab labels to 10-15 characters. Long labels crowd the tab bar.

**Don't nest tabs** - Tabs within tabs is confusing. Use a different navigation pattern.

**Consider order** - Put the most important/frequently used tab first.

**Number shortcuts** - Users can press `1`, `2`, `3` to jump to tabs. Design with this in mind.

**Wrap around** - Left arrow on first tab goes to last tab. Right arrow on last tab goes to first.

**Tab bar always visible** - The tab labels stay at the top while the content scrolls.

Next: [Forms](./forms.md)
