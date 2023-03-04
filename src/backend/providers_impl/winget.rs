use gtk::TextBuffer;
use rayon::prelude::*;
use secstr::SecVec;
use std::thread::JoinHandle;

use crate::backend::{command, package::Package, provider::Provider, utils::split_utf8};
#[derive(Clone)]
pub struct Winget {
    name: String,
    packages: Vec<Package>,
    installed: usize,
    total: usize,
    root_required: bool,
}

pub fn init() -> Winget {
    Winget {
        name: String::from("Winget"),
        packages: Vec::new(),
        root_required: false,
        installed: 0,
        total: 0,
    }
}

impl Provider for Winget {
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
        let packages = command::run("winget list");
        let packages = match packages {
            Ok(result) => result,
            Err(err) => return Err(format!("{:?}", err)),
        };

        let name = packages.find("Name").unwrap();
        let id = packages.find("Id").unwrap() - name;
        let version = packages.find("Version").unwrap() - name;
        let available = packages.find("Available").unwrap() - name;
        let source = packages.find("Source").unwrap() - name;

        let packages: Vec<&str> = packages.split('\n').collect();
        let installed_packages: Vec<Package> = packages
            .par_iter()
            .filter_map(|package| {
                if package.contains("Name") || package.contains("-----") || package.len() < source {
                    return None;
                }
                let repository = if package[source..].trim() == "" {
                    "local"
                } else {
                    &package[source..]
                };

                Some(Package {
                    provider: String::from("Winget"),
                    repository: repository.to_owned(),
                    name: split_utf8(package, 0, id),
                    qualified_name: split_utf8(package, id, version),
                    version: split_utf8(package, version, available),
                    is_installed: true,
                })
            })
            .collect();

        let packages = command::run("winget search -q `\"`\"");
        let packages = match packages {
            Ok(result) => result,
            Err(err) => return Err(format!("{:?}", err)),
        };

        let name = packages.find("Name").unwrap();
        let id = packages.find("Id").unwrap() - name;
        let version = packages.find("Version").unwrap() - name;
        let _match = packages.find("Match").unwrap() - name;
        let source = packages.find("Source").unwrap() - name;

        let packages: Vec<&str> = packages.split('\n').collect();
        self.packages = packages
            .par_iter()
            .filter_map(|package| {
                if package.contains("Name") || package.contains("-----") || package.len() < source {
                    return None;
                }
                Some(Package {
                    provider: String::from("Winget"),
                    repository: package[source..].to_owned(),
                    name: split_utf8(package, 0, id),
                    qualified_name: split_utf8(package, id, version),
                    version: split_utf8(package, version, _match),
                    is_installed: installed_packages
                        .par_iter()
                        .any(|f| f.qualified_name == split_utf8(package, id, version)),
                })
            })
            .collect();

        self.installed = installed_packages.len();
        self.total = self.packages.len();
        Ok(())
    }
    fn package_info(&self, package: &str) -> String {
        command::run(&format!("winget show {}", package)).unwrap()
    }
    fn install(&self, _: &SecVec<u8>, package: &str, text_buffer: &TextBuffer) -> JoinHandle<bool> {
        command::run_stream(format!("winget install {}", package), text_buffer)
    }
    fn remove(&self, _: &SecVec<u8>, package: &str, text_buffer: &TextBuffer) -> JoinHandle<bool> {
        command::run_stream(format!("winget uninstall {}", package), text_buffer)
    }
    fn update(&self, _: &SecVec<u8>, text_buffer: &TextBuffer) -> JoinHandle<bool> {
        command::run_stream("winget upgrade".to_owned(), text_buffer)
    }
}
pub fn is_available() -> bool {
    let packages = command::run("winget --version");
    match packages {
        Ok(_) => true,
        Err(_) => false,
    }
}
