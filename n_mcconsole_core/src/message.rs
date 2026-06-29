use std::any::{Any, TypeId};

/// Marker trait for message types
pub trait Message: Any + Send {}

/// A message which is tagged by an id and may only be opened by a matching tag
pub struct Tagged<P> {
    tag: u64,
    payload: P,
}

impl<P> Tagged<P> {
    pub fn new(tag: u64, payload: P) -> Self {
        Self { tag, payload }
    }

    /// Try reading the payload, if the tag differs returns None
    pub fn open(&self, tag: u64) -> Option<&P> {
        (tag == self.tag).then_some(&self.payload)
    }
}

impl<P: Send + 'static> Message for Tagged<P> {}

/// A message fired multiple times per second, can be used to run animations
pub struct Tick;

/// A simple wrapper over a String
pub struct LogLine {
    pub line: String,
}

/// Fired by some jobs to explicitly state that their job is done
pub struct JobDone {
    /// Whether the job ran into some issues or not
    pub ok: bool,
}

impl Message for Tick {}

/// Encloses a message erasing its type, used in the message delivery system to send messages to their subscribers
pub struct Envelope {
    pub tid: TypeId,
    pub payload: Box<dyn Any + Send>,
}

impl Envelope {
    pub fn new<T: Message>(msg: T) -> Self {
        Envelope {
            tid: TypeId::of::<T>(),
            payload: Box::new(msg),
        }
    }
}
