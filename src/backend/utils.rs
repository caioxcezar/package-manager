use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use gtk::glib;
use secstr::SecVec;

use crate::constants::APP_ID;

pub fn system_path() -> Result<PathBuf> {
    let mut path = glib::user_data_dir();
    path.push(APP_ID);
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn open_file(path: PathBuf) -> Result<fs::File> {
    let file = fs::File::open(path)?;
    Ok(file)
}

pub fn pass_2_stdin(password: Option<SecVec<u8>>) -> Result<Vec<String>> {
    let password = String::from_utf8(password.context("Missing password")?.unsecure().to_vec())?;
    Ok([password].to_vec())
}
