use super::{
    provider::Provider,
    providers_impl::{flatpak, pacman, paru},
};
use gtk::{glib, prelude::*, TextBuffer};
use secstr::SecVec;
use std::thread::{self, JoinHandle};
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
    pub fn update(
        &self,
        provider_name: &str,
        text_buffer: &TextBuffer,
        password: &SecVec<u8>,
    ) -> JoinHandle<bool> {
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
        package: &str,
        text_buffer: &TextBuffer,
        password: &SecVec<u8>,
    ) -> JoinHandle<bool> {
        let provider = self.get_provider(&provider_name).unwrap();
        provider.install(password, package, text_buffer)
    }
    pub fn remove(
        &self,
        provider_name: &str,
        package: &str,
        text_buffer: &TextBuffer,
        password: &SecVec<u8>,
    ) -> JoinHandle<bool> {
        let provider = self.get_provider(provider_name).unwrap();
        provider.remove(&password, package, text_buffer)
    }
    pub fn update_all(&self, text_buffer: &TextBuffer, password: &SecVec<u8>) {
        let mut packages_name = Vec::new();
        for package in &self.list {
            packages_name.push(package.get_name());
        }
        inner_update_all(&mut packages_name, text_buffer, password);
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
    if paru::is_available() {
        match paru::init() {
            Ok(value) => prov.list.push(Box::new(value)),
            Err(value) => errors.push_str(&value),
        }
    }
    if "".to_owned().eq(&errors) {
        return Ok(prov);
    }
    Err(errors)
}

fn get_provider(provider_name: &str) -> Option<Box<dyn Provider>> {
    match provider_name {
        "Pacman" => Some(Box::new(pacman::init().unwrap())),
        "Flatpak" => Some(Box::new(flatpak::init().unwrap())),
        "Paru" => Some(Box::new(paru::init().unwrap())),
        &_ => None,
    }
}

fn inner_update_all(
    provider_names: &mut Vec<String>,
    text_buffer: &TextBuffer,
    password: &SecVec<u8>,
) {
    if provider_names.len() <= 0 {
        let mut text_iter = text_buffer.end_iter();
        text_buffer.insert(&mut text_iter, ":::: Updated All ::::");
        return;
    }
    let provider_name = provider_names.remove(0);
    let provider = get_provider(&provider_name).unwrap();
    let handle = provider.update(&password, &text_buffer);

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    thread::spawn(move || {
        let _ = handle.join().unwrap();
        let _ = tx.send(());
    });

    let text_buffer_clone = text_buffer.clone();
    let password_clone = password.clone();
    let mut provider_names = provider_names.clone();

    rx.attach(None, move |_| {
        let _ = inner_update_all(&mut provider_names, &text_buffer_clone, &password_clone);
        glib::Continue(false)
    });
}
