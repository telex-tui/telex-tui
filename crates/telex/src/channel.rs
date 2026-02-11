//! Channel and port primitives for external event sources.
//!
//! `ChannelHandle<T>` is a typed inbound channel: external threads send `T`,
//! the run loop drains them each frame, and render code reads the messages.
//!
//! `PortHandle<In, Out>` is a bidirectional port: inbound `In` messages plus
//! an outbound `Sender<Out>` for sending data back to external systems.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

/// Trait for the run loop to drain all registered channels without knowing
/// their concrete types.
pub trait ChannelDrain {
    /// Move pending messages from the mpsc receiver into the frame buffer.
    fn drain(&self);
    /// Clear the frame buffer (called at the start of each frame).
    fn clear(&self);
    /// Returns true if there are messages in the current frame buffer.
    fn has_messages(&self) -> bool;
}

/// A typed inbound channel handle.
///
/// The `tx()` method returns a `Sender<T>` that can be cloned and sent to
/// other threads. Each frame, the run loop drains the receiver and stores
/// messages in `messages`. Components read messages with `get()`.
///
/// # Example
/// ```rust,ignore
/// let ch = channel!(cx, String);
///
/// // In an effect, hand the sender to an external thread
/// let tx = ch.tx();
/// effect_once!(cx, move || {
///     std::thread::spawn(move || {
///         tx.send("hello from thread".to_string()).ok();
///     });
///     || {}
/// });
///
/// // In render, read this frame's messages
/// for msg in ch.get() {
///     // ...
/// }
/// ```
/// A sender that wakes the event loop when a message is sent.
///
/// Wraps `mpsc::Sender<T>` and atomically sets a wake flag on each send,
/// so the run loop polls with zero timeout on the next iteration.
pub struct WakingSender<T> {
    tx: mpsc::Sender<T>,
    wake: Arc<AtomicBool>,
}

impl<T> Clone for WakingSender<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            wake: Arc::clone(&self.wake),
        }
    }
}

impl<T> WakingSender<T> {
    /// Send a value, waking the event loop.
    pub fn send(&self, value: T) -> Result<(), mpsc::SendError<T>> {
        let result = self.tx.send(value);
        if result.is_ok() {
            self.wake.store(true, Ordering::Release);
        }
        result
    }
}

pub struct ChannelHandle<T> {
    inner: Rc<ChannelInner<T>>,
}

struct ChannelInner<T> {
    tx: mpsc::Sender<T>,
    wake: Arc<AtomicBool>,
    rx: RefCell<mpsc::Receiver<T>>,
    messages: RefCell<Vec<T>>,
}

impl<T> Clone for ChannelHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: 'static> ChannelHandle<T> {
    /// Create a new channel handle with an event-loop wake flag.
    pub fn new(wake: Arc<AtomicBool>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            inner: Rc::new(ChannelInner {
                tx,
                wake,
                rx: RefCell::new(rx),
                messages: RefCell::new(Vec::new()),
            }),
        }
    }

    /// Get a cloneable sender that wakes the event loop on send.
    pub fn tx(&self) -> WakingSender<T> {
        WakingSender {
            tx: self.inner.tx.clone(),
            wake: Arc::clone(&self.inner.wake),
        }
    }

    /// Get this frame's messages (populated by the run loop's drain pass).
    pub fn get(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.inner.messages.borrow().clone()
    }

    /// Get the number of messages this frame.
    pub fn len(&self) -> usize {
        self.inner.messages.borrow().len()
    }

    /// Check if there are no messages this frame.
    pub fn is_empty(&self) -> bool {
        self.inner.messages.borrow().is_empty()
    }
}

impl<T: 'static> ChannelDrain for ChannelHandle<T> {
    fn drain(&self) {
        let rx = self.inner.rx.borrow();
        let mut messages = self.inner.messages.borrow_mut();
        loop {
            match rx.try_recv() {
                Ok(msg) => messages.push(msg),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }
    }

    fn clear(&self) {
        self.inner.messages.borrow_mut().clear();
    }

    fn has_messages(&self) -> bool {
        !self.inner.messages.borrow().is_empty()
    }
}

/// A bidirectional port handle: inbound `In` + outbound `Sender<Out>`.
///
/// Use `port.rx` for inbound messages and `port.tx()` for the outbound sender.
///
/// # Example
/// ```rust,ignore
/// let midi = port!(cx, MidiIn, MidiOut);
///
/// // Send the port's senders to an external connection
/// let tx_in = midi.rx.tx();   // external thread sends MidiIn here
/// let tx_out = midi.tx();     // component sends MidiOut here
///
/// effect_once!(cx, move || {
///     let conn = start_midi(tx_in, tx_out);
///     move || drop(conn)
/// });
///
/// // Read inbound messages
/// for msg in midi.rx.get() { /* ... */ }
/// ```
pub struct PortHandle<In, Out> {
    /// Inbound channel (external -> component).
    pub rx: ChannelHandle<In>,
    /// Outbound sender (component -> external).
    outbound_tx: mpsc::Sender<Out>,
    /// Outbound receiver (for external thread to consume).
    outbound_rx: Rc<RefCell<Option<mpsc::Receiver<Out>>>>,
}

impl<In, Out> Clone for PortHandle<In, Out> {
    fn clone(&self) -> Self {
        Self {
            rx: self.rx.clone(),
            outbound_tx: self.outbound_tx.clone(),
            outbound_rx: Rc::clone(&self.outbound_rx),
        }
    }
}

impl<In: 'static, Out: 'static> PortHandle<In, Out> {
    /// Create a new bidirectional port with an event-loop wake flag.
    pub fn new(wake: Arc<AtomicBool>) -> Self {
        let (outbound_tx, outbound_rx) = mpsc::channel();
        Self {
            rx: ChannelHandle::new(wake),
            outbound_tx,
            outbound_rx: Rc::new(RefCell::new(Some(outbound_rx))),
        }
    }

    /// Get a cloneable `Sender<Out>` for sending outbound messages.
    pub fn tx(&self) -> mpsc::Sender<Out> {
        self.outbound_tx.clone()
    }

    /// Take the outbound receiver (can only be called once).
    ///
    /// This is intended for passing to an external thread that consumes
    /// outbound messages. Returns `None` if already taken.
    pub fn take_outbound_rx(&self) -> Option<mpsc::Receiver<Out>> {
        self.outbound_rx.borrow_mut().take()
    }
}

/// Internal handle for `use_interval`. Spawns a timer thread and invokes
/// a callback each frame that ticks were received.
pub(crate) struct IntervalHandle {
    inner: Rc<IntervalInner>,
}

struct IntervalInner {
    rx: RefCell<mpsc::Receiver<()>>,
    callback: RefCell<Option<Rc<dyn Fn()>>>,
    ticked: RefCell<bool>,
}

impl Clone for IntervalHandle {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl IntervalHandle {
    /// Create a new interval that fires at the given duration.
    pub(crate) fn new(duration: Duration, callback: Rc<dyn Fn()>, wake: Arc<AtomicBool>) -> Self {
        let (tx, rx) = mpsc::channel();

        // Spawn timer thread
        std::thread::spawn(move || loop {
            std::thread::sleep(duration);
            if tx.send(()).is_err() {
                break; // Receiver dropped, stop
            }
            wake.store(true, Ordering::Release);
        });

        Self {
            inner: Rc::new(IntervalInner {
                rx: RefCell::new(rx),
                callback: RefCell::new(Some(callback)),
                ticked: RefCell::new(false),
            }),
        }
    }
}

impl ChannelDrain for IntervalHandle {
    fn drain(&self) {
        let rx = self.inner.rx.borrow();
        let mut ticked = false;
        loop {
            match rx.try_recv() {
                Ok(()) => ticked = true,
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }
        if ticked {
            *self.inner.ticked.borrow_mut() = true;
            if let Some(cb) = self.inner.callback.borrow().as_ref() {
                cb();
            }
        }
    }

    fn clear(&self) {
        *self.inner.ticked.borrow_mut() = false;
    }

    fn has_messages(&self) -> bool {
        *self.inner.ticked.borrow()
    }
}
