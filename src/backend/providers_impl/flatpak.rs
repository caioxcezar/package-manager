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
            .filter_map(|e| {
                let columns = e.split('\t').collect::<Vec<&str>>();
                if columns.len().gt(&1) {
                    Some(format!("{} {}", columns[4], columns[1]))
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
        let response = command::run(&format!("flatpak search {}", package))?;
        Ok(response.replace('\t', "\n"))
    }
    fn install(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(
            format!("flatpak install {} -y --noninteractive", package),
            None,
        )
    }
    fn remove(&self, _: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(
            format!("flatpak remove {} -y --noninteractive", package),
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
