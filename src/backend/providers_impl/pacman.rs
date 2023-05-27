use std::thread::JoinHandle;

use gtk::TextBuffer;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use secstr::SecVec;

use crate::backend::{command, package_object::PackageData, provider::Provider};
#[derive(Clone)]
pub struct Pacman {
    name: String,
    packages: Vec<PackageData>,
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
        let packages = command::run("pacman -Sl");
        let packages = match packages {
            Ok(result) => result,
            Err(err) => return Err(format!("{:?}", err)),
        };
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
    fn package_info(&self, package: &str) -> String {
        command::run(&format!("pacman -Si {}", package)).unwrap()
    }
    fn install(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool> {
        let command = command_build(password, format!("pacman -Syu {} --noconfirm", package));
        command::run_stream(command, text_buffer)
    }
    fn remove(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool> {
        let command = command_build(password, format!("pacman -Rsu {} --noconfirm", package));
        command::run_stream(command, text_buffer)
    }
    fn update(&self, password: &SecVec<u8>, text_buffer: &TextBuffer) -> JoinHandle<bool> {
        let command = command_build(password, "pacman -Syu --noconfirm".to_owned());
        command::run_stream(command, text_buffer)
    }
}
pub fn is_available() -> bool {
    let packages = command::run("pacman --version");
    packages.is_ok()
}

fn command_build(password: &SecVec<u8>, command: String) -> String {
    let mut command = command;
    if cfg!(unix) {
        let password = String::from_utf8(password.unsecure().to_owned()).unwrap();
        command.insert_str(0, &format!("echo '{}' | sudo -S ", password));
    }
    command
}
