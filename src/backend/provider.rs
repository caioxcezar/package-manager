use super::{
    package_object::PackageData,
    providers_impl::{
        dnf::Dnf, flatpak::Flatpak, pacman::Pacman, paru::Paru, protonge::ProtonGE, winget::Winget,
    },
};
use crate::backend::package_object::PackageObject;
use gtk::{gio::ListStore, prelude::*, TextBuffer};
use secstr::SecVec;
use std::thread::JoinHandle;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter)]
pub enum ProviderKind {
    FLATPAK(Flatpak),
    PACMAN(Pacman),
    PARU(Paru),
    PROTONGE(ProtonGE),
    WINGET(Winget),
    DNF(Dnf),
}

impl ProviderKind {
    fn as_provider_actions(&self) -> &dyn ProviderActions {
        match self {
            ProviderKind::FLATPAK(provider) => provider,
            ProviderKind::PACMAN(provider) => provider,
            ProviderKind::PARU(provider) => provider,
            ProviderKind::PROTONGE(provider) => provider,
            ProviderKind::WINGET(provider) => provider,
            ProviderKind::DNF(provider) => provider,
        }
    }
    fn as_mut_provider_actions(&mut self) -> &mut dyn ProviderActions {
        match self {
            ProviderKind::FLATPAK(provider) => provider,
            ProviderKind::PACMAN(provider) => provider,
            ProviderKind::PARU(provider) => provider,
            ProviderKind::PROTONGE(provider) => provider,
            ProviderKind::WINGET(provider) => provider,
            ProviderKind::DNF(provider) => provider,
        }
    }
    pub fn is_available(&self) -> bool {
        self.as_provider_actions().is_available()
    }
    pub fn name(&self) -> String {
        self.as_provider_actions().name()
    }
    pub fn is_root_required(&self) -> bool {
        self.as_provider_actions().is_root_required()
    }
    pub fn package_info(&self, package_name: &str) -> String {
        self.as_provider_actions().package_info(package_name)
    }
    pub fn update(&self, password: &SecVec<u8>, text_buffer: &TextBuffer) -> JoinHandle<bool> {
        self.as_provider_actions().update(password, text_buffer)
    }
    pub fn install(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool> {
        self.as_provider_actions()
            .install(password, package, text_buffer)
    }
    pub fn remove(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool> {
        self.as_provider_actions()
            .remove(password, package, text_buffer)
    }
    pub fn update_packages(&mut self) -> Result<(), String> {
        self.as_mut_provider_actions().load_packages()
    }
    pub fn model(&self) -> Result<ListStore, String> {
        let list_store = ListStore::new(PackageObject::static_type());

        for value in self.as_provider_actions().packages() {
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
    pub fn available_providers() -> Vec<ProviderKind> {
        ProviderKind::iter()
            .filter(|provider_kind| provider_kind.is_available())
            .collect()
    }
}

pub trait ProviderActions {
    fn load_packages(&mut self) -> Result<(), String>;
    fn is_available(&self) -> bool;
    fn name(&self) -> String;
    fn is_root_required(&self) -> bool;
    fn packages(&self) -> Vec<PackageData>;
    fn package_info(&self, package: &str) -> String;
    fn install(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool>;
    fn remove(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool>;
    fn update(&self, password: &SecVec<u8>, text_buffer: &TextBuffer) -> JoinHandle<bool>;
    #[allow(dead_code)]
    fn installed(&self) -> usize;
    #[allow(dead_code)]
    fn total(&self) -> usize;
}
