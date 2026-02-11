# telex - tui

<p align="center">
  <img src="https://raw.githubusercontent.com/telex-tui/telex-tui/main/assets/telex-tui.png" width="200" alt="Logo">
</p>

A reactive TUI framework for Rust, inspired by React's component model.

Declare your UI, manage state with hooks, and let Telex handle the rendering. If you've used React, this will feel familiar. If you haven't, it's a clean way to build terminal apps without wrestling raw cells.

---

## Quick Start

### From the repo

Clone, browse, run:

```bash
git clone https://github.com/telex-tui/telex-tui/
cd telex-tui

# Interactive example browser — best way to get a feel for things
./run-examples.sh

# Or run one directly
cargo run -p telex-tui --example 02_counter
```

> Press **F1** in any example to see what it demonstrates.

There are 33 numbered examples covering everything from hello world to an embedded terminal emulator. See the [Examples Guide](docs/examples.md) for the full list.

### As a dependency

```bash
cargo new my-app && cd my-app
cargo add telex-tui
```

```rust
use telex::prelude::*;

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let count = state!(cx, || 0);

        View::vstack()
            .child(View::text(format!("Count: {}", count.get())))
            .child(
                View::button()
                    .label("+")
                    .on_press(with!(count => move || count.update(|n| *n += 1)))
                    .build()
            )
            .build()
    }
}

fn main() {
    telex::run(App).unwrap();
}
```

---

## What's in the box

**State & hooks** — `state!` macro for order-independent reactive state. `effect!` and `effect_once!` for side effects. `use_async` and `use_stream` for background work. All single-threaded — no Arc, no Send bounds.

**Layout** — VStack, HStack, Box with flex, min/max constraints, and percentage sizing. Split panes for resizable panels. Tabs, modals, menus.

**Widgets** — Text, TextInput, TextArea, Button, Checkbox, RadioGroup, List, Table, Tree, Tabs, ProgressBar, StatusBar, MenuBar, Modal, Form, Toast, Markdown, Split, and more. Builder pattern or `view!` macro — your choice.

**Focus management** — Tab navigation, modal focus containment, cursor navigation, initial focus. Focus indicators appear only when the user starts navigating.

**Theming** — 14 built-in themes: catppuccin, dracula, gruvbox, nord, tokyo night, and others. One line to switch.

**Unicode** — Proper grapheme cluster handling. Emoji, CJK, combining characters all render at correct widths.

**Streaming** — Built-in `use_text_stream` for token-by-token rendering. The kind of thing you need for LLM output.

---

## Example: AI Chat App

The repo includes a multi-provider chat app in `examples/chat/` — a real application built with Telex that supports Anthropic, OpenAI, Gemini, and Ollama:

```bash
export GEMINI_API_KEY="..."   # or ANTHROPIC_API_KEY, OPENAI_API_KEY
cargo run -p chat
```

No API key? It falls back to mock responses so you can still see the UI. See the [chat example](examples/chat/) for full configuration.

---

## Experimental Features

These work, but expect rough edges:

- **Terminal widget** — Run bash, vim, htop inside your TUI ([example 33](crates/telex/examples/33_terminal.rs))
- **Canvas** — Pixel-level drawing via Kitty graphics protocol ([example 29](crates/telex/examples/29_canvas.rs))
- **Image** — PNG/JPEG/GIF display via Kitty protocol ([example 30](crates/telex/examples/30_image.rs))

Canvas and Image require a Kitty-compatible terminal (Kitty, Ghostty, WezTerm).

---

## Documentation

| | |
|---|---|
| [**The Telex Book**](https://telex-tui.github.io/telex-tui/) | Guide from hello world to real applications |
| [Examples](docs/examples.md) | All 33 examples with explanations |
| [Architecture](docs/architecture.md) | Design decisions and trade-offs |
| `cargo doc -p telex-tui --open` | API reference |

---

## Project Structure

```
telex-tui/
├── crates/
│   ├── telex/           # Core framework + 33 examples
│   └── telex-macro/     # view!, state!, effect!, with! proc macros
├── examples/
│   ├── chat/            # AI chat app (multi-provider)
│   ├── layout-playground/
│   ├── layout-showcase/
│   └── perf-test/
└── docs/                # Book, architecture, design docs
```

---

## Contributing

Issues and PRs welcome. If you hit a rendering bug, the test harness makes it straightforward to reproduce:

```rust
use telex::testing::TestApp;

let mut app = TestApp::new(|_cx: Scope| {
    View::text("content that renders wrong")
}).with_size(60, 10);

app.assert_visible("expected");
```

```bash
cargo test -p telex-tui
```

---

## License

MIT — see [LICENSE](LICENSE).

Copyright (c) 2025 Mark Branion
