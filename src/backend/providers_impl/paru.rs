use std::thread::JoinHandle;

use gtk::TextBuffer;
use rayon::prelude::*;
use secstr::SecVec;

use crate::backend::{command, package_object::PackageData, provider::ProviderActions};
#[derive(Clone, Debug)]
pub struct Paru {
    pub name: String,
    pub packages: Vec<PackageData>,
    pub installed: usize,
    pub total: usize,
    pub root_required: bool,
}

impl Default for Paru {
    fn default() -> Self {
        Paru {
            name: String::from("Paru"),
            packages: Vec::new(),
            root_required: true,
            installed: 0,
            total: 0,
        }
    }
}

impl ProviderActions for Paru {
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
        let packages = command::run("paru -Sl");
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
        command::run(&format!("paru -Si {}", package)).unwrap()
    }
    fn install(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool> {
        let password = String::from_utf8(password.unsecure().to_owned()).unwrap();
        let command = format!(
            "echo '{}' | sudo -S su && paru -Syu {} --noconfirm",
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
            "echo '{}' | sudo -S su && paru -Rsu {} --noconfirm",
            password, package
        );
        command::run_stream(command, text_buffer)
    }
    fn update(&self, password: &SecVec<u8>, text_buffer: &TextBuffer) -> JoinHandle<bool> {
        let password = String::from_utf8(password.unsecure().to_owned()).unwrap();
        let command = format!("echo '{}' | sudo -S su && paru -Syu --noconfirm", password);
        command::run_stream(command, text_buffer)
    }
    fn is_available(&self) -> bool {
        let packages = command::run("paru --version");
        packages.is_ok()
    }
}
