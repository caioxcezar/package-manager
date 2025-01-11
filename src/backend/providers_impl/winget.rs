use anyhow::{Context, Result};
use rayon::prelude::*;
use regex::Regex;
use secstr::SecVec;

use crate::backend::{
    command::{self, CommandStream},
    package_object::PackageData,
    provider::ProviderActions,
    utils::split_utf8,
};

#[derive(Clone, Debug)]
pub struct Winget {
    name: String,
    packages: Vec<PackageData>,
    installed: usize,
    total: usize,
    root_required: bool,
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
        let regex = Regex::new(r"[^\x00-\x7F]").context("Failed to create Regex")?;
        let packages = command::run("winget list --verbose")?;

        let name = packages.find("Name").context("Failed to find name index")?;
        let id = packages.find("Id").context("Failed to find id index")? - name;
        let version = packages
            .find("Version")
            .context("Failed to find version index")?
            - name;
        let source = packages
            .find("Source")
            .context("Failed to find source index")?
            - name;

        let packages: Vec<&str> = packages.split('\n').collect();
        let installed_packages: Vec<PackageData> = packages
            .par_iter()
            .filter_map(|package| {
                if package.contains("Name") || package.contains("-----") || package.len() < source {
                    return None;
                }
                let name = split_utf8(package, 0, id);
                if regex.is_match(&name) {
                    return None;
                }
                let repository = if package[source..].trim() == "" {
                    "local"
                } else {
                    &package[source..]
                };

                Some(PackageData {
                    repository: repository.to_owned(),
                    name,
                    qualified_name: split_utf8(package, id, version),
                    version: split_utf8(package, version, source),
                    installed: true,
                })
            })
            .collect();

        let packages = command::run("winget search -q `\"`\" --verbose")?;

        let name = packages.find("Name").context("Failed to find name")?;
        let id = packages.find("Id").context("Failed to find id")? - name;
        let version = packages.find("Version").context("Failed to find version")? - name;
        let _match = packages.find("Match").context("Failed to find match")? - name;
        let source = packages.find("Source").context("Failed to find source")? - name;

        let packages: Vec<&str> = packages.split('\n').collect();
        self.packages = packages
            .par_iter()
            .filter_map(|package| {
                if package.contains("Name") || package.contains("-----") || package.len() < source {
                    return None;
                }
                let name = split_utf8(package, 0, id);
                if regex.is_match(&name) {
                    return None;
                }
                Some(PackageData {
                    repository: package[source..].to_owned(),
                    name,
                    qualified_name: split_utf8(package, id, version),
                    version: split_utf8(package, version, _match),
                    installed: installed_packages
                        .par_iter()
                        .any(|f| f.qualified_name == split_utf8(package, id, version)),
                })
            })
            .collect();

        self.installed = installed_packages.len();
        self.total = self.packages.len();
        Ok(())
    }
    fn package_info(&self, package: String) -> Result<String> {
        command::run(&format!("winget show {}", package))
    }
    fn install(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(format!("winget install {}", package), None)
    }
    fn remove(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(format!("winget uninstall {}", package), None)
    }
    fn update(&self, _: Option<SecVec<u8>>) -> Result<CommandStream> {
        CommandStream::new("winget upgrade -h --all".to_owned(), None)
    }
    fn is_available(&self) -> bool {
        let packages = command::run("winget --version");
        packages.is_ok()
    }
}
