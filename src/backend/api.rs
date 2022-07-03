use gtk::glib;
use gtk::traits::TextBufferExt;
use serde::de::DeserializeOwned;
use std::{
    io::Cursor,
    thread::{self, JoinHandle},
};

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

pub fn download(url: String, file_path: String, text_buffer: &gtk::TextBuffer) -> JoinHandle<bool> {
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let txt_buffer = text_buffer.clone();

    let tr = thread::spawn(move || {
        let response = reqwest::blocking::get(url);
        let response = match response {
            Ok(value) => value,
            Err(value) => {
                let _ = tx.send(value.to_string());
                return false;
            }
        };
        let file = std::fs::File::create(file_path);
        let mut file = match file {
            Ok(value) => value,
            Err(value) => {
                let _ = tx.send(value.to_string());
                return false;
            }
        };
        let bytes = response.bytes();
        let bytes = match bytes {
            Ok(value) => value,
            Err(value) => {
                let _ = tx.send(value.to_string());
                return false;
            }
        };
        let mut content = Cursor::new(bytes);
        let copy = std::io::copy(&mut content, &mut file);
        match copy {
            Ok(_) => true,
            Err(value) => {
                let _ = tx.send(value.to_string());
                false
            }
        }
    });

    rx.attach(None, move |text: String| {
        let mut text_iter = txt_buffer.end_iter();
        txt_buffer.insert(&mut text_iter, &text);
        glib::Continue(true)
    });
    tr
}
