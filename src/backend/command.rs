use std::io::{Error, ErrorKind};
use std::process::{Command, Stdio};

pub fn run(command: String) -> Result<String, Error> {
     let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .expect("failed to execute process");

    if output.status.success() {
        Ok(String::from_utf8(output.stdout).unwrap())
    } else {
        Err(Error::new(
            ErrorKind::Other,
            String::from_utf8(output.stderr).unwrap(),
        ))
    }
}
