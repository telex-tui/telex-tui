use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

/// A reactive state handle that can be copied and shared.
///
/// State<T> is Copy, allowing closures to capture it without explicit cloning.
/// Internally it wraps Rc<RefCell<T>> for shared mutable state.
pub struct State<T> {
    inner: Rc<StateInner<T>>,
}

struct StateInner<T> {
    value: RefCell<T>,
    /// Flag to track if state changed since last render
    dirty: RefCell<bool>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

// Note: State is cheap to clone (just Rc pointer copy).
// Use .clone() in closures or move semantics.

impl<T> State<T> {
    /// Create a new state with the given initial value.
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(StateInner {
                value: RefCell::new(value),
                dirty: RefCell::new(false),
            }),
        }
    }

    /// Get a clone of the current value.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.value.borrow().clone()
    }

    /// Set a new value.
    pub fn set(&self, value: T) {
        *self.inner.value.borrow_mut() = value;
        *self.inner.dirty.borrow_mut() = true;
    }

    /// Update the value using a function.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.inner.value.borrow_mut());
        *self.inner.dirty.borrow_mut() = true;
    }

    /// Check if state has changed since last clear_dirty call.
    pub fn is_dirty(&self) -> bool {
        *self.inner.dirty.borrow()
    }

    /// Clear the dirty flag.
    pub fn clear_dirty(&self) {
        *self.inner.dirty.borrow_mut() = false;
    }

    /// Borrow the value immutably.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.inner.value.borrow())
    }
}

impl<T: fmt::Display> fmt::Display for State<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.value.borrow().fmt(f)
    }
}

impl<T: fmt::Debug> fmt::Debug for State<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("State")
            .field(&*self.inner.value.borrow())
            .finish()
    }
}
