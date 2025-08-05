use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use secstr::SecVec;

use crate::backend::{
    command::{self, CommandStream},
    package_object::PackageData,
    provider::ProviderActions,
    utils::pass_2_stdin,
};
#[derive(Clone, Debug)]
pub struct Dnf {
    name: String,
    packages: Vec<PackageData>,
    installed: usize,
    total: usize,
    root_required: bool,
}

impl Default for Dnf {
    fn default() -> Self {
        Dnf {
            name: String::from("Dnf"),
            packages: Vec::new(),
            root_required: true,
            installed: 0,
            total: 0,
        }
    }
}

impl ProviderActions for Dnf {
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
    fn load_packages(&mut self) -> Result<(), anyhow::Error> {
        self.packages.clear();

        let packages = command::run("dnf list --all -q")?;

        let grp_packages = packages
            .split("Available Packages\n")
            .collect::<Vec<&str>>();

        let seperator = Regex::new(r"[\s,]+").expect("Invalid regex");

        for (position, packages) in grp_packages.iter().enumerate() {
            let pkgs = if position == 0 {
                packages.replace("Installed Packages\n", "")
            } else {
                packages.to_string()
            };
            let pkgs = pkgs.par_split('\n').collect::<Vec<&str>>();
            self.packages.append(
                &mut pkgs
                    .par_iter()
                    .filter_map(|package| {
                        let list_package: Vec<&str> = seperator.split(package).collect();
                        if list_package.len() < 2 {
                            return None;
                        }
                        Some(PackageData {
                            repository: String::from(list_package[2].trim()),
                            name: String::from(list_package[0].trim()),
                            qualified_name: String::from(list_package[0].trim()),
                            version: String::from(list_package[1].trim()),
                            installed: position == 0,
                        })
                    })
                    .collect::<Vec<PackageData>>(),
            );
        }

        self.installed = self.packages.par_iter().filter(|&p| p.installed).count();
        self.total = self.packages.len();
        Ok(())
    }
    fn package_info(&self, package: String) -> Result<String> {
        command::run(&format!("dnf info {package}"))
    }
    fn install(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(
            format!("sudo -S dnf install {package} -y"),
            Some(pass_2_stdin(password)?),
        )
    }
    fn remove(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(
            format!("sudo -S dnf remove {package} -y"),
            Some(pass_2_stdin(password)?),
        )
    }
    fn update(&self, password: Option<SecVec<u8>>) -> Result<CommandStream> {
        CommandStream::new(
            "sudo -S dnf update -y".to_string(),
            Some(pass_2_stdin(password)?),
        )
    }
    fn is_available(&self) -> bool {
        let packages = command::run("dnf --version");
        packages.is_ok()
    }
}
