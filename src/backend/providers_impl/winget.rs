use anyhow::{Context, Result};
use rayon::prelude::*;
use rusqlite::Connection;
use secstr::SecVec;
use serde::{Deserialize, Serialize};

use crate::backend::{
    command::{self, CommandStream},
    package_object::PackageData,
    provider::ProviderActions,
    utils,
};

#[derive(Clone, Debug)]
pub struct Winget {
    name: String,
    packages: Vec<PackageData>,
    installed: usize,
    total: usize,
    root_required: bool,
}

#[derive(Serialize, Deserialize)]
struct WingetJson {
    #[serde(rename = "Sources")]
    sources: Vec<SourceJson>,
}

#[derive(Serialize, Deserialize)]
struct SourceJson {
    #[serde(rename = "Packages")]
    packages: Vec<PackageJson>,
    #[serde(rename = "SourceDetails")]
    source_details: SourceDetailsJson,
}

#[derive(Serialize, Deserialize)]
struct SourceDetailsJson {
    #[serde(rename = "Argument")]
    argument: String,
    #[serde(rename = "Identifier")]
    identifier: String,
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Serialize, Deserialize)]
struct PackageJson {
    #[serde(rename = "PackageIdentifier")]
    package_identifier: String,
    #[serde(rename = "Version")]
    version: String,
}

impl Default for Winget {
    fn default() -> Self {
        Winget {
            name: String::from("Winget"),
            packages: Vec::new(),
            root_required: false,
            installed: 0,
            total: 0,
        }
    }
}

impl ProviderActions for Winget {
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

        let mut path = utils::system_path()?;
        path.push("winget_installed.json");

        command::run(&format!(
            "winget export --include-versions --nowarn -o {}",
            path.to_str().context("Unable to get path")?
        ))?;
        let file = utils::open_file(path)?;
        let winget_json: WingetJson = serde_json::from_reader(file)?;

        let mut installed_packages: Vec<PackageData> = Vec::new();
        for source in winget_json.sources {
            let mut packages: Vec<PackageData> = source
                .packages
                .par_iter()
                .map(|pkg| PackageData {
                    name: "".to_string(),
                    repository: source.source_details.name.clone(),
                    qualified_name: pkg.package_identifier.clone(),
                    version: pkg.version.clone(),
                    installed: true,
                })
                .collect();

            installed_packages.append(&mut packages);
        }

        update_db()?;
        self.packages = list_db(&installed_packages)?;

        self.installed = installed_packages.len();
        self.total = self.packages.len();
        Ok(())
    }
    fn package_info(&self, package: String) -> Result<String> {
        command::run(&format!("winget show {package}"))
    }
    fn install(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(format!("winget install {package}"), None)
    }
    fn remove(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(format!("winget uninstall {package}"), None)
    }
    fn update(&self, _: Option<SecVec<u8>>) -> Result<CommandStream> {
        CommandStream::new("winget upgrade -h --all".to_owned(), None)
    }
    fn is_available(&self) -> bool {
        let packages = command::run("winget --version");
        packages.is_ok()
    }
}

fn update_db() -> Result<()> {
    let response = reqwest::blocking::get("https://cdn.winget.microsoft.com/cache/source.msix")?;
    let bytes = response.bytes()?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let name = file.name();

        if name == "Public/index.db" {
            let mut path = utils::system_path()?;
            path.push("index.db");

            let mut out_file = std::fs::File::create(path.to_str().context("Unable to get path")?)?;
            std::io::copy(&mut file, &mut out_file)?;
            break;
        }
    }
    Ok(())
}

fn list_db(installed_packages: &Vec<PackageData>) -> Result<Vec<PackageData>> {
    let mut path = utils::system_path()?;
    path.push("index.db");

    let conn = Connection::open(path.to_str().context("Unable to get path")?)?;
    let mut stmt = conn.prepare(
        "
            SELECT ids.id, names.name, versions.version
            FROM manifest
            INNER JOIN names
                ON manifest.name = names.rowid
            INNER JOIN ids
                ON manifest.id = ids.rowid
            INNER JOIN versions
                ON versions.rowid = manifest.version
            GROUP BY ids.id
            HAVING MAX(manifest.version) = manifest.version",
    )?;
    let result = stmt
        .query_map([], |row| {
            let qualified_name: String = row.get(0)?;
            let name = row.get(1)?;
            let version = row.get(2)?;

            Ok(PackageData {
                repository: "winget".to_string(),
                qualified_name: qualified_name.clone(),
                name,
                version,
                installed: installed_packages
                    .par_iter()
                    .any(|f| f.qualified_name == qualified_name),
            })
        })?
        .map(|result| result.map_err(anyhow::Error::new))
        .collect();

    result
}
