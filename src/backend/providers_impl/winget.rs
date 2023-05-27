use gtk::TextBuffer;
use rayon::prelude::*;
use regex::Regex;
use secstr::SecVec;
use std::thread::JoinHandle;

use crate::backend::{command, package_object::PackageData, provider::Provider, utils::split_utf8};
#[derive(Clone)]
pub struct Winget {
    name: String,
    packages: Vec<PackageData>,
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
        let packages = command::run("winget list --verbose");
        let packages = match packages {
            Ok(result) => result,
            Err(err) => return Err(format!("{:?}", err)),
        };

        let name = packages.find("Name").unwrap();
        let id = packages.find("Id").unwrap() - name;
        let version = packages.find("Version").unwrap() - name;
        let source = packages.find("Source").unwrap() - name;

        let packages: Vec<&str> = packages.split('\n').collect();
        let installed_packages: Vec<PackageData> = packages
            .par_iter()
            .filter_map(|package| {
                if package.contains("Name") || package.contains("-----") || package.len() < source {
                    return None;
                }
                let name = split_utf8(package, 0, id);
                if Regex::new(r"[^\x00-\x7F]").unwrap().is_match(&name) {
                    return None;
                }
                let repository = if package[source..].trim() == "" {
                    "local"
                } else {
                    &package[source..]
                };

                Some(PackageData {
                    repository: repository.to_owned(),
                    name,
                    qualified_name: split_utf8(package, id, version),
                    version: split_utf8(package, version, source),
                    installed: true,
                })
            })
            .collect();

        let packages = command::run("winget search -q `\"`\" --verbose");
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
                let name = split_utf8(package, 0, id);
                if Regex::new(r"[^\x00-\x7F]").unwrap().is_match(&name) {
                    return None;
                }
                Some(PackageData {
                    repository: package[source..].to_owned(),
                    name,
                    qualified_name: split_utf8(package, id, version),
                    version: split_utf8(package, version, _match),
                    installed: installed_packages
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
        command::run_stream("winget upgrade -h --all".to_owned(), text_buffer)
    }
}
pub fn is_available() -> bool {
    let packages = command::run("winget --version");
    packages.is_ok()
}
