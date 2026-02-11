# Terminal Widget

> ⚠️ **Experimental Preview**
> Terminal widget is in early development with significant limitations. See below for details.

Interactive PTY terminal emulator for running shell commands and CLI applications within your TUI.

```rust
let terminal = cx.use_terminal();

// Spawn a shell on first render
if !terminal.is_started() {
    terminal.spawn("bash", &[], 80, 24)?;
}

View::terminal()
    .handle(terminal)
    .rows(24)
    .cols(80)
    .border(true)
    .title("Shell")
    .build()
```

Run with: `cargo run -p telex-tui --example 33_terminal`

## What Works

- ✅ PTY spawning with `portable-pty`
- ✅ ANSI escape sequence parsing (colors, styles, cursor movement)
- ✅ Full keyboard input (arrows, Ctrl+C, function keys, etc.)
- ✅ Running interactive programs (bash, vim, htop, etc.)
- ✅ Border with title
- ✅ Focus management

## Known Limitations

- ❌ **No scrollback buffer** - Only visible area is stored
- ❌ **No terminal resize** - Size is set at spawn time
- ❌ **No copy/paste**
- ❌ **No mouse input**
- ❌ **Exit callbacks not fully wired**

## Keyboard Shortcuts

- **Ctrl+Shift+[** - Escape terminal focus (return to TUI navigation)
- **Tab** - Navigate to next widget
- **All other keys** - Sent to the terminal PTY

## Use Cases

```rust
// Shell access
terminal.spawn("bash", &[], 80, 24);

// Text editor
terminal.spawn("vim", &["config.toml"], 80, 24);

// System monitor
terminal.spawn("htop", &[], 80, 24);

// AI agent CLI
terminal.spawn("python", &["agent.py"], 80, 24);
```

## Architecture

The terminal widget uses:
- **portable-pty** - Cross-platform PTY spawning (Unix/Windows/macOS)
- **vte** - ANSI escape sequence parser (industry standard, used by Alacritty)
- **Background threads** - Non-blocking PTY I/O via mpsc channels
- **TerminalBuffer** - 2D cell grid with cursor tracking

## Recursive Telex

You can run Telex inside Telex:
```bash
# Inside a terminal widget:
cargo run -p telex-tui --example 33_terminal
```

This enables visual agent orchestration - multiple terminal widgets running AI agents with visible, interactive data flow.

## Future Roadmap

**Phase 2 (planned):**
- Scrollback buffer with configurable size
- Terminal resize support
- Copy/paste with mouse selection
- Mouse input passthrough
- Proper cleanup and exit code handling
