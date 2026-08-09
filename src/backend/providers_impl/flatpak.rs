use std::collections::HashSet;

use anyhow::{Context, Result};
use rayon::prelude::*;
use secstr::SecVec;
use serde::{Deserialize, Serialize};

use crate::{backend::{
    command::{self, CommandStream},
    package_object::PackageData,
    provider::ProviderActions,
}};

#[derive(Clone, Debug)]
pub struct Flatpak {
    name: String,
    packages: Vec<PackageData>,
    installed: usize,
    total: usize,
    root_required: bool,
}

#[derive(Serialize, Deserialize)]
struct FlatpakPackage {
    name: String,
    #[serde(rename = "ref")]
    application_ref: String,
    branch: String,
    version: String,
    origin: String,
    arch: String
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

        let packages: String = command::run("LC_ALL=C flatpak list --columns=name,ref,branch,version,origin,arch -j")?;
        let installed_packages: Vec<FlatpakPackage> = serde_json::from_str(&packages)?;
        let installed_packages: HashSet<&str> = installed_packages.par_iter().map(|f| f.application_ref.as_str()).collect();

        let packages = command::run("LC_ALL=C flatpak remote-ls --columns=name,ref,branch,version,origin,arch -j")?;
        let packages: Vec<FlatpakPackage> = serde_json::from_str(&packages)?;
        self.packages.append(
            &mut packages
                .par_iter()
                .map(|pkg| {
                    PackageData {
                        repository: format!("{} {} {}", pkg.origin, pkg.branch, pkg.arch),
                        name: pkg.name.clone(),
                        qualified_name: format!("{} {}", pkg.origin, pkg.application_ref),
                        version: pkg.version.clone(),
                        installed: installed_packages.contains(pkg.application_ref.as_str()),
                    }
                })
                .collect::<Vec<PackageData>>(),
        );

        self.installed = installed_packages.len();
        self.total = self.packages.len();
        Ok(())
    }
    fn package_info(&self, package: String) -> Result<String> {
        let response = command::run(&format!("flatpak remote-info {package}"))?;
        Ok(response)
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
