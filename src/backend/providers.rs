use super::{
    provider::Provider,
    providers_impl::{flatpak, pacman},
};
use gtk::{prelude::*, TextBuffer};
use secstr::SecVec;
#[derive(Default)]
pub struct Providers {
    pub list: Vec<Box<dyn Provider>>,
}
impl Providers {
    /// Tecnicamente impossível voltar None
    fn get_provider(&self, provider_name: &str) -> Option<&Box<dyn Provider>> {
        for prov in &self.list {
            if provider_name.eq(&prov.get_name()) {
                return Some(prov);
            }
        }
        None
    }
    pub fn get_model(&self, name: &str) -> Result<gtk::ListStore, String> {
        let provider = self.get_provider(&name);
        let provider = match provider {
            Some(value) => value,
            _ => return Err("Provider not found. ".to_owned()),
        };
        let packages = provider.get_packages();
        let model = gtk::ListStore::new(&[
            bool::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
        ]);
        // Filling up the tree view.
        for value in &packages {
            let values: [(u32, &dyn ToValue); 5] = [
                (0, &value.is_installed),
                (1, &value.repository),
                (2, &value.name),
                (3, &value.version),
                (4, &value.qualified_name),
            ];
            model.insert_with_values(None, &values);
        }
        Ok(model)
    }
    pub fn update(&self, provider_name: &str, text_buffer: &TextBuffer, password: &SecVec<u8>) {
        let provider = self.get_provider(&provider_name).unwrap();
        provider.update(&password, text_buffer)
    }
    pub fn package_info(&self, provider_name: &str, provider: &str) -> Result<String, String> {
        let provider = self.get_provider(provider);
        let provider = match provider {
            Some(value) => value,
            _ => return Err("Provider not found".to_owned()),
        };
        Ok(provider.package_info(&provider_name))
    }
    pub fn install(
        &self,
        provider_name: &str,
        packages: &Vec<String>,
        text_buffer: &TextBuffer,
        password: &SecVec<u8>,
    ) {
        let provider = self.get_provider(&provider_name).unwrap();
        provider.install(password, packages, text_buffer)
    }
    pub fn remove(
        &self,
        provider_name: &str,
        packages: &Vec<String>,
        text_buffer: &TextBuffer,
        password: &SecVec<u8>,
    ) {
        let provider = self.get_provider(provider_name).unwrap();
        provider.remove(&password, packages, text_buffer);
    }
    pub fn update_all(&self, text_buffer: &TextBuffer, password: &SecVec<u8>) {
        for provider in &self.list {
            provider.update(password, text_buffer);
        }
    }
    pub fn is_root_required(&self, provider_name: &str) -> bool {
        let provider = self.get_provider(provider_name).unwrap();
        provider.is_root_required()
    }
    pub fn some_root_required(&self) -> bool {
        self.list
            .iter()
            .any(|value: &Box<dyn Provider>| value.is_root_required())
    }
}
pub fn init() -> Result<Providers, String> {
    let mut errors = "".to_owned();
    let mut prov = Providers { list: Vec::new() };
    if pacman::is_available() {
        match pacman::init() {
            Ok(value) => prov.list.push(Box::new(value)),
            Err(value) => errors = value,
        }
    }
    if flatpak::is_available() {
        match flatpak::init() {
            Ok(value) => prov.list.push(Box::new(value)),
            Err(value) => errors.push_str(&value),
        }
    }
    if "".to_owned().eq(&errors) {
        return Ok(prov);
    }
    Err(errors)
}
