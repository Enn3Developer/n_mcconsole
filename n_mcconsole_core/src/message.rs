use std::any::{Any, TypeId};

pub trait Message: Any + Send {}

pub struct Tick;

pub struct LogLine {
    pub tag: u64,
    pub line: String,
}

pub struct JobDone {
    pub tag: u64,
    pub ok: bool,
}

impl Message for Tick {}
impl Message for LogLine {}
impl Message for JobDone {}

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
