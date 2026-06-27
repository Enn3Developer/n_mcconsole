use n_mcconsole_core::command::{Command, Output};
use n_mcconsole_core::executor::{Executor, Streaming};
use std::io::{BufRead, BufReader};

pub struct LocalExecutor;

impl Executor for LocalExecutor {
    fn run(&self, cmd: &Command) -> std::io::Result<Output> {
        let o = std::process::Command::new(&cmd.program)
            .args(&cmd.args)
            .output()?;
        Ok(Output {
            success: o.status.success(),
            stdout: o.stdout,
            stderr: o.stderr,
        })
    }

    fn spawn_streaming(&self, cmd: &Command) -> std::io::Result<Streaming> {
        let mut child = std::process::Command::new(&cmd.program)
            .args(&cmd.args)
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        let out = child.stdout.take().expect("piped stdout");
        Ok(Streaming {
            lines: Box::new(BufReader::new(out).lines()),
            killer: Some(child),
        })
    }

    fn read_file(&self, path: &str) -> std::io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn write_file(&self, path: &str, data: &[u8]) -> std::io::Result<()> {
        std::fs::write(path, data)
    }

    fn list_dir(&self, path: &str) -> std::io::Result<Vec<String>> {
        let mut v = vec![];
        for e in std::fs::read_dir(path)? {
            v.push(e?.file_name().to_string_lossy().into_owned());
        }
        Ok(v)
    }
}
