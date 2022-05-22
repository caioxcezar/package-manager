use crate::{
    backend::{command, package::Package, provider::Provider},
    messagebox,
};

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
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn load_packages(&mut self) {
        self.packages.clear();
        let remotes = command::run(String::from("flatpak remotes"));
        let remotes = match remotes {
            Ok(result) => result,
            Err(err) => {
                messagebox::show(&err.to_string()[..]);
                String::from("")
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
                    messagebox::show(&err.to_string()[..]);
                    String::from("")
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
        }
    }
    fn install(&self, packages: Vec<Package>, output: String, error: String) -> u64 {
        32
    }
    fn remove(&self, packages: Vec<Package>, output: String, error: String) -> u64 {
        32
    }
    fn update(&self, packages: Vec<Package>, output: String, error: String) -> u64 {
        43
    }
}
pub fn is_available() -> bool {
        let packages = command::run(String::from("flatpak --version"));
        match packages {
            Ok(_) => true,
            Err(_) => false
        }
}
