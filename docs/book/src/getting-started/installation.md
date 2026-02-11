# Installation

## Requirements

- Rust 1.70 or later
- A terminal emulator (any modern terminal works)

For canvas/image features (optional):
- Kitty, Ghostty, or WezTerm (Kitty graphics protocol support)

## Add to your project

```bash
cargo add telex-tui
```

Or add to `Cargo.toml`:

```toml
[dependencies]
telex-tui = "0.1"
```

## Verify installation

Create a minimal app:

```rust
use telex::prelude::*;

struct App;

impl Component for App {
    fn render(&self, _cx: Scope) -> View {
        View::text("It works!")
    }
}

fn main() {
    telex::run(App).unwrap();
}
```

Run it:

```bash
cargo run
```

You should see "It works!" in your terminal. Press `Ctrl+Q` to quit.

## Clone the repo (optional)

To run examples and explore the source:

```bash
git clone https://github.com/telex-tui/telex-tui
cd telex-tui
cargo run -p telex-tui --example 01_hello_world
```

Next: [Hello World](./hello-world.md)
