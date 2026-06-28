use n_mcconsole_core::command::{Command, Output};
use n_mcconsole_core::executor::{Executor, Streaming};
use std::io;
use std::io::{BufRead, BufReader, Write};

pub struct SshExecutor {
    pub host: String,
    pub opts: Vec<String>,
}

impl SshExecutor {
    pub fn base(&self) -> std::process::Command {
        let mut c = std::process::Command::new("ssh");
        c.args(&self.opts).arg(&self.host);
        c
    }

    pub fn remote_cmdline(cmd: &Command) -> String {
        std::iter::once(&cmd.program)
            .chain(cmd.args.iter())
            .map(|s| shell_quote(s))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Executor for SshExecutor {
    fn run(&self, cmd: &Command) -> io::Result<Output> {
        let o = self.base().arg(Self::remote_cmdline(cmd)).output()?;
        Ok(Output {
            success: o.status.success(),
            stdout: o.stdout,
            stderr: o.stderr,
        })
    }

    fn run_stdin(&self, cmd: &Command, data: &[u8]) -> io::Result<Output> {
        let mut child = self
            .base()
            .arg(Self::remote_cmdline(cmd))
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        child.stdin.take().unwrap().write_all(data)?;
        let o = child.wait_with_output()?;
        Ok(Output {
            success: o.status.success(),
            stdout: o.stdout,
            stderr: o.stderr,
        })
    }

    fn spawn_streaming(&self, cmd: &Command) -> io::Result<Streaming> {
        let mut child = self
            .base()
            .arg(Self::remote_cmdline(cmd))
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        let out = child.stdout.take().expect("piped stdout");
        Ok(Streaming {
            lines: Box::new(BufReader::new(out).lines()),
            killer: Some(child),
        })
    }

    fn read_file(&self, path: &str) -> io::Result<Vec<u8>> {
        let o = self
            .base()
            .arg(Self::remote_cmdline(&Command::new("cat").arg(path)))
            .output()?;
        if o.status.success() {
            Ok(o.stdout)
        } else {
            Err(io::Error::other("remote cat failed"))
        }
    }

    fn write_file(&self, path: &str, data: &[u8]) -> io::Result<()> {
        let mut child = self
            .base()
            .arg(format!("cat > {}", shell_quote(path)))
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        child.stdin.take().unwrap().write_all(data)?;
        if child.wait()?.success() {
            Ok(())
        } else {
            Err(io::Error::other("remote write failed"))
        }
    }

    fn list_dir(&self, path: &str) -> io::Result<Vec<String>> {
        let o = self
            .base()
            .arg(Self::remote_cmdline(
                &Command::new("ls").arg("-1A").arg(path),
            ))
            .output()?;
        Ok(String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect())
    }
}

fn shell_quote(s: &str) -> String {
    let safe = !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"@%+=:,./-_".contains(&b));
    if safe {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}
