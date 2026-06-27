use crate::command::{Command, Output};
use std::io;

pub struct Streaming {
    pub lines: Box<dyn Iterator<Item = io::Result<String>> + Send>,
    pub killer: Option<std::process::Child>,
}

pub trait Executor: Send + Sync {
    fn run(&self, cmd: &Command) -> io::Result<Output>;
    fn spawn_streaming(&self, cmd: &Command) -> io::Result<Streaming>;
    fn read_file(&self, path: &str) -> io::Result<Vec<u8>>;
    fn write_file(&self, path: &str, data: &[u8]) -> io::Result<()>;
    fn list_dir(&self, path: &str) -> io::Result<Vec<String>>;
}
