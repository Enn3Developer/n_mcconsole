use n_mcconsole_core::message::{Envelope, Message};

#[derive(Default)]
pub struct Outbox {
    pub(crate) pending: Vec<Envelope>,
}

impl Outbox {
    pub fn emit<T: Message>(&mut self, msg: T) {
        self.pending.push(Envelope::new(msg));
    }
}
