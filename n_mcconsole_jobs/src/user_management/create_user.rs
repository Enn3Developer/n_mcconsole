use crate::user_management::{HELPER, Role};
use n_mcconsole_core::command::{Command, Reason};
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Tagged;
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use std::sync::Arc;

pub struct CreateUserJob {
    pub user: String,
    pub role: Role,
    pub pubkey: Vec<u8>,
}

job_emits!(CreateUserJob => Tagged<CreateUserMessage>);

impl CreateUserJob {
    pub fn new(user: impl Into<String>, role: Role, pubkey: impl Into<Vec<u8>>) -> Self {
        Self {
            user: user.into(),
            role,
            pubkey: pubkey.into(),
        }
    }
}

// TODO: error handling
impl Job for CreateUserJob {
    fn run(
        self,
        tag: u64,
        writer: EventWriter,
        executor: Arc<dyn Executor>,
        _token: Option<JobToken>,
    ) {
        let Ok(output) = executor.run_stdin(
            &Command::new("pkexec")
                .arg(HELPER)
                .arg(&self.user)
                .arg(self.role),
            &self.pubkey,
        ) else {
            let _ = writer.bus_tagged(tag, CreateUserMessage::Err(Reason::Internal));
            return;
        };

        if !output.success() {
            let _ = writer.bus_tagged(tag, CreateUserMessage::Err(Reason::from_exit(output.code)));
            return;
        }

        let _ = writer.bus_tagged(tag, CreateUserMessage::Ok(self.user));
    }
}

pub enum CreateUserMessage {
    Err(Reason),
    Ok(String),
}
