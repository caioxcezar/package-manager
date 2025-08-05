use anyhow::Result;
use serde::de::DeserializeOwned;

use super::command::CommandStream;

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

pub fn download_and_extract(url: String, file_path: String) -> Result<CommandStream> {
    let command = format!(
        "wget {url} -O /tmp/proton-ge.tar.gz &> /dev/stdout && tar -xvzf /tmp/proton-ge.tar.gz -C {file_path}"
    );
    CommandStream::new(command, None)
}
