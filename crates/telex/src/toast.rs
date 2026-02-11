//! Toast notification system for ephemeral messages.
//!
//! Toasts are non-interactive, auto-dismissing notifications that appear
//! in a corner of the screen.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// The severity level of a toast notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastLevel {
    /// Informational message (default).
    #[default]
    Info,
    /// Success message.
    Success,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
}

/// A single toast notification.
#[derive(Clone)]
pub struct Toast {
    /// Unique identifier for this toast.
    pub id: u64,
    /// The message to display.
    pub message: String,
    /// Severity level.
    pub level: ToastLevel,
    /// Duration before auto-dismiss.
    pub duration: Duration,
    /// When this toast was created.
    pub created_at: Instant,
}

impl Toast {
    /// Check if this toast has expired.
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    /// Get the remaining time as a fraction (0.0 to 1.0).
    pub fn remaining_fraction(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32();
        (1.0 - elapsed / total).max(0.0)
    }
}

/// A queue of toast notifications.
#[derive(Clone)]
pub struct ToastQueue {
    inner: Rc<RefCell<ToastQueueInner>>,
}

struct ToastQueueInner {
    toasts: Vec<Toast>,
    next_id: u64,
    default_duration: Duration,
}

impl Default for ToastQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastQueue {
    /// Create a new toast queue.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(ToastQueueInner {
                toasts: Vec::new(),
                next_id: 1,
                default_duration: Duration::from_secs(3),
            })),
        }
    }

    /// Create a new toast queue with a custom default duration.
    pub fn with_duration(duration: Duration) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ToastQueueInner {
                toasts: Vec::new(),
                next_id: 1,
                default_duration: duration,
            })),
        }
    }

    /// Show an info toast.
    pub fn info(&self, message: impl Into<String>) -> u64 {
        self.push(message, ToastLevel::Info)
    }

    /// Show a success toast.
    pub fn success(&self, message: impl Into<String>) -> u64 {
        self.push(message, ToastLevel::Success)
    }

    /// Show a warning toast.
    pub fn warning(&self, message: impl Into<String>) -> u64 {
        self.push(message, ToastLevel::Warning)
    }

    /// Show an error toast.
    pub fn error(&self, message: impl Into<String>) -> u64 {
        self.push(message, ToastLevel::Error)
    }

    /// Show an error toast with a longer duration (5 seconds).
    pub fn error_long(&self, message: impl Into<String>) -> u64 {
        self.push_with_duration(message, ToastLevel::Error, Duration::from_secs(5))
    }

    /// Push a toast with a specific level.
    pub fn push(&self, message: impl Into<String>, level: ToastLevel) -> u64 {
        let duration = self.inner.borrow().default_duration;
        self.push_with_duration(message, level, duration)
    }

    /// Push a toast with a specific level and duration.
    pub fn push_with_duration(
        &self,
        message: impl Into<String>,
        level: ToastLevel,
        duration: Duration,
    ) -> u64 {
        let mut inner = self.inner.borrow_mut();
        let id = inner.next_id;
        inner.next_id += 1;

        inner.toasts.push(Toast {
            id,
            message: message.into(),
            level,
            duration,
            created_at: Instant::now(),
        });

        id
    }

    /// Remove a specific toast by ID.
    pub fn dismiss(&self, id: u64) {
        let mut inner = self.inner.borrow_mut();
        inner.toasts.retain(|t| t.id != id);
    }

    /// Clear all toasts.
    pub fn clear(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.toasts.clear();
    }

    /// Remove expired toasts and return the current list.
    pub fn collect(&self) -> Vec<Toast> {
        let mut inner = self.inner.borrow_mut();
        // Remove expired toasts
        inner.toasts.retain(|t| !t.is_expired());
        // Return a copy
        inner.toasts.clone()
    }

    /// Check if there are any toasts to display.
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.borrow();
        inner.toasts.is_empty()
    }

    /// Get the number of active toasts.
    pub fn len(&self) -> usize {
        let inner = self.inner.borrow();
        inner.toasts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_queue() {
        let queue = ToastQueue::with_duration(Duration::from_millis(100));

        let id1 = queue.info("Test message");
        assert_eq!(queue.len(), 1);

        let _id2 = queue.success("Success!");
        assert_eq!(queue.len(), 2);

        queue.dismiss(id1);
        assert_eq!(queue.len(), 1);

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_toast_expiry() {
        let toast = Toast {
            id: 1,
            message: "Test".to_string(),
            level: ToastLevel::Info,
            duration: Duration::from_millis(1),
            created_at: Instant::now() - Duration::from_millis(10),
        };

        assert!(toast.is_expired());
    }

    #[test]
    fn test_toast_levels() {
        let queue = ToastQueue::new();

        queue.info("Info");
        queue.success("Success");
        queue.warning("Warning");
        queue.error("Error");

        let toasts = queue.collect();
        assert_eq!(toasts.len(), 4);
        assert_eq!(toasts[0].level, ToastLevel::Info);
        assert_eq!(toasts[1].level, ToastLevel::Success);
        assert_eq!(toasts[2].level, ToastLevel::Warning);
        assert_eq!(toasts[3].level, ToastLevel::Error);
    }
}
