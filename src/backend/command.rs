use anyhow::{anyhow, Context, Result};
use secstr::SecVec;
use std::io::{BufRead, BufReader, Lines};
use std::process::{Child, ChildStdout, Command, Stdio};

fn build_command(command: String, root_pass: Option<SecVec<u8>>) -> Result<Command> {
    let cmd = if cfg!(windows) {
        let mut cmd = Command::new("powershell");
        cmd.args(["-c", &command]);
        cmd
    } else if let Some(password) = root_pass {
        let password = String::from_utf8(password.unsecure().to_vec())?;
        let mut cmd = Command::new("sh");
        cmd.args(["-c", &format!("echo '{}' | sudo -S {}", password, command)]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", &command]);
        cmd
    };
    Ok(cmd)
}

pub fn run(command: &str) -> Result<String> {
    let program = if cfg!(windows) { "powershell" } else { "sh" };
    let output = Command::new(program).arg("-c").arg(command).output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
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
    pub fn new(command: String, root_pass: Option<SecVec<u8>>) -> Result<Self> {
        let mut child = build_command(command, root_pass)?
            // .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        // let mut stdin = child.stdin.take().context("Failed to open stdin")?;
        // if let Some(pass) = root_pass {
        //     stdin.write_all(pass.unsecure())?;
        //     stdin.write_all("\n".as_bytes())?;
        // }

        let stdout = child.stdout.take().context("Failed to run command")?;
        let stdout_reader = BufReader::new(stdout);
        let lines = stdout_reader.lines();

        Ok(CommandStream { child, lines })
    }

    pub fn close(&mut self) -> bool {
        match self.child.wait() {
            Ok(value) => value.success(),
            _ => false,
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
