use flate2::bufread::GzDecoder;
use gtk::{glib, prelude::TextBufferExt};
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
    let (sender, receiver) = async_channel::unbounded();
    let txt_buffer = text_buffer.clone();

    let join_handle = thread::spawn(move || {
        let _ = sender.send_blocking("Starting request...\n".to_owned());
        let response = reqwest::blocking::get(url);
        let response = match response {
            Ok(value) => value,
            Err(value) => {
                let _ = sender.send_blocking(value.to_string());
                return false;
            }
        };
        let _ = sender.send_blocking("Start downloading...\n".to_owned());
        let content_br = BufReader::new(response);
        let tarfile = GzDecoder::new(content_br);
        let mut archive = Archive::new(tarfile);
        let _ = sender.send_blocking("Downloading and unpacking...\n".to_owned());
        archive.unpack(file_path).unwrap();
        let _ = sender.send_blocking("Finished. \n".to_owned());
        true
    });

    glib::MainContext::default().spawn_local(async move {
        while let Ok(text) = receiver.recv().await {
            let mut text_iter = txt_buffer.end_iter();
            txt_buffer.insert(&mut text_iter, &text);
        }
    });

    join_handle
}
