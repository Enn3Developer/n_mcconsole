use n_mcconsole_core::message::{Envelope, Message};

/// A list of messages you can send back to the event bus
#[derive(Default)]
pub struct Outbox {
    pub(crate) pending: Vec<Envelope>,
}

impl Outbox {
    pub fn emit<T: Message>(&mut self, msg: T) {
        self.pending.push(Envelope::new(msg));
    }
}
