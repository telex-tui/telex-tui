# Deep Dive: Future Considerations

[← Back to main](../architecture.md)

---

This document explores what comes next for Telex, examining the design decisions that will shape future phases and how the current architecture supports growth.

## Phase 3: Reactive Primitives

### Derived State

**Current limitation:**

```rust
fn TotalPrice(cx: Scope) -> View {
    let quantity = cx.use_state(|| 1);
    let price = cx.use_state(|| 10.0);

    // Computed every render
    let total = quantity.get() * price.get();

    view! { <Text>{format!("Total: ${}", total)}</Text> }
}
```

**Future: `use_memo`**

```rust
fn TotalPrice(cx: Scope) -> View {
    let quantity = cx.use_state(|| 1);
    let price = cx.use_state(|| 10.0);

    // Only recomputed when dependencies change
    let total = cx.use_memo(
        || quantity.get() * price.get(),
        (quantity.get(), price.get())  // Dependency tuple
    );

    view! { <Text>{format!("Total: ${}", total.get())}</Text> }
}
```

**Implementation sketch:**

```rust
pub struct Memo<T> {
    value: RefCell<Option<T>>,
    deps: RefCell<Option<Box<dyn Any>>>,
}

impl Scope {
    pub fn use_memo<T, D>(&self, compute: impl FnOnce() -> T, deps: D) -> Memo<T>
    where
        T: 'static,
        D: PartialEq + 'static,
    {
        // Similar to use_state, but checks deps before recomputing
    }
}
```

**Design consideration:** Dependencies as tuple vs explicit tracking. Tuple is simpler, explicit is more flexible.

### Effects ✅ Implemented

Effects are now implemented via the `effect!` and `effect_once!` macros.

```rust
fn Logger(cx: Scope) -> View {
    let count = state!(cx, || 0);

    // Runs when count changes
    effect!(cx, count.get(), |&c| {
        println!("Count changed to {}", c);
        || println!("Cleaning up")
    });

    // Runs once on initialization
    effect_once!(cx, || {
        println!("Component mounted");
        || {}
    });

    view! { <Text>{count.get()}</Text> }
}
```

**Key features:**

1. **Order-independent:** Macros use TypeId keying, safe in conditionals
2. **Cleanup:** Cleanup functions run before next effect or on app exit
3. **Cycle detection:** Automatic panic if effect runs >100 times in 10 frames

See `docs/use-effect-design.md` and example `32_effects` for full documentation.

---

## Phase 4: Layout System

### Current: Simple Division

```rust
fn render_vstack(buffer: &mut Buffer, node: &VStackNode, area: Rect, ctx: &mut RenderContext) {
    let child_height = area.height / node.children.len() as u16;

    for (i, child) in node.children.iter().enumerate() {
        let child_area = Rect {
            x: area.x,
            y: area.y + (i as u16 * child_height),
            width: area.width,
            height: child_height,
        };
        render_view(buffer, child, child_area, ctx);
    }
}
```

**Limitation:** All children get equal space.

### Future: Constraint-Based Layout

**Desired API:**

```rust
view! {
    <VStack>
        <Text height={Fixed(1)}>"Header"</Text>
        <List height={Flex(1)}>{items}</List>
        <Text height={Fixed(1)}>"Footer"</Text>
    </VStack>
}
```

**Constraint types:**

```rust
pub enum Constraint {
    Fixed(u16),       // Exact size
    Min(u16),         // At least this size
    Max(u16),         // At most this size
    Flex(u16),        // Proportional (flex factor)
    Percent(u16),     // Percentage of parent
}
```

**Layout algorithm (Cassowary-inspired):**

```rust
fn solve_constraints(available: u16, constraints: &[Constraint]) -> Vec<u16> {
    // 1. Sum fixed sizes
    // 2. Distribute remaining to flex items by ratio
    // 3. Enforce min/max constraints
    // 4. Iterate until stable
}
```

**Design consideration:** Full Cassowary solver vs simpler algorithm. Simple is probably enough for TUI.

### Text Measurement

**Challenge:** Know text dimensions before layout.

```rust
// Need to measure this before deciding layout
let description = "This is a long description that might wrap...";
```

**Approach:**

```rust
impl Buffer {
    pub fn measure_text(&self, text: &str, max_width: u16) -> (u16, u16) {
        // Returns (width, height) accounting for wrapping
    }
}
```

---

## Phase 5: Input Handling

### Current: Global Key Handling

```rust
match key.code {
    KeyCode::Tab => focus.focus_next(),
    KeyCode::Enter => focus.activate(),
    // Everything handled at top level
}
```

### Future: Bubbling Events

**Desired behavior:**

```rust
fn Form(cx: Scope) -> View {
    view! {
        <VStack on_key={|key| handle_form_keys(key)}>
            <TextInput />
            <TextInput />
            <Button>"Submit"</Button>
        </VStack>
    }
}
```

Events bubble up from focused element:
1. TextInput gets key
2. If not handled, VStack gets it
3. If not handled, parent gets it

**Implementation:**

```rust
pub enum EventResult {
    Handled,
    Propagate,
}

pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

// In View
pub struct VStackNode {
    pub children: Vec<View>,
    pub on_key: Option<Rc<dyn Fn(KeyEvent) -> EventResult>>,
}
```

### TextInput Widget

**State needed:**

```rust
pub struct TextInputState {
    value: String,
    cursor: usize,
    selection: Option<(usize, usize)>,
}
```

**API:**

```rust
fn LoginForm(cx: Scope) -> View {
    let username = cx.use_state(|| String::new());
    let password = cx.use_state(|| String::new());

    view! {
        <VStack>
            <TextInput
                value={username.get()}
                on_change={move |s| username.set(s)}
                placeholder="Username"
            />
            <TextInput
                value={password.get()}
                on_change={move |s| password.set(s)}
                secret={true}
            />
        </VStack>
    }
}
```

**Challenges:**
- Unicode handling (grapheme clusters)
- Cursor positioning
- Selection rendering
- Copy/paste

---

## Phase 6: Async Support

### The Challenge

TUI apps need async for:
- Network requests
- File I/O
- Timers
- Background tasks

But our render loop is synchronous.

### Approach: `use_async`

```rust
fn UserProfile(cx: Scope, user_id: u64) -> View {
    let user = cx.use_async(async move {
        api::fetch_user(user_id).await
    });

    match user.get() {
        AsyncState::Pending => view! { <Text>"Loading..."</Text> },
        AsyncState::Ready(user) => view! { <Text>{user.name}</Text> },
        AsyncState::Error(e) => view! { <Text>{format!("Error: {}", e)}</Text> },
    }
}
```

**Implementation approach:**

```rust
pub enum AsyncState<T> {
    Pending,
    Ready(T),
    Error(String),
}

pub struct AsyncHandle<T> {
    state: Rc<RefCell<AsyncState<T>>>,
}

impl Scope {
    pub fn use_async<T, F>(&self, future: F) -> AsyncHandle<T>
    where
        T: 'static,
        F: Future<Output = T> + 'static,
    {
        // Spawn future, update state when complete
        // Trigger re-render on completion
    }
}
```

**Runtime consideration:**

Option 1: Bring our own runtime
```rust
// In run()
let rt = tokio::runtime::Builder::new_current_thread().build()?;
rt.block_on(async { main_loop().await });
```

Option 2: Let user provide runtime
```rust
#[tokio::main]
async fn main() {
    telex::run_async(App).await.unwrap();
}
```

Option 3: Use smol (lightweight)
```rust
// Minimal async runtime, good for TUI
smol::block_on(main_loop());
```

---

## Phase 7+: Advanced Features

### Context (Dependency Injection)

**Problem:** Passing data through many layers.

```rust
// Current: prop drilling
fn App(cx: Scope) -> View {
    let theme = Theme::dark();
    view! {
        <Sidebar theme={theme.clone()}>
            <Menu theme={theme.clone()}>
                <MenuItem theme={theme.clone()} />
            </Menu>
        </Sidebar>
    }
}
```

**Future: Context**

```rust
fn App(cx: Scope) -> View {
    let theme = Theme::dark();

    cx.provide_context(theme);

    view! {
        <Sidebar>
            <Menu>
                <MenuItem />  // Can access theme via context
            </Menu>
        </Sidebar>
    }
}

fn MenuItem(cx: Scope) -> View {
    let theme = cx.use_context::<Theme>();
    // ...
}
```

**Implementation:**

```rust
impl Scope {
    pub fn provide_context<T: 'static>(&self, value: T) {
        self.storage.contexts.borrow_mut().insert(
            TypeId::of::<T>(),
            Rc::new(value)
        );
    }

    pub fn use_context<T: 'static>(&self) -> Option<Rc<T>> {
        self.storage.contexts.borrow()
            .get(&TypeId::of::<T>())
            .and_then(|any| any.downcast_ref::<Rc<T>>().cloned())
    }
}
```

### List Virtualization

**Problem:** Rendering 10,000 items.

```rust
// Naive: renders all items
fn FileList(cx: Scope, files: Vec<File>) -> View {
    view! {
        <VStack>
            {files.iter().map(|f| view! { <Text>{f.name}</Text> }).collect()}
        </VStack>
    }
}
```

**Future: VirtualList**

```rust
fn FileList(cx: Scope, files: Vec<File>) -> View {
    view! {
        <VirtualList
            items={files}
            item_height={1}
            render_item={|file| view! { <Text>{file.name}</Text> }}
        />
    }
}
```

**Implementation:**

```rust
pub struct VirtualListNode<T> {
    items: Vec<T>,
    item_height: u16,
    scroll_offset: usize,
    render_item: Rc<dyn Fn(&T) -> View>,
}

fn render_virtual_list<T>(buffer: &mut Buffer, node: &VirtualListNode<T>, area: Rect) {
    let visible_count = area.height / node.item_height;
    let start = node.scroll_offset;
    let end = (start + visible_count as usize).min(node.items.len());

    for (i, item) in node.items[start..end].iter().enumerate() {
        let item_area = Rect {
            y: area.y + (i as u16 * node.item_height),
            height: node.item_height,
            ..area
        };
        let view = (node.render_item)(item);
        render_view(buffer, &view, item_area);
    }
}
```

### Component Keys

**Problem:** List reordering loses state.

```rust
fn TodoList(cx: Scope, todos: Vec<Todo>) -> View {
    view! {
        <VStack>
            {todos.iter().map(|t| {
                // If todos reorder, state gets mixed up
                <TodoItem todo={t} />
            }).collect()}
        </VStack>
    }
}
```

**Future: Keys**

```rust
view! {
    <VStack>
        {todos.iter().map(|t| {
            <TodoItem key={t.id} todo={t} />
        }).collect()}
    </VStack>
}
```

**Implementation challenge:**

Need to associate hook state with keys, not call order. This requires:
1. Tracking keys in StateStorage
2. Matching keys across renders
3. Migrating state when keys move

This is the biggest architectural change and may require significant refactoring.

---

## Extensibility Paths

### Custom Widgets

**Current:** All widgets are in the View enum.

**Future option 1: Escape hatch**

```rust
pub enum View {
    // ... existing variants
    Custom(Box<dyn Widget>),
}

pub trait Widget {
    fn render(&self, buffer: &mut Buffer, area: Rect, ctx: &mut RenderContext);
    fn focusable(&self) -> bool;
    fn on_key(&self, key: KeyEvent) -> EventResult;
}
```

**Future option 2: Macro-based registration**

```rust
#[rte::widget]
pub struct Sparkline {
    data: Vec<f64>,
    max: f64,
}

impl Sparkline {
    fn render(&self, buffer: &mut Buffer, area: Rect) {
        // Custom rendering
    }
}

// Generates View::Sparkline variant automatically
```

### Styling System

**Current:** Inline colors.

```rust
View::text("Hello").fg(Color::Red)
```

**Future: Themes**

```rust
#[derive(Theme)]
struct MyTheme {
    text: Style,
    button: Style,
    button_focused: Style,
    error: Style,
}

fn App(cx: Scope) -> View {
    cx.provide_context(MyTheme::default());

    view! {
        <Text style="error">"Something went wrong"</Text>
    }
}
```

### Testing Utilities

**Future:**

```rust
#[test]
fn test_counter() {
    let test = TestHarness::new(Counter);

    assert_eq!(test.find_text("Count: 0").is_some(), true);

    test.press_key(KeyCode::Enter);  // Activate focused button

    assert_eq!(test.find_text("Count: 1").is_some(), true);
}
```

**Implementation:**

```rust
pub struct TestHarness<C: Component> {
    component: C,
    storage: Rc<StateStorage>,
    focus: FocusManager,
    buffer: Buffer,
}

impl<C: Component> TestHarness<C> {
    pub fn render(&mut self) -> View {
        let cx = Scope::with_storage(Rc::clone(&self.storage));
        self.component.render(cx)
    }

    pub fn press_key(&mut self, key: KeyCode) {
        // Simulate key press
    }

    pub fn find_text(&self, needle: &str) -> Option<Rect> {
        // Search buffer for text
    }
}
```

---

## Migration Strategy

Each future feature follows this pattern:

### 1. Additive Changes First

```rust
// Phase N: Add new feature
impl Scope {
    pub fn use_memo(...) { ... }  // New method
}

// Existing code unchanged
```

### 2. Deprecation Window

```rust
// Phase N+1: Deprecate old approach
#[deprecated(since = "0.3", note = "Use use_memo instead")]
pub fn computed(...) { ... }
```

### 3. Breaking Changes in Major Versions

```rust
// Version 1.0: Remove deprecated APIs
// Old code must be updated
```

### Versioning Philosophy

- **0.x:** API may change, deprecation not guaranteed
- **1.0:** Stable API, deprecation window for changes
- **2.0:** Breaking changes accumulated

---

## Summary: The Path Forward

### Near Term (Phases 3-4)

| Feature | Complexity | Impact |
|---------|------------|--------|
| use_memo | Low | Medium - optimization |
| use_effect | Medium | High - side effects |
| Constraints | Medium | High - real layouts |
| Text measurement | Low | Medium - proper sizing |

### Medium Term (Phases 5-6)

| Feature | Complexity | Impact |
|---------|------------|--------|
| Event bubbling | Medium | High - real apps |
| TextInput | High | Critical - forms |
| use_async | High | Critical - real apps |

### Long Term (Phase 7+)

| Feature | Complexity | Impact |
|---------|------------|--------|
| Context | Medium | Medium - DX |
| Virtualization | High | High - large data |
| Keys | Very High | Medium - edge cases |
| Custom widgets | Medium | Low - extensibility |

### Guiding Principles

1. **Ship working software** - Each phase produces usable features
2. **Defer complexity** - Add it when needed, not before
3. **Maintain simplicity** - New features shouldn't complicate existing ones
4. **Preserve escape hatches** - Users can work around limitations
5. **Value stability** - Don't break what works

The current architecture supports all these features. The foundations are solid.

[← Back to main](../architecture.md)

