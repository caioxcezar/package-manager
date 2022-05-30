use gtk::TextBuffer;
use secstr::SecVec;

use crate::{
    backend::{command, package::Package, provider::Provider},
    messagebox,
};
#[derive(Clone)]
pub struct Pacman {
    pub name: String,
    pub packages: Vec<Package>,
    pub installed: usize,
    pub total: usize,
    pub root_required: bool,
}

pub fn init() -> Pacman {
    let mut provider = Pacman {
        name: String::from("Pacman"),
        packages: Vec::new(),
        root_required: true,
        installed: 0,
        total: 0,
    };
    provider.load_packages();
    provider
}

impl Provider for Pacman {
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
        let packages = command::run(String::from("pacman -Sl"));
        let packages = match packages {
            Ok(result) => result,
            Err(err) => {
                messagebox::error("Erro ao carregar Pacman", &err.to_string()[..]);
                return;
            }
        };
        let packages: Vec<&str> = packages.split('\n').collect();

        for package in packages {
            let list_package: Vec<&str> = package.split(' ').collect();
            if list_package.len() < 2 {
                continue;
            }
            self.packages.push(Package {
                provider: String::from("Pacman"),
                repository: String::from(list_package[0]),
                name: String::from(list_package[1]),
                qualified_name: String::from(list_package[1]),
                version: String::from(list_package[2]),
                is_installed: list_package.len() == 4,
            });
        }

        self.installed = self.packages.iter().filter(|&p| p.is_installed).count();
        self.total = self.packages.len();
    }
    fn package_info(&self, package: String) -> String {
        command::run(format!("pacman -Si {}", package)).unwrap()
    }
    fn install(&self, password: SecVec<u8>, packages: Vec<String>, text_buffer: &TextBuffer) {
        command::run_stream(
            format!(
                "echo '{}' | sudo -S pacman -Syu {} --noconfirm",
                password.to_string(),
                packages.join(" ")
            ),
            text_buffer,
        )
    }
    fn remove(&self, password: SecVec<u8>, packages: Vec<String>, text_buffer: &TextBuffer) {
        command::run_stream(
            format!(
                "echo '{}' | sudo -S pacman -Rsu {} --noconfirm",
                password.to_string(),
                packages.join(" ")
            ),
            text_buffer,
        )
    }
    fn update(&self, password: SecVec<u8>, text_buffer: &TextBuffer) {
        command::run_stream(
            format!(
                "echo '{}' | sudo -S pacman -Syy --noconfirm",
                password.to_string()
            ),
            text_buffer,
        )
    }
}
pub fn is_available() -> bool {
    let packages = command::run(String::from("pacman --version"));
    match packages {
        Ok(_) => true,
        Err(_) => false,
    }
}
