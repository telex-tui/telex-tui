# Forms

Declarative form validation with built-in validators and error handling.

```rust
let form = state!(cx, || {
    FormState::new()
        .field(
            FieldBuilder::new("email")
                .required()
                .email()
                .error_message("Please enter a valid email")
                .build()
        )
});

View::form_field("email")
    .label("Email Address")
    .value(form.get().get_value("email"))
    .error(form.get().get_error("email"))
    .on_change(with!(form => move |v: String| {
        form.get().set_value("email", v);
    }))
    .build()
```

Run with: `cargo run -p telex-tui --example 22_forms`

## Form state

Create a `FormState` to manage all fields:

```rust
let form = state!(cx, || {
    FormState::new()
        .field(FieldBuilder::new("username").required().build())
        .field(FieldBuilder::new("email").required().email().build())
        .field(FieldBuilder::new("password").required().min_length(8).build())
});
```

Each field has a name, validators, and optional error messages.

## Built-in validators

`FieldBuilder` provides common validators:

```rust
FieldBuilder::new("field_name")
    .required()                    // must not be empty
    .email()                       // must be valid email
    .min_length(n)                 // at least n characters
    .max_length(n)                 // at most n characters
    .integer()                     // must parse as integer
    .error_message("Custom error") // shown on validation failure
    .build()
```

Validators chain - all must pass for the field to be valid.

## Custom validators

Add custom validation logic:

```rust
FieldBuilder::new("username")
    .custom(|value| {
        if value.contains(' ') {
            Some("Username cannot contain spaces".into())
        } else if !value.chars().all(|c| c.is_alphanumeric()) {
            Some("Only letters and numbers allowed".into())
        } else {
            None  // Valid
        }
    })
    .build()
```

Custom validators return `Option<String>`:
- `None` - field is valid
- `Some(msg)` - field is invalid, show this error

## Form fields

Render fields with `View::form_field()`:

```rust
View::form_field("email")
    .label("Email Address *")
    .value(form.get().get_value("email"))
    .placeholder("you@example.com")
    .error(form.get().get_error("email"))
    .on_change(with!(form => move |v: String| {
        form.get().set_value("email", v);
    }))
    .on_blur(with!(form => move || {
        form.get().touch("email");
    }))
    .build()
```

The form field widget handles:
- Label display
- Input rendering
- Error message display (red text below input)
- Placeholder text

## Validation timing

Validation happens on two events:

**On change:** Update the value, but don't show errors yet
```rust
.on_change(with!(form => move |v| {
    form.get().set_value("field", v);
}))
```

**On blur:** Mark field as "touched" to show errors
```rust
.on_blur(with!(form => move || {
    form.get().touch("field");
}))
```

This prevents showing errors before the user finishes typing.

## Submitting the form

Check validity before processing:

```rust
let on_submit = with!(form => move || {
    if form.get().validate() {
        // Form is valid - process it
        let values = form.get().values();
        let email = values.get("email").unwrap();
        let password = values.get("password").unwrap();

        save_user(email, password);
    } else {
        // Show error message
        show_error("Please fix the errors above");
    }
});

View::button()
    .label("Submit")
    .on_press(on_submit)
    .build()
```

`.validate()` returns `true` if all fields are valid.

## Accessing values

Get individual field values:

```rust
let email = form.get().get_value("email");  // returns String
```

Get all values as a map:

```rust
let values = form.get().values();  // HashMap<String, String>
let email = values.get("email").unwrap_or(&String::new());
```

## Accessing errors

Get field-specific errors:

```rust
let email_error = form.get().get_error("email");  // Option<String>
```

Check if the whole form is valid:

```rust
let is_valid = form.get().is_valid();  // bool
```

## Password fields

For sensitive input like passwords, use the `.password(true)` method to hide the text as asterisks:

```rust
View::form_field("password")
    .label("Password")
    .value(password)
    .password(true)  // display as ****
    .error(error)
    .on_change(on_change)
    .build()
```

The actual value is still stored normally in state - only the display is masked.

## Optional fields

Fields without `.required()` can be empty:

```rust
FieldBuilder::new("age")
    .integer()  // if not empty, must be an integer
    .custom(|v| {
        if v.is_empty() {
            return None;  // empty is ok
        }
        match v.parse::<i32>() {
            Ok(n) if n < 0 => Some("Age must be positive".into()),
            Ok(n) if n > 150 => Some("Please enter a realistic age".into()),
            Ok(_) => None,
            Err(_) => Some("Must be a number".into()),
        }
    })
    .build()
```

## Resetting the form

Clear all values and errors:

```rust
let on_reset = with!(form => move || {
    form.get().reset();
});

View::button()
    .label("Reset")
    .on_press(on_reset)
    .build()
```

## Form layout

Use `View::form()` to layout fields with consistent spacing:

```rust
View::form()
    .spacing(1)
    .child(email_field)
    .child(password_field)
    .child(username_field)
    .build()
```

This is just a `vstack` with spacing - purely cosmetic.

## Real-world example

Registration form:

```rust
let form = state!(cx, || {
    FormState::new()
        .field(
            FieldBuilder::new("email")
                .required()
                .email()
                .error_message("Please enter a valid email address")
                .build()
        )
        .field(
            FieldBuilder::new("username")
                .required()
                .min_length(3)
                .max_length(20)
                .custom(|v| {
                    if !v.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        Some("Only letters, numbers, and underscores".into())
                    } else {
                        None
                    }
                })
                .build()
        )
        .field(
            FieldBuilder::new("password")
                .required()
                .min_length(8)
                .custom(|v| {
                    if !v.chars().any(|c| c.is_numeric()) {
                        Some("Must contain at least one number".into())
                    } else {
                        None
                    }
                })
                .build()
        )
});
```

## Validation messages

Show form-level validation status:

```rust
View::text(format!(
    "Form valid: {}",
    if form.get().is_valid() { "Yes" } else { "No" }
))
.color(if form.get().is_valid() { Color::Green } else { Color::Red })
```

## Common patterns

**Confirm password:**
```rust
FieldBuilder::new("confirm_password")
    .custom({
        let password = password.clone();
        move |v| {
            if v != password.get() {
                Some("Passwords do not match".into())
            } else {
                None
            }
        }
    })
    .build()
```

**Trim whitespace before validation:**
```rust
.on_change(with!(form => move |v: String| {
    form.get().set_value("email", v.trim().to_string());
}))
```

**Disable submit until valid:**
```rust
View::button()
    .label("Submit")
    .disabled(!form.get().is_valid())
    .on_press(on_submit)
    .build()
```

**Show error count:**
```rust
let error_count = form.get().values().keys()
    .filter(|k| form.get().get_error(k).is_some())
    .count();

if error_count > 0 {
    View::text(format!("{} errors remaining", error_count))
}
```

## Tips

**Touch on blur** - Only call `.touch()` when the user leaves a field. This prevents showing errors too early.

**Required fields in labels** - Mark required fields with `*` in the label: `"Email Address *"`.

**Custom error messages** - Use `.error_message()` for friendlier errors than the defaults.

**Validate on submit** - Always call `.validate()` before processing, even if you track validity in the UI.

**Field names are keys** - Use consistent field names throughout (get_value, set_value, touch, get_error all use the same name).

**Validators compose** - Chain multiple validators: `.required().email().min_length(5)`.

**Custom validators are flexible** - Access other state, make async checks (in the validator), whatever you need.

**Reset doesn't re-validate** - After `.reset()`, the form is back to initial state. Errors won't show until fields are touched again.

Next: [Menus](./menus.md)
