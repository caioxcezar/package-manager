use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::backend::utils;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub fuzzy_search: bool,
}

impl Settings {
    pub fn set_bool(&mut self, prop: &str, value: bool) -> Result<()> {
        if prop == "fuzzy_search" {
            self.fuzzy_search = value
        }
        self.update_json()?;
        Ok(())
    }

    pub fn update_json(&self) -> Result<()> {
        let path = settings_path()?;
        let file = fs::File::create(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }
}

pub fn settings_path() -> Result<PathBuf> {
    let mut path = utils::system_path()?;
    path.push("setting.json");
    Ok(path)
}

pub fn get() -> Result<Settings> {
    let path = settings_path()?;
    if !fs::exists(&path).unwrap_or(true) {
        Settings::default().update_json()?;
    }
    let file = utils::open_file(path)?;
    let settings = serde_json::from_reader(file).expect("Failed to read settings file");
    Ok(settings)
}
