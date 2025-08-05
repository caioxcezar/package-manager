use anyhow::Result;
use rayon::prelude::*;
use secstr::SecVec;

use crate::backend::{
    command::{self, CommandStream},
    package_object::PackageData,
    provider::ProviderActions,
    utils::pass_2_stdin,
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
        let packages = command::run("paru -Sl")?;
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
