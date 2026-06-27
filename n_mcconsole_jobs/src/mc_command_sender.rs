use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::{Message, Tagged};
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::Job;
use n_mcconsole_event_bus::job_emits;
use std::sync::Arc;

pub struct McCommandSenderJob;

job_emits!(McCommandSenderJob => Tagged<McCommandSentMessage>);

impl Job for McCommandSenderJob {
    fn run(self, tag: u64, writer: EventWriter, executor: Arc<dyn Executor>) {}
}

pub struct McCommandSentMessage;
