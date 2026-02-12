//! Stream state management for Telex.
//!
//! Provides the `use_stream` hook for handling async streams (e.g., LLM token streaming).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

/// Represents the state of a streaming operation.
#[derive(Clone, Debug)]
pub enum StreamState {
    /// Stream not started yet.
    Idle,
    /// Stream is active and receiving data.
    Streaming,
    /// Stream completed successfully.
    Done,
    /// Stream encountered an error.
    Error(String),
}

/// Handle for stream state that can be stored and cloned.
pub struct StreamHandle<T> {
    inner: Rc<RefCell<StreamInner<T>>>,
}

struct StreamInner<T> {
    /// Accumulated values from the stream.
    accumulated: T,
    /// Current state of the stream.
    state: StreamState,
    /// Whether the stream has been started.
    started: bool,
    /// Receiver for stream items.
    receiver: Option<Receiver<StreamItem<T>>>,
    /// Wake flag to notify the event loop when tokens arrive.
    wake_flag: Option<Arc<AtomicBool>>,
}

/// An item received from the stream.
enum StreamItem<T> {
    /// A value from the stream.
    Value(T),
    /// Stream completed.
    Done,
    /// Stream errored.
    Error(String),
}

impl<T> Clone for StreamHandle<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: Clone + Default + 'static> StreamHandle<T> {
    /// Create a new stream handle with default accumulated value.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(StreamInner {
                accumulated: T::default(),
                state: StreamState::Idle,
                started: false,
                receiver: None,
                wake_flag: None,
            })),
        }
    }

    /// Create a new stream handle with an event-loop wake flag.
    pub fn with_wake_flag(wake_flag: Arc<AtomicBool>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(StreamInner {
                accumulated: T::default(),
                state: StreamState::Idle,
                started: false,
                receiver: None,
                wake_flag: Some(wake_flag),
            })),
        }
    }

    /// Create a new stream handle with a specific initial value.
    pub fn with_initial(initial: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(StreamInner {
                accumulated: initial,
                state: StreamState::Idle,
                started: false,
                receiver: None,
                wake_flag: None,
            })),
        }
    }

    /// Get the current accumulated value.
    pub fn get(&self) -> T {
        self.inner.borrow().accumulated.clone()
    }

    /// Check if the stream is currently loading/streaming.
    pub fn is_loading(&self) -> bool {
        matches!(
            self.inner.borrow().state,
            StreamState::Idle | StreamState::Streaming
        )
    }

    /// Check if the stream is actively receiving data.
    pub fn is_streaming(&self) -> bool {
        matches!(self.inner.borrow().state, StreamState::Streaming)
    }

    /// Check if the stream has completed.
    pub fn is_done(&self) -> bool {
        matches!(self.inner.borrow().state, StreamState::Done)
    }

    /// Check if the stream encountered an error.
    pub fn is_error(&self) -> bool {
        matches!(self.inner.borrow().state, StreamState::Error(_))
    }

    /// Get the error message if there was an error.
    pub fn error(&self) -> Option<String> {
        match &self.inner.borrow().state {
            StreamState::Error(e) => Some(e.clone()),
            _ => None,
        }
    }

    /// Get the current stream state.
    pub fn state(&self) -> StreamState {
        self.inner.borrow().state.clone()
    }
}

impl<T: Clone + Send + 'static> StreamHandle<T> {
    /// Start the stream if not already started.
    ///
    /// The `stream_fn` should be a function that returns an iterator.
    /// Each item from the iterator will be sent through the channel.
    pub fn start<F, I>(&self, stream_fn: F)
    where
        F: FnOnce() -> I + Send + 'static,
        I: Iterator<Item = T> + Send + 'static,
    {
        let mut inner = self.inner.borrow_mut();
        if inner.started {
            return;
        }

        inner.started = true;
        inner.state = StreamState::Streaming;

        // Create channel for stream items
        let (tx, rx): (Sender<StreamItem<T>>, Receiver<StreamItem<T>>) = mpsc::channel();
        inner.receiver = Some(rx);
        let wake_flag = inner.wake_flag.clone();

        // Spawn thread to run the stream
        thread::spawn(move || {
            let iter = stream_fn();
            for item in iter {
                if tx.send(StreamItem::Value(item)).is_err() {
                    // Receiver dropped, stop streaming
                    return;
                }
                if let Some(ref flag) = wake_flag {
                    flag.store(true, Ordering::Release);
                }
            }
            let _ = tx.send(StreamItem::Done);
            if let Some(ref flag) = wake_flag {
                flag.store(true, Ordering::Release);
            }
        });
    }

    /// Start the stream with error handling.
    pub fn start_with_result<F, I>(&self, stream_fn: F)
    where
        F: FnOnce() -> Result<I, String> + Send + 'static,
        I: Iterator<Item = T> + Send + 'static,
    {
        let mut inner = self.inner.borrow_mut();
        if inner.started {
            return;
        }

        inner.started = true;
        inner.state = StreamState::Streaming;

        let (tx, rx): (Sender<StreamItem<T>>, Receiver<StreamItem<T>>) = mpsc::channel();
        inner.receiver = Some(rx);
        let wake_flag = inner.wake_flag.clone();

        thread::spawn(move || match stream_fn() {
            Ok(iter) => {
                for item in iter {
                    if tx.send(StreamItem::Value(item)).is_err() {
                        return;
                    }
                    if let Some(ref flag) = wake_flag {
                        flag.store(true, Ordering::Release);
                    }
                }
                let _ = tx.send(StreamItem::Done);
                if let Some(ref flag) = wake_flag {
                    flag.store(true, Ordering::Release);
                }
            }
            Err(e) => {
                let _ = tx.send(StreamItem::Error(e));
                if let Some(ref flag) = wake_flag {
                    flag.store(true, Ordering::Release);
                }
            }
        });
    }

    /// Poll for new items and update accumulated value.
    /// Returns true if there were updates.
    pub fn poll(&self, accumulate: impl Fn(&mut T, T)) -> bool {
        let mut inner = self.inner.borrow_mut();
        let mut updated = false;

        // Take receiver temporarily to avoid borrow conflicts
        if let Some(receiver) = inner.receiver.take() {
            // Drain all available items
            let mut new_state = None;
            loop {
                match receiver.try_recv() {
                    Ok(StreamItem::Value(item)) => {
                        accumulate(&mut inner.accumulated, item);
                        updated = true;
                    }
                    Ok(StreamItem::Done) => {
                        new_state = Some(StreamState::Done);
                        break;
                    }
                    Ok(StreamItem::Error(e)) => {
                        new_state = Some(StreamState::Error(e));
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        break;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        if !matches!(inner.state, StreamState::Done | StreamState::Error(_)) {
                            new_state = Some(StreamState::Error(
                                "Stream disconnected unexpectedly".to_string(),
                            ));
                        }
                        break;
                    }
                }
            }

            // Put receiver back (unless stream is done)
            if new_state.is_none() || matches!(new_state, Some(StreamState::Streaming)) {
                inner.receiver = Some(receiver);
            }

            if let Some(state) = new_state {
                inner.state = state;
            }
        }

        updated
    }

    /// Reset the stream to allow restarting.
    pub fn reset(&self)
    where
        T: Default,
    {
        let mut inner = self.inner.borrow_mut();
        inner.accumulated = T::default();
        inner.state = StreamState::Idle;
        inner.started = false;
        inner.receiver = None;
        // wake_flag is preserved across resets
    }

    /// Reset the stream with a specific initial value.
    pub fn reset_with(&self, initial: T) {
        let mut inner = self.inner.borrow_mut();
        inner.accumulated = initial;
        inner.state = StreamState::Idle;
        inner.started = false;
        inner.receiver = None;
        // wake_flag is preserved across resets
    }
}

impl<T: Clone + Default + 'static> Default for StreamHandle<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience handle specifically for text streaming.
/// Automatically accumulates string tokens.
pub type TextStreamHandle = StreamHandle<String>;

impl TextStreamHandle {
    /// Poll and accumulate text by concatenation.
    pub fn poll_text(&self) -> bool {
        self.poll(|acc, item| acc.push_str(&item))
    }
}
