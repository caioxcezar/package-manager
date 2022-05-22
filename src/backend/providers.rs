use super::{
    provider::Provider,
    providers_impl::{flatpak, pacman},
};

pub struct Providers {
    pub list: Vec<Box<dyn Provider>>,
}
pub fn init() -> Providers {
    let mut prov = Providers {
        list: Vec::new(),
    };
    if pacman::is_available() {
        prov.list.push(Box::new(pacman::init()));
    }
    if flatpak::is_available() {
        prov.list.push(Box::new(flatpak::init()));
    }
    prov
}
