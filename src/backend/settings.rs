use anyhow::Result;
use gtk::glib;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::constants::APP_ID;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub fuzzy_search: bool,
}

impl Settings {
    pub fn set_bool(&mut self, prop: &str, value: bool) -> Result<()> {
        match prop {
            "fuzzy_search" => {
                self.fuzzy_search = value;
            }
            &_ => {}
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
    let mut path = glib::user_data_dir();
    path.push(APP_ID);
    std::fs::create_dir_all(&path)?;
    path.push("setting.json");
    Ok(path)
}

pub fn open_file(path: PathBuf) -> Result<fs::File> {
    let file = fs::File::open(path)?;
    Ok(file)
}

pub fn get() -> Result<Settings> {
    let file = open_file(settings_path().expect("Failed to get settings path"))?;
    let settings = serde_json::from_reader(file).expect("Failed to read settings file");
    Ok(settings)
}
