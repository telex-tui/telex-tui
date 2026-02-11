# Deep Dive: Data Flow

[← Back to main](../architecture.md)

---

Understanding how data flows through Telex reveals the elegant simplicity beneath the API. This document traces the journey from user code to terminal output and back.

## The Main Loop

Everything starts with `telex::run()`:

```rust
pub fn run<C: Component>(root: C) -> Result<()> {
    // Setup
    let mut terminal = Terminal::new()?;
    let mut focus = FocusManager::new();
    let storage = Rc::new(StateStorage::new());

    // Main loop
    loop {
        // 1. Render phase
        let cx = Scope::with_storage(Rc::clone(&storage));
        let view = root.render(cx);

        // 2. Focus collection
        focus.collect_focusables(&view);

        // 3. Draw phase
        terminal.draw(&view, focus.focus_index())?;

        // 4. Event handling
        if let Some(event) = terminal.poll_event()? {
            match event {
                // Handle keys...
            }
        }
    }
}
```

Let's trace each phase in detail.

## Phase 1: Render

### Creating the Scope

```rust
let cx = Scope::with_storage(Rc::clone(&storage));
```

What happens:
1. Clone the `Rc<StateStorage>` (cheap—just increment reference count)
2. Create a `Scope` pointing to that storage
3. Reset the storage's hook index to 0

```rust
impl Scope {
    pub fn with_storage(storage: Rc<StateStorage>) -> Self {
        storage.reset_index();  // Start hooks from beginning
        Self { storage }
    }
}
```

### Calling the Component

```rust
let view = root.render(cx);
```

The component is a closure:

```rust
|cx: Scope| {
    let count = cx.use_state(|| 0);
    view! { <Text>{count.get()}</Text> }
}
```

### Hook Execution: First Render

```rust
let count = cx.use_state(|| 0);
```

First time through:
```
StateStorage {
    states: [],      // Empty
    index: 0
}

// use_state called
// index (0) >= states.len() (0), so create new state

let state = State::new(init());  // State { value: 0 }
states.push(Rc::new(state.clone()));

// Return handle
return state;

StateStorage {
    states: [State<i32>],  // Now has one entry
    index: 1               // Incremented
}
```

### Hook Execution: Re-render

```rust
let count = cx.use_state(|| 0);  // Same call, second render
```

Second time through:
```
StateStorage {
    states: [State<i32>],  // Already has state
    index: 0               // Reset by with_storage
}

// use_state called
// index (0) < states.len() (1), so retrieve existing

let state = states[0].downcast::<State<i32>>();

// Return existing handle (init closure ignored)
return state;

StateStorage {
    states: [State<i32>],  // Unchanged
    index: 1               // Incremented
}
```

### Building the View Tree

The component returns a View:

```rust
view! {
    <VStack>
        <Text>{count.get()}</Text>
        <Button on_press={move || c.update(|n| *n + 1)}>"+"</Button>
    </VStack>
}
```

This expands to:

```rust
View::vstack()
    .child(View::text(format!("{}", count.get())))
    .child(
        View::button()
            .on_press(move || c.update(|n| *n + 1))
            .label("+")
            .build()
    )
    .build()
```

Result:
```
View::VStack(VStackNode {
    children: [
        View::Text(TextNode { content: "0" }),
        View::Button(ButtonNode {
            label: "+",
            on_press: Some(Rc<dyn Fn()>)
        })
    ]
})
```

## Phase 2: Focus Collection

```rust
focus.collect_focusables(&view);
```

Walk the tree, collect focusable elements:

```rust
fn collect_recursive(&mut self, view: &View) {
    match view {
        View::Button(btn) => {
            self.focusables.push(btn.on_press.clone());
        }
        View::VStack(node) => {
            for child in &node.children {
                self.collect_recursive(child);
            }
        }
        // ... other variants
    }
}
```

For our example:
```
focusables: [
    Some(Rc<dyn Fn()>)  // The button's on_press
]
focus_index: 0  // First (only) button is focused
```

## Phase 3: Draw

```rust
terminal.draw(&view, focus.focus_index())?;
```

### View to Buffer

```rust
pub fn draw(&mut self, view: &View, focus_index: usize) -> io::Result<()> {
    self.buffer.clear();

    let area = self.buffer.rect();
    let mut ctx = RenderContext::new(focus_index);
    render_view(&mut self.buffer, view, area, &mut ctx);
    // ...
}
```

The render function walks the View tree:

```rust
pub fn render_view(buffer: &mut Buffer, view: &View, area: Rect, ctx: &mut RenderContext) {
    match view {
        View::Text(node) => {
            buffer.write_str(area.x, area.y, &node.content, fg, bg);
        }
        View::VStack(node) => {
            // Divide area among children
            for (i, child) in node.children.iter().enumerate() {
                let child_area = calculate_child_area(area, i, node.children.len());
                render_view(buffer, child, child_area, ctx);
            }
        }
        View::Button(node) => {
            let is_focused = ctx.is_next_focused();
            let (fg, bg) = if is_focused {
                (Color::Black, Color::White)  // Highlighted
            } else {
                (Color::Reset, Color::Reset)
            };
            buffer.write_str(area.x, area.y, &format!("[ {} ]", node.label), fg, bg);
        }
        // ...
    }
}
```

### Buffer State

After rendering:
```
Buffer (simplified):
Row 0: "0                    "  // Text content
Row 1: "[>+<]                "  // Button (focused, highlighted)
Row 2: "                     "
...
```

### Diffing

```rust
self.flush_diff()?;
```

Compare current buffer with previous:

```rust
fn flush_diff(&mut self) -> io::Result<()> {
    let changes = self.buffer.diff(&self.prev_buffer);

    for (x, y, cell) in changes {
        queue!(self.stdout, MoveTo(x, y))?;
        queue!(self.stdout, SetForegroundColor(cell.fg))?;
        queue!(self.stdout, SetBackgroundColor(cell.bg))?;
        queue!(self.stdout, Print(cell.ch))?;
    }

    self.stdout.flush()?;
    Ok(())
}
```

First frame: everything is different from empty buffer, full redraw.
Subsequent frames: only changed cells are written.

### Buffer Swap

```rust
std::mem::swap(&mut self.buffer, &mut self.prev_buffer);
```

Now:
- `prev_buffer` contains what we just drew
- `buffer` is ready to be cleared for next frame

## Phase 4: Event Handling

```rust
if let Some(event) = terminal.poll_event()? {
    if let Event::Key(key) = event {
        match (key.modifiers, key.code) {
            // Quit
            (m, KeyCode::Char('q')) if m.contains(KeyModifiers::CONTROL) => {
                break;
            }
            // Focus navigation
            (KeyModifiers::NONE, KeyCode::Tab) => {
                focus.focus_next();
            }
            // Activation
            (KeyModifiers::NONE, KeyCode::Enter) => {
                focus.activate();
            }
            _ => {}
        }
    }
}
```

### Activation Flow

When Enter is pressed:

```rust
pub fn activate(&self) {
    if let Some(Some(callback)) = self.focusables.get(self.focus_index) {
        callback();  // Call the stored closure
    }
}
```

The callback is:
```rust
move || c.update(|n| *n + 1)
```

Which calls:
```rust
impl<T> State<T> {
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.inner.value.borrow_mut());
        *self.inner.dirty.borrow_mut() = true;
    }
}
```

State is now:
```
StateInner {
    value: RefCell { value: 1 },  // Was 0, now 1
    dirty: RefCell { value: true }
}
```

### Loop Continues

After event handling, the loop repeats:
1. Render phase runs again
2. `count.get()` now returns 1
3. View tree has updated text
4. Draw phase shows new value

## Complete Data Flow Diagram

```
┌──────────────────────────────────────────────────────────┐
│                      Main Loop                           │
│                                                          │
│  ┌─────────────────┐                                     │
│  │  StateStorage   │◄─────────────────────────┐          │
│  │   (persists)    │                          │          │
│  └────────┬────────┘                          │          │
│           │ Rc clone                          │          │
│           ▼                                   │          │
│  ┌─────────────────┐                          │          │
│  │     Scope       │                          │          │
│  └────────┬────────┘                          │          │
│           │ passed to                         │          │
│           ▼                                   │          │
│  ┌─────────────────┐    ┌──────────────┐      │          │
│  │   Component     │───▶│    View      │      │          │
│  │   (closure)     │    │   (tree)     │      │          │
│  └─────────────────┘    └──────┬───────┘      │          │
│                                │              │          │
│           ┌────────────────────┼──────────────┤          │
│           │                    │              │          │
│           ▼                    ▼              │          │
│  ┌─────────────────┐  ┌──────────────┐        │          │
│  │  FocusManager   │  │   Terminal   │        │          │
│  │  (callbacks)    │  │   (buffer)   │        │          │
│  └────────┬────────┘  └──────┬───────┘        │          │
│           │                  │                │          │
│           │                  ▼                │          │
│           │           ┌──────────────┐        │          │
│           │           │   Screen     │        │          │
│           │           └──────────────┘        │          │
│           │                  ▲                │          │
│           │                  │ User input     │          │
│           │                  │                │          │
│           ▼                  │                │          │
│  ┌─────────────────┐         │                │          │
│  │    activate()   │─────────┘                │          │
│  │    callback()   │                          │          │
│  └────────┬────────┘                          │          │
│           │                                   │          │
│           │ state.update()                    │          │
│           └───────────────────────────────────┘          │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

## Timing Characteristics

| Operation | Typical Time |
|-----------|-------------|
| Scope creation | ~10 ns (Rc clone) |
| Hook lookup | ~50 ns (Vec index + downcast) |
| View tree build | ~1 μs (small tree) |
| Focus collection | ~1 μs (tree walk) |
| Buffer render | ~100 μs (10k cells) |
| Buffer diff | ~50 μs (comparison) |
| Terminal write | ~1 ms (I/O bound) |
| Event poll | 100 ms (timeout) |

Total frame time: dominated by event poll timeout.
Render path: ~2 ms for complex UI.

## Memory Characteristics

| Structure | Memory |
|-----------|--------|
| StateStorage | 24 bytes + states |
| State<T> | 8 bytes (Rc pointer) |
| StateInner<T> | 24 bytes + sizeof(T) |
| View node | ~48-96 bytes each |
| Buffer | width × height × 12 bytes |

For a typical terminal (200×50):
- Buffer: ~120 KB
- Two buffers: ~240 KB
- View tree (100 nodes): ~10 KB

Total: ~250 KB per application.

## Key Invariants

1. **StateStorage outlives all renders**
   - Created once in `run()`
   - Shared via Rc with all Scopes

2. **Hook index resets each render**
   - `with_storage()` calls `reset_index()`
   - Hooks are called in same order

3. **View tree is rebuilt each frame**
   - No diffing at View level
   - Fresh tree from component

4. **Buffer diff catches all changes**
   - Full comparison, no missed updates
   - Only writes what changed

5. **Callbacks own state handles**
   - Captured via `move`
   - State lives in StateStorage

[← Back to main](../architecture.md) | [Next: What We Avoided →](what-we-avoided.md)
