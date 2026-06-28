use crate::user_management::HELPER;
use n_mcconsole_core::command::{Command, Reason};
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Tagged;
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use std::sync::Arc;

pub struct DeleteUserJob {
    pub user: String,
}

job_emits!(DeleteUserJob => Tagged<DeleteUserMessage>);

impl DeleteUserJob {
    pub fn new(user: impl Into<String>) -> Self {
        Self { user: user.into() }
    }
}

impl Job for DeleteUserJob {
    fn run(
        self,
        tag: u64,
        writer: EventWriter,
        executor: Arc<dyn Executor>,
        _token: Option<JobToken>,
    ) {
        let Ok(output) = executor.run(&Command::new("pkexec").arg(HELPER).arg(&self.user)) else {
            let _ = writer.bus_tagged(tag, DeleteUserMessage::Err(Reason::Internal));
            return;
        };

        if !output.success() {
            let _ = writer.bus_tagged(tag, DeleteUserMessage::Err(Reason::from_exit(output.code)));
            return;
        }

        let _ = writer.bus_tagged(tag, DeleteUserMessage::Ok(self.user));
    }
}

pub enum DeleteUserMessage {
    Err(Reason),
    Ok(String),
}
