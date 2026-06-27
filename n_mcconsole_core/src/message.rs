use std::any::{Any, TypeId};

pub trait Message: Any + Send {}

pub struct Tagged<P> {
    tag: u64,
    payload: P,
}

impl<P> Tagged<P> {
    pub fn new(tag: u64, payload: P) -> Self {
        Self { tag, payload }
    }

    pub fn open(&self, tag: u64) -> Option<&P> {
        (tag == self.tag).then_some(&self.payload)
    }
}

impl<P: Send + 'static> Message for Tagged<P> {}

pub struct Tick;

pub struct LogLine {
    pub line: String,
}

pub struct JobDone {
    pub ok: bool,
}

impl Message for Tick {}

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
