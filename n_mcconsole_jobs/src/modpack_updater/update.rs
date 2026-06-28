use crate::modpack_updater::HELPER;
use n_mcconsole_core::command::Command;
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Tagged;
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use std::sync::Arc;

pub struct UpdateJob {
    pub version: String,
}

job_emits!(UpdateJob => Tagged<UpdateMessage>);

impl UpdateJob {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
        }
    }
}

impl Job for UpdateJob {
    fn run(
        self,
        tag: u64,
        writer: EventWriter,
        executor: Arc<dyn Executor>,
        _token: Option<JobToken>,
    ) {
        let Ok(output) = executor.run(&Command::new("pkexec").arg(HELPER).arg(self.version)) else {
            let _ = writer.bus_tagged(tag, UpdateMessage::Err);
            return;
        };

        if !output.success() {
            let _ = writer.bus_tagged(tag, UpdateMessage::Err);
            return;
        }

        let _ = writer.bus_tagged(tag, UpdateMessage::Ok);
    }
}

pub enum UpdateMessage {
    Err,
    Ok,
}
