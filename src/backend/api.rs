use flate2::bufread::GzDecoder;
use gtk::glib;
use gtk::traits::TextBufferExt;
use serde::de::DeserializeOwned;
use std::{
    io::BufReader,
    thread::{self, JoinHandle},
};
use tar::Archive;

pub fn get<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .header(reqwest::header::ACCEPT, "*/*")
        .header(reqwest::header::USER_AGENT, "PackageManager/1.0.0")
        .send();

    let resp = match resp {
        Ok(value) => value,
        Err(value) => {
            return Err(value.to_string());
        }
    };

    let resp = match resp.json::<T>() {
        Ok(value) => value,
        Err(value) => {
            return Err(value.to_string());
        }
    };

    Ok(resp)
}

pub fn get_str(url: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .header(reqwest::header::ACCEPT, "*/*")
        .header(reqwest::header::USER_AGENT, "PackageManager/1.0.0")
        .send();

    let resp = match resp {
        Ok(value) => value,
        Err(value) => {
            return Err(value.to_string());
        }
    };

    let resp = match resp.text() {
        Ok(value) => value,
        Err(value) => {
            return Err(value.to_string());
        }
    };

    Ok(resp)
}

pub fn download_and_extract(
    url: String,
    file_path: String,
    text_buffer: &gtk::TextBuffer,
) -> JoinHandle<bool> {
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let txt_buffer = text_buffer.clone();

    let tr = thread::spawn(move || {
        let _ = tx.send("Starting request...\n".to_owned());
        let response = reqwest::blocking::get(url);
        let response = match response {
            Ok(value) => value,
            Err(value) => {
                let _ = tx.send(value.to_string());
                return false;
            }
        };
        let _ = tx.send("Start downloading...\n".to_owned());
        let content_br = BufReader::new(response);
        let tarfile = GzDecoder::new(content_br);
        let mut archive = Archive::new(tarfile);
        let _ = tx.send("Downloading and unpacking...\n".to_owned());
        archive.unpack(file_path).unwrap();
        let _ = tx.send("Finished. \n".to_owned());
        true
    });

    rx.attach(None, move |text: String| {
        let mut text_iter = txt_buffer.end_iter();
        txt_buffer.insert(&mut text_iter, &text);
        glib::Continue(true)
    });
    tr
}
