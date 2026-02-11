//! Form validation system for declarative input validation.
//!
//! Forms collect multiple fields and validate them against rules,
//! providing error messages when validation fails.

use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Type alias for custom validation functions.
type ValidationFn = Rc<dyn Fn(&str) -> Option<String>>;

/// A validator that can be applied to form fields.
#[derive(Clone)]
pub enum Validator {
    /// Field is required (non-empty).
    Required,
    /// Field must have at least this many characters.
    MinLength(usize),
    /// Field must have at most this many characters.
    MaxLength(usize),
    /// Field must match a regex pattern.
    Pattern(Regex),
    /// Field must be a valid email address.
    Email,
    /// Field must be a valid number.
    Number,
    /// Field must be a valid integer.
    Integer,
    /// Custom validation with error message.
    Custom(ValidationFn),
}

impl Validator {
    /// Create a custom validator with a closure.
    pub fn custom<F>(f: F) -> Self
    where
        F: Fn(&str) -> Option<String> + 'static,
    {
        Validator::Custom(Rc::new(f))
    }

    /// Create a pattern validator from a regex string.
    pub fn pattern(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Validator::Pattern(Regex::new(pattern)?))
    }

    /// Validate a value, returning an error message if invalid.
    pub fn validate(&self, value: &str) -> Option<String> {
        match self {
            Validator::Required => {
                if value.trim().is_empty() {
                    Some("This field is required".to_string())
                } else {
                    None
                }
            }
            Validator::MinLength(min) => {
                if value.len() < *min {
                    Some(format!("Must be at least {} characters", min))
                } else {
                    None
                }
            }
            Validator::MaxLength(max) => {
                if value.len() > *max {
                    Some(format!("Must be at most {} characters", max))
                } else {
                    None
                }
            }
            Validator::Pattern(regex) => {
                if regex.is_match(value) {
                    None
                } else {
                    Some("Invalid format".to_string())
                }
            }
            Validator::Email => {
                // Simple email validation
                let email_pattern = Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
                if value.is_empty() || email_pattern.is_match(value) {
                    None
                } else {
                    Some("Invalid email address".to_string())
                }
            }
            Validator::Number => {
                if value.is_empty() || value.parse::<f64>().is_ok() {
                    None
                } else {
                    Some("Must be a valid number".to_string())
                }
            }
            Validator::Integer => {
                if value.is_empty() || value.parse::<i64>().is_ok() {
                    None
                } else {
                    Some("Must be a valid integer".to_string())
                }
            }
            Validator::Custom(f) => f(value),
        }
    }
}

/// A single field in a form.
#[derive(Clone)]
pub struct FormField {
    /// The field name (identifier).
    pub name: String,
    /// Current value.
    pub value: String,
    /// Validators to apply.
    pub validators: Vec<Validator>,
    /// Current error message (if validation failed).
    pub error: Option<String>,
    /// Custom error message to use instead of validator messages.
    pub custom_error_message: Option<String>,
    /// Whether this field has been "touched" (user interacted with it).
    pub touched: bool,
}

impl FormField {
    /// Create a new form field.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: String::new(),
            validators: Vec::new(),
            error: None,
            custom_error_message: None,
            touched: false,
        }
    }

    /// Set the initial value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Add a validator.
    pub fn validate(mut self, validator: Validator) -> Self {
        self.validators.push(validator);
        self
    }

    /// Set a custom error message to use for any validation failure.
    pub fn error_message(mut self, message: impl Into<String>) -> Self {
        self.custom_error_message = Some(message.into());
        self
    }

    /// Run validation and return the error if any.
    pub fn run_validation(&mut self) -> Option<String> {
        for validator in &self.validators {
            if let Some(err) = validator.validate(&self.value) {
                let error = self.custom_error_message.clone().unwrap_or(err);
                self.error = Some(error.clone());
                return Some(error);
            }
        }
        self.error = None;
        None
    }

    /// Check if the field is valid (no error).
    pub fn is_valid(&self) -> bool {
        self.error.is_none()
    }
}

/// Form state managing multiple fields.
#[derive(Clone)]
pub struct FormState {
    inner: Rc<RefCell<FormStateInner>>,
}

struct FormStateInner {
    fields: HashMap<String, FormField>,
    field_order: Vec<String>,
}

impl Default for FormState {
    fn default() -> Self {
        Self::new()
    }
}

impl FormState {
    /// Create a new empty form state.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(FormStateInner {
                fields: HashMap::new(),
                field_order: Vec::new(),
            })),
        }
    }

    /// Add a field to the form.
    pub fn field(self, field: FormField) -> Self {
        let mut inner = self.inner.borrow_mut();
        let name = field.name.clone();
        inner.fields.insert(name.clone(), field);
        if !inner.field_order.contains(&name) {
            inner.field_order.push(name);
        }
        drop(inner);
        self
    }

    /// Get a field by name.
    pub fn get_field(&self, name: &str) -> Option<FormField> {
        let inner = self.inner.borrow();
        inner.fields.get(name).cloned()
    }

    /// Get the value of a field.
    pub fn get_value(&self, name: &str) -> String {
        let inner = self.inner.borrow();
        inner
            .fields
            .get(name)
            .map(|f| f.value.clone())
            .unwrap_or_default()
    }

    /// Set the value of a field.
    pub fn set_value(&self, name: &str, value: String) {
        let mut inner = self.inner.borrow_mut();
        if let Some(field) = inner.fields.get_mut(name) {
            field.value = value;
            field.touched = true;
            // Re-validate on change
            field.run_validation();
        }
    }

    /// Get the error for a field.
    pub fn get_error(&self, name: &str) -> Option<String> {
        let inner = self.inner.borrow();
        inner.fields.get(name).and_then(|f| f.error.clone())
    }

    /// Check if a field has been touched.
    pub fn is_touched(&self, name: &str) -> bool {
        let inner = self.inner.borrow();
        inner.fields.get(name).map(|f| f.touched).unwrap_or(false)
    }

    /// Mark a field as touched.
    pub fn touch(&self, name: &str) {
        let mut inner = self.inner.borrow_mut();
        if let Some(field) = inner.fields.get_mut(name) {
            field.touched = true;
            // Validate when touched
            field.run_validation();
        }
    }

    /// Validate all fields and return whether the form is valid.
    pub fn validate(&self) -> bool {
        let mut inner = self.inner.borrow_mut();
        let mut all_valid = true;
        for field in inner.fields.values_mut() {
            if field.run_validation().is_some() {
                all_valid = false;
            }
        }
        all_valid
    }

    /// Check if the entire form is valid (all fields pass validation).
    pub fn is_valid(&self) -> bool {
        let inner = self.inner.borrow();
        inner.fields.values().all(|f| f.error.is_none())
    }

    /// Get all field values as a HashMap.
    pub fn values(&self) -> HashMap<String, String> {
        let inner = self.inner.borrow();
        inner
            .fields
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    /// Get field names in order.
    pub fn field_names(&self) -> Vec<String> {
        let inner = self.inner.borrow();
        inner.field_order.clone()
    }

    /// Reset all fields to empty and clear errors.
    pub fn reset(&self) {
        let mut inner = self.inner.borrow_mut();
        for field in inner.fields.values_mut() {
            field.value = String::new();
            field.error = None;
            field.touched = false;
        }
    }

    /// Get all errors as a HashMap.
    pub fn errors(&self) -> HashMap<String, String> {
        let inner = self.inner.borrow();
        inner
            .fields
            .iter()
            .filter_map(|(k, v)| v.error.as_ref().map(|e| (k.clone(), e.clone())))
            .collect()
    }
}

/// Builder for creating form fields fluently.
pub struct FieldBuilder {
    field: FormField,
}

impl FieldBuilder {
    /// Create a new field builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            field: FormField::new(name),
        }
    }

    /// Set the initial value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.field.value = value.into();
        self
    }

    /// Add the Required validator.
    pub fn required(mut self) -> Self {
        self.field.validators.push(Validator::Required);
        self
    }

    /// Add the MinLength validator.
    pub fn min_length(mut self, min: usize) -> Self {
        self.field.validators.push(Validator::MinLength(min));
        self
    }

    /// Add the MaxLength validator.
    pub fn max_length(mut self, max: usize) -> Self {
        self.field.validators.push(Validator::MaxLength(max));
        self
    }

    /// Add the Email validator.
    pub fn email(mut self) -> Self {
        self.field.validators.push(Validator::Email);
        self
    }

    /// Add the Number validator.
    pub fn number(mut self) -> Self {
        self.field.validators.push(Validator::Number);
        self
    }

    /// Add the Integer validator.
    pub fn integer(mut self) -> Self {
        self.field.validators.push(Validator::Integer);
        self
    }

    /// Add a pattern validator.
    pub fn pattern(mut self, pattern: &str) -> Result<Self, regex::Error> {
        self.field
            .validators
            .push(Validator::Pattern(Regex::new(pattern)?));
        Ok(self)
    }

    /// Add a custom validator.
    pub fn custom<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Option<String> + 'static,
    {
        self.field.validators.push(Validator::Custom(Rc::new(f)));
        self
    }

    /// Set a custom error message.
    pub fn error_message(mut self, message: impl Into<String>) -> Self {
        self.field.custom_error_message = Some(message.into());
        self
    }

    /// Build the field.
    pub fn build(self) -> FormField {
        self.field
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_validator() {
        assert!(Validator::Required.validate("").is_some());
        assert!(Validator::Required.validate("   ").is_some());
        assert!(Validator::Required.validate("hello").is_none());
    }

    #[test]
    fn test_min_length_validator() {
        let validator = Validator::MinLength(5);
        assert!(validator.validate("hi").is_some());
        assert!(validator.validate("hello").is_none());
        assert!(validator.validate("hello world").is_none());
    }

    #[test]
    fn test_email_validator() {
        assert!(Validator::Email.validate("invalid").is_some());
        assert!(Validator::Email.validate("test@example.com").is_none());
        assert!(Validator::Email.validate("").is_none()); // Empty is valid (use Required for mandatory)
    }

    #[test]
    fn test_form_state() {
        let form = FormState::new()
            .field(FieldBuilder::new("email").required().email().build())
            .field(
                FieldBuilder::new("password")
                    .required()
                    .min_length(8)
                    .build(),
            );

        // Initially invalid (empty values)
        assert!(!form.validate());

        // Set valid values
        form.set_value("email", "test@example.com".to_string());
        form.set_value("password", "password123".to_string());
        assert!(form.validate());
        assert!(form.is_valid());

        // Invalid email
        form.set_value("email", "invalid".to_string());
        assert!(!form.is_valid());
    }

    #[test]
    fn test_custom_validator() {
        let field = FieldBuilder::new("username")
            .custom(|v| {
                if v.contains(' ') {
                    Some("Username cannot contain spaces".to_string())
                } else {
                    None
                }
            })
            .build();

        let form = FormState::new().field(field);

        form.set_value("username", "hello world".to_string());
        assert!(!form.is_valid());
        assert_eq!(
            form.get_error("username"),
            Some("Username cannot contain spaces".to_string())
        );

        form.set_value("username", "helloworld".to_string());
        assert!(form.is_valid());
    }
}
