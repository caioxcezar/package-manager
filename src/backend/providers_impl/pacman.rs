use std::fs;

use alpm::{Alpm, SigLevel};
use anyhow::Result;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use secstr::SecVec;

use crate::backend::{
    command::{self, CommandStream},
    package_object::PackageData,
    provider::ProviderActions,
    utils::pass_2_stdin,
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

        let handle = Alpm::new("/", "/var/lib/pacman")?;
        let dbs = fs::read_dir("/var/lib/pacman/sync/")?
            .filter_map(|dir| dir.ok())
            .filter_map(|dir| dir.file_name().into_string().ok())
            .filter_map(|name| {
                if name.ends_with(".db") {
                    Some(name.trim_end_matches(".db").to_string())
                } else {
                    None
                }
            });

        for db_name in dbs {
            let db = handle.register_syncdb(db_name.clone(), SigLevel::NONE)?;
            for pkg in db.pkgs() {
                let pkg_name = pkg.name();
                let pkg_version = pkg.version().to_string();
                let local_entry = handle.localdb().pkg(pkg_name).ok();
                let installed = local_entry.is_some() && local_entry.as_ref().unwrap().version().eq(&pkg_version);

                self.packages.push(PackageData {
                    repository: db_name.clone(),
                    name: pkg_name.to_string(),
                    qualified_name: pkg_name.to_string(),
                    version: pkg_version,
                    installed,
                })
            }
        }

        self.installed = self.packages.par_iter().filter(|p| p.installed).count();
        self.total = self.packages.len();
        Ok(())
    }
    fn package_info(&self, package: String) -> Result<String> {
        command::run(&format!("pacman -Si {package}"))
    }
    fn install(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(
            format!("sudo -S pacman -Syu {package} --noconfirm"),
            Some(pass_2_stdin(password)?),
        )
    }
    fn remove(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        CommandStream::new(
            format!("sudo -S pacman -Runs {package} --noconfirm"),
            Some(pass_2_stdin(password)?),
        )
    }
    fn update(&self, password: Option<SecVec<u8>>) -> Result<CommandStream> {
        CommandStream::new(
            "sudo -S pacman -Syu --noconfirm".to_string(),
            Some(pass_2_stdin(password)?),
        )
    }
    fn is_available(&self) -> bool {
        let packages = command::run("pacman --version");
        packages.is_ok()
    }
}

