use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Tagged;
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use std::sync::Arc;

pub struct ListDirJob {
    pub path: String,
}

job_emits!(ListDirJob => Tagged<ListDirMessage>);

impl ListDirJob {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl Job for ListDirJob {
    fn run(
        self,
        tag: u64,
        writer: EventWriter,
        executor: Arc<dyn Executor>,
        _token: Option<JobToken>,
    ) {
        let Ok(files) = executor.list_dir(&self.path) else {
            return;
        };

        let _ = writer.bus_tagged(tag, ListDirMessage { files });
    }
}

pub struct ListDirMessage {
    pub files: Vec<String>,
}
