use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Tagged;
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use std::sync::Arc;

pub struct McCommandSenderJob;

job_emits!(McCommandSenderJob => Tagged<McCommandSentMessage>);

impl Job for McCommandSenderJob {
    fn run(
        self,
        _tag: u64,
        _writer: EventWriter,
        _executor: Arc<dyn Executor>,
        _token: Option<JobToken>,
    ) {
    }
}

pub struct McCommandSentMessage;
