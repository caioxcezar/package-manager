use std::thread::JoinHandle;

use gtk::TextBuffer;
use rayon::prelude::*;
use regex::Regex;
use secstr::SecVec;

use crate::backend::{command, package_object::PackageData, provider::Provider};
#[derive(Clone)]
pub struct Dnf {
    name: String,
    packages: Vec<PackageData>,
    installed: usize,
    total: usize,
    root_required: bool,
}

pub fn init() -> Dnf {
    Dnf {
        name: String::from("Dnf"),
        packages: Vec::new(),
        root_required: true,
        installed: 0,
        total: 0,
    }
}

impl Provider for Dnf {
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
    fn load_packages(&mut self) -> Result<(), String> {
        self.packages.clear();

        let packages = command::run("dnf list --all -q");
        let packages = match packages {
            Ok(result) => result,
            Err(err) => return Err(format!("{:?}", err)),
        };

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
    fn package_info(&self, package: &str) -> String {
        command::run(&format!("dnf info {}", package)).unwrap()
    }
    fn install(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool> {
        let pass = String::from_utf8(password.unsecure().to_owned()).unwrap();
        command::run_stream(
            format!("echo '{}' | sudo -S dnf install {} -y", pass, package),
            text_buffer,
        )
    }
    fn remove(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool> {
        let pass = String::from_utf8(password.unsecure().to_owned()).unwrap();
        command::run_stream(
            format!("echo '{}' | sudo -S dnf remove {} -y", pass, package),
            text_buffer,
        )
    }
    fn update(&self, password: &SecVec<u8>, text_buffer: &TextBuffer) -> JoinHandle<bool> {
        let pass = String::from_utf8(password.unsecure().to_owned()).unwrap();
        command::run_stream(
            format!("echo '{}' | sudo -S dnf update -y", pass),
            text_buffer,
        )
    }
}
pub fn is_available() -> bool {
    let packages = command::run("dnf --version");
    packages.is_ok()
}
