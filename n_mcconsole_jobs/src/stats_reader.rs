use chrono::{DateTime, Utc};
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::Message;
use n_mcconsole_event_bus::event::EventWriter;
use n_mcconsole_event_bus::job::{Job, JobToken};
use n_mcconsole_event_bus::job_emits;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const STATS_FILE: &str = "/run/minecraft/stats";

pub struct StatsReaderJob;

job_emits!(StatsReaderJob => StatsReaderMessage);

impl Job for StatsReaderJob {
    fn run(
        self,
        _tag: u64,
        writer: EventWriter,
        executor: Arc<dyn Executor>,
        token: Option<JobToken>,
    ) {
        loop {
            if token.as_ref().is_some_and(|t| t.cancelled()) {
                break;
            }

            let Ok(file) = executor.read_file(STATS_FILE) else {
                let _ = writer.bus(StatsReaderMessage::default());
                break;
            };

            let Ok(content) = String::from_utf8(file) else {
                let _ = writer.bus(StatsReaderMessage::default());
                break;
            };

            let stats = content
                .lines()
                .filter_map(|l| Stats::try_from(l).ok())
                .collect();

            if writer.bus(StatsReaderMessage { stats }).is_err() {
                break;
            }

            thread::sleep(Duration::from_secs(5));
        }
    }
}

pub struct Stats {
    pub ts: DateTime<Utc>,
    pub tps: f32,
    pub players: Vec<String>,
}

impl TryFrom<&str> for Stats {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut split = value.split_whitespace();
        let ts_str = split.next().ok_or(())?;
        let tps_str = split.next().ok_or(())?;

        let ts_int = ts_str.parse::<i64>().map_err(|_| ())?;
        let ts = DateTime::from_timestamp_secs(ts_int).ok_or(())?;

        let tps = tps_str.parse::<f32>().map_err(|_| ())?;

        let players = split
            .next()
            .unwrap_or_default()
            .split(',')
            .map(String::from)
            .collect();

        Ok(Self { ts, tps, players })
    }
}

#[derive(Default)]
pub struct StatsReaderMessage {
    pub stats: Vec<Stats>,
}

impl Message for StatsReaderMessage {}
