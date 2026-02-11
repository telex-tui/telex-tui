# Reducer

Action-dispatched state management for complex state transitions.

```rust
let (state, dispatch) = reducer!(cx, AppState::Idle, |state, action| {
    match (state, action) {
        (_, Action::Reset) => AppState::Idle,
        (AppState::Idle, Action::Start) => AppState::Running,
        (AppState::Running, Action::Finish(data)) => AppState::Done(data),
        (s, _) => s,
    }
});
```

Run with: `cargo run -p telex-tui --example 36_reducer`

## What is reducer!?

`reducer!` creates a state/dispatch pair — a pattern familiar from React's `useReducer` or Redux. Instead of mutating state directly, you dispatch actions that a reducer function processes into new state.

```rust
let (state, dispatch) = reducer!(cx, initial_state, |state, action| {
    // Return new state based on current state + action
    new_state
});
```

Returns:
- **`state`** — A `State<S>` handle (same as `state!` returns)
- **`dispatch`** — An `Rc<dyn Fn(A)>` that accepts actions

## When to use reducer vs state!

**Use `reducer!` when:**
- State transitions depend on the current state (state machines)
- Multiple actions affect the same state in different ways
- You want to centralize state logic in one place

**Use `state!` when:**
- Simple values (counters, toggles, strings)
- Transitions don't depend on current state
- Direct `.set()` and `.update()` are clear enough

## Example: form submission

```rust
#[derive(Clone)]
enum FormState {
    Editing,
    Submitting,
    Success(String),
    Error(String),
}

enum FormAction {
    Submit,
    Succeed(String),
    Fail(String),
    Reset,
}

let (form, dispatch) = reducer!(cx, FormState::Editing, |state, action| {
    match (state, action) {
        (FormState::Editing, FormAction::Submit) => FormState::Submitting,
        (FormState::Submitting, FormAction::Succeed(msg)) => FormState::Success(msg),
        (FormState::Submitting, FormAction::Fail(err)) => FormState::Error(err),
        (_, FormAction::Reset) => FormState::Editing,
        (s, _) => s,  // ignore invalid transitions
    }
});

// In callbacks
let on_submit = {
    let dispatch = dispatch.clone();
    Rc::new(move || dispatch(FormAction::Submit))
};
```

## Dispatching from effects

```rust
effect!(cx, form.get(), {
    let dispatch = dispatch.clone();
    move |state| {
        if let FormState::Submitting = state {
            let dispatch = dispatch.clone();
            std::thread::spawn(move || {
                match submit_form() {
                    Ok(msg) => dispatch(FormAction::Succeed(msg)),
                    Err(e) => dispatch(FormAction::Fail(e.to_string())),
                }
            });
        }
        || {}
    }
});
```

## Tips

**State must be Clone** — The reducer function takes `S` by value and returns `S`. Your state type needs `Clone`.

**Actions can carry data** — Use enum variants with fields to pass data with actions: `Action::Loaded(String)`.

**Dispatch is Rc** — The dispatch function is wrapped in `Rc`, so it can be cloned into callbacks and closures.

**Order-independent** — Like all macros, `reducer!` is keyed by call site. Safe in conditionals.
