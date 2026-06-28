use crate::modpack_updater::MANIFEST;
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Tagged;
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize)]
struct Manifest {
    versions: HashMap<String, serde::de::IgnoredAny>,
}

impl Manifest {
    fn versions(&self) -> Vec<String> {
        self.versions.keys().cloned().collect()
    }

    fn gt(&self, version: &String) -> Vec<String> {
        let Some(index) = self.versions.keys().position(|v| version == v) else {
            return vec![];
        };

        self.versions.keys().take(index).cloned().collect()
    }
}

#[derive(Default)]
pub struct ListVersionsJob {
    pub current: Option<String>,
}

job_emits!(ListVersionsJob => Tagged<ListVersionsMessage>);

impl ListVersionsJob {
    pub fn new(current: impl Into<String>) -> Self {
        Self {
            current: Some(current.into()),
        }
    }
}

impl Job for ListVersionsJob {
    fn run(
        self,
        tag: u64,
        writer: EventWriter,
        _executor: Arc<dyn Executor>,
        _token: Option<JobToken>,
    ) {
        let Ok(response) = reqwest::blocking::get(MANIFEST) else {
            let _ = writer.bus_tagged(tag, ListVersionsMessage::default());
            return;
        };
        let Ok(manifest) = response.json::<Manifest>() else {
            let _ = writer.bus_tagged(tag, ListVersionsMessage::default());
            return;
        };

        let versions = match &self.current {
            None => manifest.versions(),
            Some(v) => manifest.gt(v),
        };

        let _ = writer.bus_tagged(tag, ListVersionsMessage { versions });
    }
}

#[derive(Default)]
pub struct ListVersionsMessage {
    pub versions: Vec<String>,
}
