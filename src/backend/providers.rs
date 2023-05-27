use super::{
    package_object::PackageObject,
    provider::Provider,
    providers_impl::{dnf, flatpak, pacman, paru, protonge, winget},
};
use gtk::{gio::ListStore, glib, prelude::*, TextBuffer};
use secstr::SecVec;
use std::thread::{self, JoinHandle};

#[derive(Default)]
pub struct Providers {
    pub avaible_providers: Vec<String>,
}
impl Providers {
    pub fn model(&self, name: &str) -> Result<ListStore, String> {
        let mut provider = provider(name);
        if let Err(value) = provider.load_packages() {
            return Err(format!("Error while loading packages. {}", value));
        }
        let packages = provider.packages();
        let list_store = ListStore::new(PackageObject::static_type());
        // Filling up the tree view.
        for value in &packages {
            list_store.append(&PackageObject::new(
                value.installed,
                value.repository.to_owned(),
                value.name.to_owned(),
                value.version.to_owned(),
                value.qualified_name.to_owned(),
            ));
        }
        Ok(list_store)
    }
    pub fn update(
        &self,
        provider_name: &str,
        text_buffer: &TextBuffer,
        password: &SecVec<u8>,
    ) -> JoinHandle<bool> {
        let provider = provider(provider_name);
        provider.update(password, text_buffer)
    }
    pub fn package_info(&self, package_name: &str, provider_name: &str) -> Result<String, String> {
        let provider = provider(provider_name);
        Ok(provider.package_info(package_name))
    }
    pub fn install(
        &self,
        provider_name: &str,
        package: &str,
        text_buffer: &TextBuffer,
        password: &SecVec<u8>,
    ) -> JoinHandle<bool> {
        let provider = provider(provider_name);
        provider.install(password, package, text_buffer)
    }
    pub fn remove(
        &self,
        provider_name: &str,
        package: &str,
        text_buffer: &TextBuffer,
        password: &SecVec<u8>,
    ) -> JoinHandle<bool> {
        let provider = provider(provider_name);
        provider.remove(password, package, text_buffer)
    }
    pub fn update_all(&self, text_buffer: &TextBuffer, password: &SecVec<u8>) {
        inner_update_all(self.avaible_providers.clone(), text_buffer, password);
    }
    pub fn is_root_required(&self, provider_name: &str) -> bool {
        let provider = provider(provider_name);
        provider.is_root_required()
    }
    pub fn some_root_required(&self) -> bool {
        self.avaible_providers
            .iter()
            .any(|value| provider(value).is_root_required())
    }
}
pub fn init() -> Providers {
    let mut providers = Vec::<String>::new();
    if dnf::is_available() {
        providers.push(dnf::init().name())
    }
    if pacman::is_available() {
        providers.push(pacman::init().name())
    }
    if paru::is_available() {
        providers.push(paru::init().name())
    }
    if winget::is_available() {
        providers.push(winget::init().name())
    }
    if flatpak::is_available() {
        providers.push(flatpak::init().name())
    }
    if protonge::is_available() {
        providers.push(protonge::init().name())
    }
    Providers {
        avaible_providers: providers,
    }
}

fn provider(provider_name: &str) -> Box<dyn Provider> {
    match provider_name {
        "Pacman" => Box::new(pacman::init()),
        "Flatpak" => Box::new(flatpak::init()),
        "Paru" => Box::new(paru::init()),
        "Proton GE" => {
            let mut proton_ge = protonge::init();
            let _ = proton_ge.load_packages();
            Box::new(proton_ge)
        }
        "Winget" => Box::new(winget::init()),
        "Dnf" => Box::new(dnf::init()),
        &_ => panic!("Invalid Package"),
    }
}
fn inner_update_all(provider_names: Vec<String>, text_buffer: &TextBuffer, password: &SecVec<u8>) {
    if provider_names.is_empty() {
        let mut text_iter = text_buffer.end_iter();
        text_buffer.insert(&mut text_iter, "\n:::: All Updated ::::");
        return;
    }
    let mut provider_names = provider_names;
    let provider_name = provider_names.remove(0);
    let provider = provider(&provider_name);
    let handle = provider.update(password, text_buffer);

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    thread::spawn(move || {
        let _ = handle.join().unwrap();
        let _ = tx.send(());
    });

    let text_buffer_clone = text_buffer.clone();
    let password_clone = password.clone();

    rx.attach(None, move |_| {
        inner_update_all(provider_names.clone(), &text_buffer_clone, &password_clone);
        glib::Continue(false)
    });
}
