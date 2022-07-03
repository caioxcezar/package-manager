use std::thread::JoinHandle;

use gtk::TextBuffer;
use secstr::SecVec;

use crate::backend::{command, package::Package, provider::Provider};
#[derive(Clone)]
pub struct Pacman {
    name: String,
    packages: Vec<Package>,
    installed: usize,
    total: usize,
    root_required: bool,
}

pub fn init() -> Pacman {
    Pacman {
        name: String::from("Pacman"),
        packages: Vec::new(),
        root_required: true,
        installed: 0,
        total: 0,
    }
}

impl Provider for Pacman {
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
        let packages = command::run("pacman -Sl");
        let packages = match packages {
            Ok(result) => result,
            Err(err) => return Err(format!("{:?}", err)),
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
        Ok(())
    }
    fn package_info(&self, package: &str) -> String {
        command::run(&format!("pacman -Si {}", package)).unwrap()
    }
    fn install(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool> {
        let password = String::from_utf8(password.unsecure().to_owned()).unwrap();
        let command = format!(
            "echo '{}' | sudo -S pacman -Syu {} --noconfirm",
            password, package
        );
        command::run_stream(command, text_buffer)
    }
    fn remove(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool> {
        let password = String::from_utf8(password.unsecure().to_owned()).unwrap();
        let command = format!(
            "echo '{}' | sudo -S pacman -Rsu {} --noconfirm",
            password.to_string(),
            package
        );
        command::run_stream(command, text_buffer)
    }
    fn update(&self, password: &SecVec<u8>, text_buffer: &TextBuffer) -> JoinHandle<bool> {
        let password = String::from_utf8(password.unsecure().to_owned()).unwrap();
        let command = format!("echo '{}' | sudo -S pacman -Syu --noconfirm", password);
        command::run_stream(command, text_buffer)
    }
}
pub fn is_available() -> bool {
    let packages = command::run("pacman --version");
    match packages {
        Ok(_) => true,
        Err(_) => false,
    }
}
