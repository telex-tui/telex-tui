# Styling

Telex styling works in two layers:

1. **Themes** - Pick one, and all widgets use it automatically. Buttons, inputs, selections, borders - they all pull from the theme's semantic colors (`primary`, `error`, `success`, etc.).

2. **Inline overrides** - For when you need something specific: an error message in red, a heading in bold, a muted hint.

Most of the time, you just pick a theme and go. Inline styling is for exceptions.

---

## Inline styling

Override colors and attributes on specific text:

```rust
View::styled_text("Important!")
    .bold()
    .color(Color::Red)
    .build()
```

Run with: `cargo run -p telex-tui --example 03_theme_switcher`

## Text attributes

```rust
View::styled_text("Hello")
    .bold()
    .dim()
    .italic()
    .underline()
    .color(Color::Cyan)
    .build()
```

Chain style methods, then `.build()`.

## Colors

```rust
use telex::Color;

Color::Red
Color::Green
Color::Blue
Color::Yellow
Color::Cyan
Color::Magenta
Color::White
Color::DarkGrey
Color::Rgb { r: 255, g: 128, b: 0 }  // custom RGB
```

## Themes

Themes define semantic colors that widgets use automatically - you don't style each button, the theme handles it.

```rust
telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
```

Available themes:

| Theme | Style |
|-------|-------|
| `Theme::dark()` | Dark background (default) |
| `Theme::light()` | Light background |
| `Theme::nord()` | Nord |
| `Theme::dracula()` | Dracula |
| `Theme::gruvbox_dark()` | Gruvbox |
| `Theme::solarized_dark()` | Solarized |
| `Theme::tokyo_night()` | Tokyo Night |
| `Theme::monokai()` | Monokai |
| `Theme::catppuccin_mocha()` | Catppuccin (dark) |
| `Theme::catppuccin_latte()` | Catppuccin (light) |
| `Theme::rose_pine()` | Rosé Pine |
| `Theme::hax0r_green()` | Monochrome green |
| `Theme::hax0r_blue()` | Monochrome cyan |

Switch themes at runtime:

```rust
use telex::theme::{set_theme, Theme};

set_theme(Theme::dracula());
```

## Boxed containers

Add borders and padding with `View::boxed()`:

```rust
View::boxed()
    .border(true)
    .padding(1)
    .child(View::text("Boxed content"))
    .build()
```

Next: [Layouts](../building-uis/layouts.md)
