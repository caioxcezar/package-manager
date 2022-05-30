use crate::messagebox;

use super::{
    provider::Provider,
    providers_impl::{flatpak, pacman},
};
use gtk::{prelude::*, TextBuffer};
use secstr::SecStr;
#[derive(Default)]
pub struct Providers {
    pub list: Vec<Box<dyn Provider>>,
}
impl Providers {
    fn get_provider(&self, provider_name: String) -> Option<&Box<dyn Provider>> {
        for prov in &self.list {
            if provider_name.eq(&prov.get_name()) {
                return Some(prov);
            }
        }
        None
    }
    pub fn get_model(&self, name: String) -> Option<gtk::ListStore> {
        let provider = self.get_provider(name);
        let provider = match provider {
            Some(value) => value,
            _ => return None,
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
        Some(model)
    }
    pub fn update(&self, provider_name: String, text_buffer: &TextBuffer) {
        let provider = self.get_provider(provider_name.clone());
        if let Some(provider) = provider {
            let mut password = SecStr::from("");
            if provider.is_root_required() {
                password = messagebox::ask_password();
            }
            provider.update(password, text_buffer);
        } else {
            let text = format!("No provider with the name of {}", provider_name);
            messagebox::error("Provider not found", &text[..]);
        }
    }
    pub fn package_info(&self, provider_name: String, provider: String) -> String {
        self.get_provider(provider)
            .unwrap()
            .package_info(provider_name)
    }
    pub fn install(&self, provider_name: String, packages: Vec<String>, text_buffer: &TextBuffer) {
        let provider = self.get_provider(provider_name.clone());
        if let Some(provider) = provider {
            let mut password = SecStr::from("");
            if provider.is_root_required() {
                password = messagebox::ask_password();
            }
            provider.install(password, packages, text_buffer);
        } else {
            let text = format!("No provider with the name of {}", provider_name);
            messagebox::error("Provider not found", &text[..]);
        }
    }
    pub fn remove(&self, provider_name: String, packages: Vec<String>, text_buffer: &TextBuffer) {
        let provider = self.get_provider(provider_name.clone());
        if let Some(provider) = provider {
            let mut password = SecStr::from("");
            if provider.is_root_required() {
                password = messagebox::ask_password();
            }
            provider.install(password, packages, text_buffer);
        } else {
            let text = format!("No provider with the name of {}", provider_name);
            messagebox::error("Provider not found", &text[..]);
        }
    }
    pub fn update_all(&self, text_buffer: &TextBuffer) {
        let ask_password = self
            .list
            .iter()
            .any(|value: &Box<dyn Provider>| value.is_root_required());
        let mut password = SecStr::from("");
        if ask_password {
            password = messagebox::ask_password();
        }
        for provider in &self.list {
            provider.update(password.clone(), text_buffer);
        }
    }
}
pub fn init() -> Providers {
    let mut prov = Providers { list: Vec::new() };
    if pacman::is_available() {
        prov.list.push(Box::new(pacman::init()));
    }
    if flatpak::is_available() {
        prov.list.push(Box::new(flatpak::init()));
    }
    prov
}
