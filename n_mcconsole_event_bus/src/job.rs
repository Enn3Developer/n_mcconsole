use crate::event::EventWriter;
use n_mcconsole_core::command::Command;
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::{JobDone, LogLine, Tagged};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

pub trait Job: Send + 'static {
    fn run(self, tag: u64, writer: EventWriter, executor: Arc<dyn Executor>, token: JobToken);
}

#[macro_export]
macro_rules! job_emits {
    ($job:ty => $($msg:ty),+ $(,)?) => {
        impl $job {
            pub fn subscribe<S>(reg: &mut $crate::registrar::Registrar<S>)
            where
                S: $crate::scene::Scene $(+ $crate::Handle<$msg>)+,
            {
                $( reg.on::<$msg>(); )+
            }
        }
    };
}

#[derive(Clone)]
pub struct JobControl {
    pub writer: EventWriter,
    pub executor: Arc<dyn Executor>,
    pub next: Arc<AtomicU64>,
}

impl JobControl {
    pub fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::Relaxed)
    }

    pub fn spawn_stream(&self, cmd: Command, tag: u64) -> JobHandle {
        let exec = self.executor.clone();
        let writer = self.writer.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let killer: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
        let (stop_t, killer_t) = (stop.clone(), killer.clone());

        thread::spawn(move || {
            if let Ok(mut s) = exec.spawn_streaming(&cmd) {
                *killer_t.lock().unwrap() = s.killer.take();
                for line in s.lines {
                    if stop_t.load(Ordering::Relaxed) {
                        break;
                    }
                    match line {
                        Ok(l) => {
                            if writer.bus_tagged(tag, LogLine { line: l }).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            let _ = writer.bus_tagged(tag, JobDone { ok: true });
        });

        JobHandle { stop, killer }
    }

    pub fn spawn_oneshot(&self, cmd: Command, tag: u64) {
        let exec = self.executor.clone();
        let writer = self.writer.clone();
        thread::spawn(move || {
            let ok = exec.run(&cmd).map(|o| o.success).unwrap_or(false);
            let _ = writer.bus_tagged(tag, JobDone { ok });
        });
    }

    pub fn spawn_job<J: Job>(&self, job: J, tag: u64) -> JobHandle {
        let exec = self.executor.clone();
        let writer = self.writer.clone();

        let stop = Arc::new(AtomicBool::new(false));
        let killer: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
        let token = JobToken {
            stop: stop.clone(),
            killer: killer.clone(),
        };

        thread::spawn(move || {
            job.run(tag, writer, exec, token);
        });

        JobHandle { stop, killer }
    }

    pub fn start<J: Job>(&self, job: J) -> RunningJob {
        let tag = self.next_id();
        let handle = self.spawn_job(job, tag);
        RunningJob {
            tag,
            handle: Some(handle),
        }
    }
}

#[derive(Clone)]
pub struct JobToken {
    stop: Arc<AtomicBool>,
    killer: Arc<Mutex<Option<std::process::Child>>>,
}

impl JobToken {
    pub fn cancelled(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    pub fn register_child(&self, child: std::process::Child) {
        *self.killer.lock().unwrap() = Some(child);
    }
}

/// Handle to a spawned job
///
/// When you drop the handle, it signals the job to stop
pub struct JobHandle {
    stop: Arc<AtomicBool>,
    killer: Arc<Mutex<Option<std::process::Child>>>,
}

impl Drop for JobHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(mut c) = self.killer.lock().unwrap().take() {
            let _ = c.kill();
        }
    }
}

pub struct RunningJob {
    tag: u64,
    handle: Option<JobHandle>,
}

impl RunningJob {
    pub fn tag(&self) -> u64 {
        self.tag
    }

    pub fn open<'a, P>(&self, m: &'a Tagged<P>) -> Option<&'a P> {
        m.open(self.tag)
    }
}
