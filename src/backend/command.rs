use gtk::glib;
use gtk::prelude::TextBufferExt;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};
use std::thread::{self, JoinHandle};

pub fn run(command: &str) -> Result<String, Error> {
    let program = if cfg!(windows) { "powershell" } else { "sh" };
    let output = Command::new(program)
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

pub fn run_stream(command: String, text_buffer: &gtk::TextBuffer) -> JoinHandle<bool> {
    let program = if cfg!(windows) { "powershell" } else { "sh" };
    let (sender, receiver) = async_channel::unbounded();
    let txt_buffer = text_buffer.clone();
    let join_handle = thread::spawn(move || {
        let mut cmd = Command::new(program)
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = cmd.stdout.as_mut().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();
        for line in stdout_lines {
            let _ = sender.send_blocking(format!("{}\n", line.unwrap()));
        }
        match cmd.wait() {
            Ok(value) => value.success(),
            _ => false,
        }
    });

    glib::MainContext::default().spawn_local(async move {
        while let Ok(text) = receiver.recv().await {
            let mut text_iter = txt_buffer.end_iter();
            txt_buffer.insert(&mut text_iter, &text);
        }
    });

    join_handle
}
