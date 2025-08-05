use anyhow::{Context, Result};
use rayon::prelude::*;
use regex::Regex;
use secstr::SecVec;

use crate::backend::{
    command::{self, CommandStream},
    package_object::PackageData,
    provider::ProviderActions,
};
#[derive(Clone, Debug)]
pub struct Flatpak {
    name: String,
    packages: Vec<PackageData>,
    installed: usize,
    total: usize,
    root_required: bool,
}

impl Default for Flatpak {
    fn default() -> Self {
        Flatpak {
            name: String::from("Flatpak"),
            packages: Vec::new(),
            root_required: false,
            installed: 0,
            total: 0,
        }
    }
}

impl ProviderActions for Flatpak {
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

        let packages = command::run("flatpak list --columns=origin,application")?;
        let installed_packages = packages
            .split('\n')
            .filter_map(|e| {
                let columns = e.split('\t').collect::<Vec<&str>>();
                if columns.len().gt(&1) {
                    Some(format!("{} {}", columns[0], columns[1]))
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        let packages = command::run("flatpak remote-ls")?;
        let packages = packages.split('\n').collect::<Vec<&str>>();
        self.packages.append(
            &mut packages
                .par_iter()
                .filter_map(|str_package| {
                    let arr_package = str_package.split('\t').collect::<Vec<&str>>();
                    if arr_package.len() < 6 {
                        return None;
                    }
                    let qualified_name = format!("{} {}", arr_package[5], arr_package[1]);
                    Some(PackageData {
                        repository: String::from(arr_package[5]),
                        name: String::from(arr_package[0]),
                        qualified_name: qualified_name.clone(),
                        version: String::from(arr_package[2]),
                        installed: installed_packages.par_iter().any(|f| f.eq(&qualified_name)),
                    })
                })
                .collect::<Vec<PackageData>>(),
        );
        self.installed = self.packages.par_iter().filter(|&p| p.installed).count();
        self.total = self.packages.len();
        Ok(())
    }
    fn package_info(&self, package: String) -> Result<String> {
        let split = package.split(' ').collect::<Vec<&str>>();
        let remote = split[0];
        let package_name = split[1];
        let re = format!("[^-](\\b{remote}\\b)([^-]|$)");
        let regex = Regex::new(&re).expect("Invalid regex");
        let response = command::run(&format!("flatpak search {package_name}"))?;
        let lines = response
            .split('\n')
            .filter(|value| regex.is_match(value))
            .collect::<Vec<&str>>();
        let info = if lines.is_empty() {
            response
        } else {
            lines.first().context("Package Info not found")?.to_string()
        };
        Ok(info.replace('\t', "\n"))
    }
    fn install(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(
            format!("flatpak install {package} -y --noninteractive"),
            None,
        )
    }
    fn remove(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        let idx_name = package.find(' ').context("Package name not found")?;
        let package_name = package[idx_name..].to_string();
        CommandStream::new(
            format!("flatpak remove {package_name} -y --noninteractive"),
            None,
        )
    }
    fn update(&self, _: Option<SecVec<u8>>) -> Result<CommandStream> {
        CommandStream::new("flatpak update -y --noninteractive".to_owned(), None)
    }
    fn is_available(&self) -> bool {
        let packages = command::run("flatpak --version");
        packages.is_ok()
    }
}
