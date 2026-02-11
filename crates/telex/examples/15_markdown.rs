//! Example 15: Markdown Rendering
//!
//! Demonstrates the markdown rendering capabilities with a split view
//! showing raw markdown on the left and rendered output on the right.
//!
//! Run with: cargo run -p telex-tui --example 15_markdown

use crossterm::event::KeyCode;
use telex::prelude::*;
use telex::Color;

telex::require_api!(0, 2);

fn main() {
    telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
}

struct App;

const DEMO_MARKDOWN: &str = r#"# Markdown Demo

This is **bold** and *italic*.

## Code

Inline `code` and blocks:

```rust
fn main() {
    println!("Hello!");
}
```

## Lists

- First item
- Second item
  - Nested item

1. Step one
2. Step two

## Blockquote

> A wise quote
> spans lines.

---

*The end.*
"#;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let show_help = state!(cx, || false);

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );

        let rendered = telex::markdown::render(DEMO_MARKDOWN);

        View::vstack()
            .child(
                View::styled_text("Markdown Rendering Demo")
                    .color(Color::Cyan)
                    .bold()
                    .build(),
            )
            .child(
                View::boxed()
                    .flex(1)
                    .child(
                        View::split()
                            .horizontal()
                            .ratio(0.4)
                            .first(
                                View::vstack()
                                    .child(View::styled_text(" Source ").bold().build())
                                    .child(
                                        View::boxed()
                                            .flex(1)
                                            .border(true)
                                            .scroll(true)
                                            .child(View::text(DEMO_MARKDOWN))
                                            .build(),
                                    )
                                    .build(),
                            )
                            .second(
                                View::vstack()
                                    .child(View::styled_text(" Rendered ").bold().build())
                                    .child(
                                        View::boxed()
                                            .flex(1)
                                            .border(true)
                                            .scroll(true)
                                            .child(rendered)
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::styled_text("Tab: switch panes | ↑↓/jk: scroll | F1 help | Ctrl+Q: quit")
                    .dim()
                    .build(),
            )
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 15: Markdown")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Side-by-side markdown source and rendered"))
                            .child(View::text("• Full markdown syntax support"))
                            .child(View::text("• Scrollable panes for long content"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• telex::markdown::render() parses markdown"))
                            .child(View::text("• Returns a View tree with styled text"))
                            .child(View::text("• Code blocks, lists, quotes, headers"))
                            .child(View::text("• Split view for comparison"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Tab between source and rendered panes"))
                            .child(View::text("• Scroll with arrow keys or j/k"))
                            .child(View::text("• Compare source with rendered output"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 16_progress: progress bars"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
