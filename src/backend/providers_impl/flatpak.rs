use gtk::TextBuffer;
use secstr::SecVec;

use crate::{
    backend::{command, package::Package, provider::Provider},
    messagebox,
};
#[derive(Clone)]
pub struct Flatpak {
    pub name: String,
    pub packages: Vec<Package>,
    pub installed: usize,
    pub total: usize,
    pub root_required: bool,
}

pub fn init() -> Flatpak {
    let mut provider = Flatpak {
        name: String::from("Flatpak"),
        packages: Vec::new(),
        root_required: true,
        installed: 0,
        total: 0,
    };
    provider.load_packages();
    provider
}

impl Provider for Flatpak {
    fn is_root_required(&self) -> bool {
        self.root_required.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_packages(&self) -> Vec<Package> {
        self.packages.clone()
    }
    fn load_packages(&mut self) {
        self.packages.clear();
        let remotes = command::run(String::from("flatpak remotes"));
        let remotes = match remotes {
            Ok(result) => result,
            Err(err) => {
                messagebox::error("Erro ao carregar Flatpak", &err.to_string()[..]);
                return;
            }
        };
        let remotes: Vec<&str> = remotes.split('\n').collect();

        for str_remote in remotes {
            let arr_remote: Vec<&str> = str_remote.split('\t').collect();
            if arr_remote[0] == "Name" {
                continue;
            }
            let packages = command::run(format!("{} {}", "flatpak remote-ls", arr_remote[0]));
            let packages = match packages {
                Ok(result) => result,
                Err(err) => {
                    messagebox::error("Erro ao carregar Flatpak", &err.to_string()[..]);
                    return;
                }
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

            let packages = command::run(String::from("flatpak list"));
            let packages = match packages {
                Ok(result) => result,
                Err(err) => {
                    messagebox::error("Erro ao carregar Flatpak", &err.to_string()[..]);
                    return;
                }
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
    }
    fn package_info(&self, package: String) -> String {
        let response = command::run(format!("flatpak search {}", package)).unwrap();
        response.replace("\t", "\n")
    }
    fn install(&self, _: SecVec<u8>, packages: Vec<String>, text_buffer: &TextBuffer) {
        command::run_stream(
            format!("flatpak install {} -y", packages.join(" ")),
            text_buffer,
        )
    }
    fn remove(&self, _: SecVec<u8>, packages: Vec<String>, text_buffer: &TextBuffer) {
        command::run_stream(
            format!("flatpak remove {} -y", packages.join(" ")),
            text_buffer,
        )
    }
    fn update(&self, _: SecVec<u8>, text_buffer: &TextBuffer) {
        command::run_stream("flatpak update -y".to_owned(), text_buffer)
    }
}
pub fn is_available() -> bool {
    let packages = command::run(String::from("flatpak --version"));
    match packages {
        Ok(_) => true,
        Err(_) => false,
    }
}
