use std::thread::JoinHandle;

use gtk::TextBuffer;
use secstr::SecVec;

use crate::backend::{command, package::Package, provider::Provider};
#[derive(Clone)]
pub struct Flatpak {
    name: String,
    packages: Vec<Package>,
    installed: usize,
    total: usize,
    root_required: bool,
}

pub fn init() -> Flatpak {
    Flatpak {
        name: String::from("Flatpak"),
        packages: Vec::new(),
        root_required: false,
        installed: 0,
        total: 0,
    }
}

impl Provider for Flatpak {
    fn installed(&self) -> usize {
        self.installed
    }
    fn total(&self) -> usize {
        self.total
    }
    fn is_root_required(&self) -> bool {
        self.root_required.clone()
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn packages(&self) -> Vec<Package> {
        self.packages.clone()
    }
    fn load_packages(&mut self) -> Result<(), String> {
        self.packages.clear();
        let remotes = command::run("flatpak remotes");
        let remotes = match remotes {
            Ok(result) => result,
            Err(err) => return Err(format!("{:?}", err)),
        };
        let remotes: Vec<&str> = remotes.split('\n').collect();

        for str_remote in remotes {
            let arr_remote: Vec<&str> = str_remote.split('\t').collect();
            if arr_remote[0] == "Name" || arr_remote[0].trim() == "" {
                continue;
            }
            let packages = command::run(&format!("{} {}", "flatpak remote-ls", arr_remote[0]));
            let packages = match packages {
                Ok(result) => result,
                Err(err) => return Err(format!("{:?}", err)),
            };
            let packages: Vec<&str> = packages.split('\n').collect();

            for str_package in packages {
                let arr_package: Vec<&str> = str_package.split('\t').collect();
                if arr_package.len() < 2 {
                    continue;
                }
                self.packages.push(Package {
                    provider: String::from("Flatpak"),
                    repository: String::from(arr_remote[0]),
                    name: String::from(arr_package[0]),
                    qualified_name: String::from(arr_package[1]),
                    version: String::from(arr_package[2]),
                    is_installed: false,
                });
            }

            let packages = command::run("flatpak list");
            let packages = match packages {
                Ok(result) => result,
                Err(err) => return Err(format!("{:?}", err)),
            };
            let installed_package: Vec<&str> = packages
                .split('\n')
                .filter(|e| {
                    let x: Vec<&str> = e.split('\t').collect();
                    x.len().gt(&1)
                })
                .collect();

            for package in &mut self.packages {
                package.is_installed = installed_package
                    .iter()
                    .any(|f| f.contains(&package.qualified_name));
            }

            self.total = self.packages.len();
            self.installed = installed_package.len();
        }
        Ok(())
    }
    fn package_info(&self, package: &str) -> String {
        let response = command::run(&format!("flatpak search {}", package)).unwrap();
        response.replace("\t", "\n")
    }
    fn install(&self, _: &SecVec<u8>, package: &str, text_buffer: &TextBuffer) -> JoinHandle<bool> {
        command::run_stream(format!("flatpak install {} -y", package), text_buffer)
    }
    fn remove(&self, _: &SecVec<u8>, package: &str, text_buffer: &TextBuffer) -> JoinHandle<bool> {
        command::run_stream(format!("flatpak remove {} -y", package), text_buffer)
    }
    fn update(&self, _: &SecVec<u8>, text_buffer: &TextBuffer) -> JoinHandle<bool> {
        command::run_stream("flatpak update -y".to_owned(), text_buffer)
    }
}
pub fn is_available() -> bool {
    let packages = command::run("flatpak --version");
    match packages {
        Ok(_) => true,
        Err(_) => false,
    }
}
