use crate::modpack_updater::TAG;
use n_mcconsole_core::command::Command;
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::{JobDone, LogLine, Tagged};
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use std::sync::Arc;

pub struct UpdaterLogReaderJob;

job_emits!(UpdaterLogReaderJob => Tagged<LogLine>, Tagged<JobDone>);

impl Job for UpdaterLogReaderJob {
    fn run(
        self,
        tag: u64,
        writer: EventWriter,
        executor: Arc<dyn Executor>,
        mut token: Option<JobToken>,
    ) {
        let Ok(stream) = executor.spawn_streaming(
            &Command::new("journalctl")
                .arg("-t")
                .arg(TAG)
                .arg("-o")
                .arg("cat")
                .arg("-n")
                .arg("0")
                .arg("-f"),
        ) else {
            let _ = writer.bus_tagged(tag, JobDone { ok: false });
            return;
        };

        if let Some(token) = token.as_mut()
            && let Some(child) = stream.killer
        {
            token.register_child(child);
        }

        for line in stream.lines {
            if token.as_ref().is_some_and(|t| t.cancelled()) {
                break;
            }
            let Ok(line) = line else {
                continue;
            };

            if writer.bus_tagged(tag, LogLine { line }).is_err() {
                break;
            }
        }

        let _ = writer.bus_tagged(tag, JobDone { ok: true });
    }
}
