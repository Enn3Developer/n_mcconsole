use n_mcconsole_core::command::{Command, Reason};
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Tagged;
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use std::sync::Arc;

const HELPER: &str = "/usr/local/sbin/mcconsole-notify";

#[derive(Clone, Copy)]
pub enum UnitAction {
    Stop,
    Restart,
}

impl From<String> for UnitAction {
    fn from(value: String) -> Self {
        if value == "stop" {
            UnitAction::Stop
        } else if value == "restart" {
            UnitAction::Restart
        } else {
            panic!("Invalid action {} for unit management", value)
        }
    }
}

impl From<UnitAction> for String {
    fn from(value: UnitAction) -> Self {
        match value {
            UnitAction::Stop => "stop".into(),
            UnitAction::Restart => "restart".into(),
        }
    }
}

pub struct UnitManagementJob {
    pub action: UnitAction,
}

job_emits!(UnitManagementJob => Tagged<UnitManagementMessage>);

impl UnitManagementJob {
    pub fn new(action: UnitAction) -> Self {
        Self { action }
    }
}

// TODO: error handling
impl Job for UnitManagementJob {
    fn run(
        self,
        tag: u64,
        writer: EventWriter,
        executor: Arc<dyn Executor>,
        _token: Option<JobToken>,
    ) {
        let Ok(output) = executor.run(&Command::new("pkexec").arg(HELPER).arg(self.action)) else {
            let _ = writer.bus_tagged(tag, UnitManagementMessage::Err(Reason::Internal));
            return;
        };

        if !output.success() {
            let _ = writer.bus_tagged(
                tag,
                UnitManagementMessage::Err(Reason::from_exit(output.code)),
            );
            return;
        }

        let _ = writer.bus_tagged(tag, self.action);
    }
}

pub enum UnitManagementMessage {
    Err(Reason),
    Ok(UnitAction),
}
