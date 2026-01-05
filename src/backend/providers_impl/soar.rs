use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use secstr::SecVec;

use crate::backend::{
    command::{self, CommandStream},
    package_object::PackageData,
    provider::ProviderActions,
};

#[derive(Clone, Debug)]
pub struct Soar {
    name: String,
    packages: Vec<PackageData>,
    installed: usize,
    total: usize,
    root_required: bool,
}

impl Default for Soar {
    fn default() -> Self {
        Soar {
            name: String::from("Soar"),
            packages: Vec::new(),
            root_required: false,
            installed: 0,
            total: 0,
        }
    }
}

impl ProviderActions for Soar {
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

        let regex_colors = Regex::new(r"\x1b\[[0-9;]*[0-9]m").expect("Invalid regex");
        let regex_installed = Regex::new(r"\[[✓?○]\]").expect("Invalid regex");

        let packages = command::run("soar list")?;
        let packages: Vec<&str> = packages.split('\n').collect();

        self.packages = packages
            .iter()
            .filter_map(|_package| {
                let package: String = regex_colors.replace_all(_package, "").chars().collect();
                let list_package: Vec<&str> = package.split(" | ").collect();
                if list_package.len() == 3 && regex_installed.is_match(list_package[0]) {
                    let installed = list_package[0].contains("[✓]");
                    let qualified_name: String = regex_installed
                        .replace_all(list_package[0], "")
                        .chars()
                        .collect();
                    let name = qualified_name.split("#").collect::<Vec<&str>>()[0].to_string();

                    return Some(PackageData {
                        repository: String::from(list_package[2]),
                        version: String::from(list_package[1]),
                        qualified_name,
                        installed,
                        name,
                    });
                }
                None
            })
            .collect::<Vec<PackageData>>();

        self.installed = self.packages.par_iter().filter(|&p| p.installed).count();
        self.total = self.packages.len();
        Ok(())
    }
    fn package_info(&self, package: String) -> Result<String> {
        let regex_colors = Regex::new(r"\x1b\[[0-9;]*[0-9]m").expect("Invalid regex");

        let result = command::run(&format!("soar query {package}"))?;
        Ok(regex_colors.replace_all(&result, "").chars().collect())
    }
    fn install(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(format!("soar install {package}"), None)
    }
    fn remove(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(format!("soar remove {package}"), None)
    }
    fn update(&self, _: Option<SecVec<u8>>) -> Result<CommandStream> {
        CommandStream::new("soar update".to_string(), None)
    }
    fn is_available(&self) -> bool {
        let result = command::run("soar --version");
        result.is_ok()
    }
}
