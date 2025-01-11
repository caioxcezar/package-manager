use anyhow::Result;
use rayon::prelude::*;
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

        let packages = command::run("flatpak list")?;
        let installed_packages = packages
            .split('\n')
            .filter(|e| {
                let x = e.split('\t').collect::<Vec<&str>>();
                x.len().gt(&1)
            })
            .collect::<Vec<&str>>();

        let remotes = command::run("flatpak remotes")?;
        let remotes = remotes.split('\n').collect::<Vec<&str>>();

        for str_remote in remotes {
            let arr_remote: Vec<&str> = str_remote.split('\t').collect();
            if arr_remote[0] == "Name" || arr_remote[0].trim() == "" {
                continue;
            }
            let packages = command::run(&format!("{} {}", "flatpak remote-ls", arr_remote[0]))?;
            let packages = packages.split('\n').collect::<Vec<&str>>();
            self.packages.append(
                &mut packages
                    .par_iter()
                    .filter_map(|str_package| {
                        let arr_package = str_package.split('\t').collect::<Vec<&str>>();
                        if arr_package.len() < 2 {
                            return None;
                        }
                        Some(PackageData {
                            repository: String::from(arr_remote[0]),
                            name: String::from(arr_package[0]),
                            qualified_name: String::from(arr_package[1]),
                            version: String::from(arr_package[2]),
                            installed: installed_packages
                                .par_iter()
                                .any(|f| f.contains(arr_package[1])),
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
        let response = command::run(&format!("flatpak search {}", package))?;
        Ok(response.replace('\t', "\n"))
    }
    fn install(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(format!("flatpak install {} -y", package), None)
    }
    fn remove(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(format!("flatpak remove {} -y", package), None)
    }
    fn update(&self, _: Option<SecVec<u8>>) -> Result<CommandStream> {
        CommandStream::new("flatpak update -y".to_owned(), None)
    }
    fn is_available(&self) -> bool {
        let packages = command::run("flatpak --version");
        packages.is_ok()
    }
}
