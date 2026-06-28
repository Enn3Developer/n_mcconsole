use crate::user_management::{HELPER, Role};
use n_mcconsole_core::command::Command;
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Tagged;
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use std::sync::Arc;

pub struct SetRoleJob {
    pub user: String,
    pub role: Role,
}

job_emits!(SetRoleJob => Tagged<SetRoleMessage>);

impl SetRoleJob {
    pub fn new(user: impl Into<String>, role: Role) -> Self {
        Self {
            user: user.into(),
            role,
        }
    }
}

// TODO: error handling
impl Job for SetRoleJob {
    fn run(
        self,
        tag: u64,
        writer: EventWriter,
        executor: Arc<dyn Executor>,
        _token: Option<JobToken>,
    ) {
        let Ok(output) = executor.run(
            &Command::new("pkexec")
                .arg(HELPER)
                .arg(&self.user)
                .arg(self.role),
        ) else {
            let _ = writer.bus_tagged(tag, SetRoleMessage::Err());
            return;
        };

        if !output.success() {
            let _ = writer.bus_tagged(tag, SetRoleMessage::Err());
            return;
        }

        let _ = writer.bus_tagged(tag, SetRoleMessage::Ok(self.user));
    }
}

pub enum SetRoleMessage {
    Err(),
    Ok(String),
}
