//! Async state management for RTE.
//!
//! Provides the `Async<T>` enum and `use_async` hook for loading data.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Type alias for the async result channel.
type AsyncChannel<T> = (Sender<Result<T, String>>, Receiver<Result<T, String>>);

/// Represents the state of an async operation.
#[derive(Clone)]
pub enum Async<T: Clone> {
    /// The operation is still loading.
    Loading,
    /// The operation completed successfully.
    Ready(T),
    /// The operation failed with an error.
    Error(String),
}

impl<T: Clone> Async<T> {
    /// Returns true if the async operation is still loading.
    pub fn is_loading(&self) -> bool {
        matches!(self, Async::Loading)
    }

    /// Returns true if the async operation completed successfully.
    pub fn is_ready(&self) -> bool {
        matches!(self, Async::Ready(_))
    }

    /// Returns true if the async operation failed.
    pub fn is_error(&self) -> bool {
        matches!(self, Async::Error(_))
    }

    /// Returns the value if ready, or None otherwise.
    pub fn get(&self) -> Option<&T> {
        match self {
            Async::Ready(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the value if ready, or a default.
    pub fn unwrap_or(&self, default: T) -> T {
        match self {
            Async::Ready(v) => v.clone(),
            _ => default,
        }
    }

    /// Returns the value if ready, or computes a default.
    pub fn unwrap_or_else(&self, f: impl FnOnce() -> T) -> T {
        match self {
            Async::Ready(v) => v.clone(),
            _ => f(),
        }
    }
}

/// Internal state for tracking async operations.
pub(crate) struct AsyncInner<T> {
    /// The current result (None if still loading).
    result: Option<Result<T, String>>,
    /// Whether the async operation has been started.
    started: bool,
    /// Receiver for the async result.
    receiver: Option<Receiver<Result<T, String>>>,
}

/// Handle for async state that can be stored and cloned.
pub struct AsyncHandle<T> {
    inner: Rc<RefCell<AsyncInner<T>>>,
}

impl<T> Clone for AsyncHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: Clone + Send + 'static> AsyncHandle<T> {
    /// Create a new async handle.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(AsyncInner {
                result: None,
                started: false,
                receiver: None,
            })),
        }
    }

    /// Start the async operation if not already started.
    pub fn start<F>(&self, f: F)
    where
        F: FnOnce() -> Result<T, String> + Send + 'static,
    {
        let mut inner = self.inner.borrow_mut();
        if inner.started {
            return;
        }

        inner.started = true;

        // Create channel for result
        let (tx, rx): AsyncChannel<T> = mpsc::channel();
        inner.receiver = Some(rx);

        // Spawn thread to run the function
        thread::spawn(move || {
            let result = f();
            let _ = tx.send(result);
        });
    }

    /// Poll for completion and return current state.
    pub fn poll(&self) -> Async<T>
    where
        T: Clone,
    {
        let mut inner = self.inner.borrow_mut();

        // If we already have a result, return it
        if let Some(ref result) = inner.result {
            return match result {
                Ok(v) => Async::Ready(v.clone()),
                Err(e) => Async::Error(e.clone()),
            };
        }

        // Try to receive from channel
        if let Some(ref receiver) = inner.receiver {
            match receiver.try_recv() {
                Ok(result) => {
                    let async_result = match &result {
                        Ok(v) => Async::Ready(v.clone()),
                        Err(e) => Async::Error(e.clone()),
                    };
                    inner.result = Some(result);
                    return async_result;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // Still loading
                    return Async::Loading;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Thread panicked or channel closed
                    let error = "Async operation failed: channel disconnected".to_string();
                    inner.result = Some(Err(error.clone()));
                    return Async::Error(error);
                }
            }
        }

        Async::Loading
    }

    /// Reset the async state to allow refetching.
    #[allow(dead_code)]
    pub fn reset(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.result = None;
        inner.started = false;
        inner.receiver = None;
    }
}

impl<T: Clone + Send + 'static> Default for AsyncHandle<T> {
    fn default() -> Self {
        Self::new()
    }
}
