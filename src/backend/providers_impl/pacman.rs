use anyhow::Result;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use secstr::SecVec;

use crate::backend::{
    command::{self, CommandStream},
    package_object::PackageData,
    provider::ProviderActions,
};
#[derive(Clone, Debug)]
pub struct Pacman {
    name: String,
    packages: Vec<PackageData>,
    installed: usize,
    total: usize,
    root_required: bool,
}

impl Default for Pacman {
    fn default() -> Self {
        Pacman {
            name: String::from("Pacman"),
            packages: Vec::new(),
            root_required: true,
            installed: 0,
            total: 0,
        }
    }
}

impl ProviderActions for Pacman {
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
        let packages = command::run("pacman -Sl")?;
        let packages: Vec<&str> = packages.split('\n').collect();
        self.packages = packages
            .par_iter()
            .filter_map(|package| {
                let list_package: Vec<&str> = package.split(' ').collect();
                if list_package.len() < 2 {
                    return None;
                }
                Some(PackageData {
                    repository: String::from(list_package[0]),
                    name: String::from(list_package[1]),
                    qualified_name: String::from(list_package[1]),
                    version: String::from(list_package[2]),
                    installed: list_package.len() == 4,
                })
            })
            .collect();

        self.installed = self.packages.par_iter().filter(|&p| p.installed).count();
        self.total = self.packages.len();
        Ok(())
    }
    fn package_info(&self, package: String) -> Result<String> {
        command::run(&format!("pacman -Si {}", package))
    }
    fn install(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(format!("pacman -Syu {} --noconfirm", package), password)
    }
    fn remove(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(format!("pacman -Runs {} --noconfirm", package), password)
    }
    fn update(&self, password: Option<SecVec<u8>>) -> Result<CommandStream> {
        CommandStream::new("pacman -Syu --noconfirm".to_string(), password)
    }
    fn is_available(&self) -> bool {
        let packages = command::run("pacman --version");
        packages.is_ok()
    }
}
