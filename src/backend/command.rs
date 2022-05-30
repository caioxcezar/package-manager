use gtk::glib;
use gtk::traits::TextBufferExt;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};
use std::thread;
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

pub fn run_stream(command: String, text_buffer: &gtk::TextBuffer) {
    let mut cmd = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let txt_buffer = text_buffer.clone();
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    thread::spawn(move || {
        let stdout = cmd.stdout.as_mut().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();
        for line in stdout_lines {
            let _ = tx.send(format!("{}\n", line.unwrap()));
        }
    });

    rx.attach(None, move |text| {
        let mut text_iter = txt_buffer.end_iter();
        txt_buffer.insert(&mut text_iter, &text);
        glib::Continue(true)
    });
}
