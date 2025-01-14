use anyhow::{anyhow, Context, Result};
use std::io::{BufRead, BufReader, Lines, Read, Write};
use std::process::{Child, ChildStdout, Command, Stdio};

fn build_command(command: &str) -> Result<Command> {
    let cmd = if cfg!(windows) {
        let mut cmd = Command::new("powershell");
        cmd.args(["-c", &command]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", &command]);
        cmd
    };
    Ok(cmd)
}

pub fn run(command: &str) -> Result<String> {
    let output = build_command(command)?.output()?;

    if output.status.success() {
        let msg = String::from_utf8(output.stdout)?;
        Ok(msg)
    } else {
        let err_msg = String::from_utf8(output.stderr)?;
        Err(anyhow!(err_msg))
    }
}

pub struct CommandStream {
    child: Child,
    lines: Lines<BufReader<ChildStdout>>,
}
impl CommandStream {
    pub fn new(command: String, stdin: Option<Vec<String>>) -> Result<Self> {
        let mut child = build_command(&command)?
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(inputs) = stdin {
            let mut stdin = child.stdin.take().context("Failed to run command")?;
            for input in inputs {
                stdin.write_all(format!("{}\r\n", input).as_bytes())?;
            }
        }

        let stdout = child.stdout.take().context("Failed to run command")?;
        let stdout_reader = BufReader::new(stdout);
        let lines = stdout_reader.lines();

        Ok(CommandStream { child, lines })
    }

    pub fn close(&mut self) -> Result<()> {
        let result = self.child.wait()?;
        if result.success() {
            Ok(())
        } else {
            let stdout = self.child.stderr.take().context("Failed to run command")?;
            let mut stdout_reader = BufReader::new(stdout);
            let mut buf = "".to_string();
            stdout_reader.read_to_string(&mut buf)?;
            Err(anyhow!(buf))
        }
    }
}

impl Iterator for CommandStream {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            Some(result) => result.ok(),
            _ => None,
        }
    }
}
