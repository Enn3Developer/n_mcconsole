use n_mcconsole_core::command::Command;
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Tagged;
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use std::str::Split;
use std::sync::Arc;

const MC_VIEWER: &str = "mc-viewer";
const MC_OPERATOR: &str = "mc-operator";
const MC_ADMIN: &str = "mc-admin";

pub struct ListUsersJob;

job_emits!(ListUsersJob => Tagged<ListUsersMessage>);

// TODO: error handling
impl Job for ListUsersJob {
    fn run(
        self,
        tag: u64,
        writer: EventWriter,
        executor: Arc<dyn Executor>,
        _token: Option<JobToken>,
    ) {
        let Ok(output) = executor.run(
            &Command::new("getent")
                .arg("group")
                .arg(MC_VIEWER)
                .arg(MC_OPERATOR)
                .arg(MC_ADMIN),
        ) else {
            let _ = writer.bus_tagged(tag, ListUsersMessage::Err());
            return;
        };

        if !output.success {
            let _ = writer.bus_tagged(tag, ListUsersMessage::Err());
            return;
        }

        let Ok(parsed_output) = String::from_utf8(output.stdout) else {
            let _ = writer.bus_tagged(tag, ListUsersMessage::Err());
            return;
        };

        let mut groups = parsed_output.split('\n');
        let viewers = line_to_users(&mut groups);
        let operators = line_to_users(&mut groups);
        let admins = line_to_users(&mut groups);

        let _ = writer.bus_tagged(
            tag,
            ListUsersMessage::Ok {
                viewers,
                operators,
                admins,
            },
        );
    }
}

pub enum ListUsersMessage {
    Err(),
    Ok {
        viewers: Vec<String>,
        operators: Vec<String>,
        admins: Vec<String>,
    },
}

fn line_to_users(groups: &mut Split<char>) -> Vec<String> {
    groups
        .next()
        .unwrap()
        .split(':')
        .last()
        .unwrap()
        .split(',')
        .map(|s| String::from(s))
        .collect::<Vec<_>>()
}
