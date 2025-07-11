use super::{
    command::{self, CommandStream},
    package_object::PackageData,
    providers_impl::{
        dnf::Dnf, flatpak::Flatpak, pacman::Pacman, paru::Paru, protonge::ProtonGE, winget::Winget,
    },
};
use anyhow::Result;
use gtk::gio::ListStore;
use secstr::SecVec;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, Clone)]
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
    pub fn package_info(&self, package_name: String) -> Result<String> {
        self.as_provider_actions().package_info(package_name)
    }
    pub fn update(&self, password: Option<SecVec<u8>>) -> Result<CommandStream> {
        let _ = command::run("sudo -k");
        self.as_provider_actions().update(password)
    }
    pub fn install(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        let _ = command::run("sudo -k");
        self.as_provider_actions().install(password, package)
    }
    pub fn remove(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream> {
        let _ = command::run("sudo -k");
        self.as_provider_actions().remove(password, package)
    }
    pub fn update_packages(&mut self) -> Result<()> {
        self.as_mut_provider_actions().load_packages()
    }
    pub fn model(&self) -> Result<ListStore> {
        let list_store = ListStore::from_iter(
            self.as_provider_actions()
                .packages()
                .iter()
                .map(|value| value.cast()),
        );
        Ok(list_store)
    }
    pub fn available_providers() -> Vec<ProviderKind> {
        ProviderKind::iter()
            .filter(|provider_kind| provider_kind.is_available())
            .collect()
    }
}

pub trait ProviderActions {
    fn load_packages(&mut self) -> Result<()>;
    fn is_available(&self) -> bool;
    fn name(&self) -> String;
    fn is_root_required(&self) -> bool;
    fn packages(&self) -> Vec<PackageData>;
    fn package_info(&self, package: String) -> Result<String>;
    fn install(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream>;
    fn remove(&self, password: Option<SecVec<u8>>, package: String) -> Result<CommandStream>;
    fn update(&self, password: Option<SecVec<u8>>) -> Result<CommandStream>;
    #[allow(dead_code)]
    fn installed(&self) -> usize;
    #[allow(dead_code)]
    fn total(&self) -> usize;
}
