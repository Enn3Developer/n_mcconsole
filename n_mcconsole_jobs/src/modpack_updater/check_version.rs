use crate::modpack_updater::VERSION_FILE;
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Message;
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use std::sync::Arc;

pub struct CheckVersionJob;

job_emits!(CheckVersionJob => CheckVersionMessage);

impl Job for CheckVersionJob {
    fn run(
        self,
        _tag: u64,
        writer: EventWriter,
        executor: Arc<dyn Executor>,
        _token: Option<JobToken>,
    ) {
        let version = String::from_utf8(executor.read_file(VERSION_FILE).unwrap_or_default())
            .unwrap_or_default();
        let version = if version.is_empty() {
            None
        } else {
            Some(version)
        };

        let _ = writer.bus(CheckVersionMessage { version });
    }
}

pub struct CheckVersionMessage {
    pub version: Option<String>,
}

impl Message for CheckVersionMessage {}
