use alpm::Alpm;
use anyhow::Result;
use flate2::read::GzDecoder;
use secstr::SecVec;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufReader, Read},
    ops::Sub,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use crate::backend::{
    command::{self, CommandStream},
    package_object::PackageData,
    provider::ProviderActions,
    utils::{self, pass_2_stdin},
};
#[derive(Clone, Debug)]
pub struct Paru {
    pub name: String,
    pub packages: Vec<PackageData>,
    pub installed: usize,
    pub total: usize,
    pub root_required: bool,
}

impl Default for Paru {
    fn default() -> Self {
        Paru {
            name: String::from("Paru"),
            packages: Vec::new(),
            root_required: true,
            installed: 0,
            total: 0,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct AurPackageShort {
    name: String,
    version: String,
}

impl ProviderActions for Paru {
    fn installed(&self) -> usize {
        self.installed
    }
    fn total(&self) -> usize {
        self.total
    }
    fn is_root_required(&self) -> bool {
        self.root_required
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn packages(&self) -> Vec<PackageData> {
        self.packages.clone()
    }
    fn load_packages(&mut self) -> Result<()> {
        self.packages.clear();

        let handle = Alpm::new("/", "/var/lib/pacman")?;

        self.packages = get_json_packages()?
            .iter()
            .map(|pkg| PackageData {
                repository: "AUR".to_string(),
                name: pkg.name.to_string(),
                qualified_name: pkg.name.to_string(),
                version: pkg.version.to_string(),
                installed: handle.localdb().pkg(pkg.name.to_string()).is_ok(),
            })
            .collect();

        self.installed = handle.localdb().pkgs().len();
        self.total = self.packages.len();
        Ok(())
    }
    fn package_info(&self, package: String) -> Result<String> {
        command::run(&format!("paru -Si {package}"))
    }
    fn install(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(
            format!("sudo -S su && paru -Syu {package} --noconfirm --sudoflags -S --sudoloop"),
            Some(pass_2_stdin(password)?),
        )
    }
    fn remove(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(
            format!("sudo -S su && paru -Runs {package} --noconfirm --sudoflags -S --sudoloop"),
            Some(pass_2_stdin(password)?),
        )
    }
    fn update(&self, password: Option<SecVec<u8>>) -> Result<CommandStream> {
        CommandStream::new(
            "paru -Syu --noconfirm --sudoflags -S --sudoloop".to_string(),
            Some(pass_2_stdin(password)?),
        )
    }
    fn is_available(&self) -> bool {
        let packages = command::run("paru --version");
        packages.is_ok()
    }
}

fn json_path() -> Result<PathBuf> {
    let mut path = utils::system_path()?;
    path.push("aur_packages.json");
    Ok(path)
}

fn get_json_packages() -> Result<Vec<AurPackageShort>> {
    let path = json_path()?;
    let exists = fs::exists(&path)?;
    if exists {
        let created = fs::File::open(&path)?.metadata()?.created()?;
        let yesterday = SystemTime::now().sub(Duration::from_hours(24));

        if created > yesterday {
            let file = fs::File::open(&path)?;
            let reader = BufReader::new(file);
            return Ok(serde_json::from_reader(reader)?);
        }
    }
    download_json()
}

fn download_json() -> Result<Vec<AurPackageShort>> {
    let response = reqwest::blocking::get("https://aur.archlinux.org/packages-meta-v1.json.gz")?;

    let mut decoder = GzDecoder::new(response);
    let mut json_string = String::new();
    decoder.read_to_string(&mut json_string)?;

    let list = serde_json::from_str::<Vec<AurPackageShort>>(&json_string)?;

    let path = json_path()?;
    let file = fs::File::create(path)?;
    serde_json::to_writer(file, &list)?;

    Ok(list)
}
