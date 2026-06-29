use crate::command::{Command, Output};
use std::io;

/// Return type for streaming commands
pub struct Streaming {
    /// Command output as an iterator of lines
    pub lines: Box<dyn Iterator<Item = io::Result<String>> + Send>,
    pub killer: Option<std::process::Child>,
}

/// Command executor, can either be local or over ssh
pub trait Executor: Send + Sync {
    /// Run a command
    fn run(&self, cmd: &Command) -> io::Result<Output>;
    /// Run a command and feed data to its stdin
    fn run_stdin(&self, cmd: &Command, data: &[u8]) -> io::Result<Output>;
    /// Spawn a streaming command
    fn spawn_streaming(&self, cmd: &Command) -> io::Result<Streaming>;
    /// Read a file, this may not be UTF-8
    fn read_file(&self, path: &str) -> io::Result<Vec<u8>>;
    fn write_file(&self, path: &str, data: &[u8]) -> io::Result<()>;
    fn list_dir(&self, path: &str) -> io::Result<Vec<String>>;
}
