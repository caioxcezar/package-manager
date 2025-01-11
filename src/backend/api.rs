use anyhow::{Context, Result};
use flate2::bufread::GzDecoder;
use gtk::{prelude::TextBufferExt, TextBuffer};
use serde::de::DeserializeOwned;
use std::io::BufReader;
use tar::Archive;

pub fn get<T: DeserializeOwned>(url: &str) -> Result<T> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .header(reqwest::header::ACCEPT, "*/*")
        .header(reqwest::header::USER_AGENT, "PackageManager/1.0.0")
        .send()?
        .json::<T>()?;

    Ok(resp)
}

pub fn get_str(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .header(reqwest::header::ACCEPT, "*/*")
        .header(reqwest::header::USER_AGENT, "PackageManager/1.0.0")
        .send()?;

    let resp = resp.text()?;

    Ok(resp)
}

pub fn download_and_extract(url: String, file_path: String, text_buffer: TextBuffer) -> Result<()> {
    let mut text_iter = text_buffer.end_iter();

    text_buffer.insert(&mut text_iter, &"Starting request...\n".to_string());
    let response = reqwest::blocking::get(url)?;

    text_buffer.insert(&mut text_iter, &"Start downloading...\n".to_string());
    let content_br = BufReader::new(response);
    let tarfile = GzDecoder::new(content_br);
    let mut archive = Archive::new(tarfile);

    text_buffer.insert(
        &mut text_iter,
        &"Downloading and unpacking...\n".to_string(),
    );
    archive
        .unpack(file_path)
        .context("Failed to unpack the extracted file")?;

    text_buffer.insert(&mut text_iter, &"Finished. \n".to_string());

    Ok(())
}
