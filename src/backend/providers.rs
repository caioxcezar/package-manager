use super::{
    provider::Provider,
    providers_impl::{flatpak, pacman},
};
use gtk::prelude::*;
#[derive(Default)]
pub struct Providers {
    pub list: Vec<Box<dyn Provider>>,
}
impl Providers {
    pub fn get_model(&self, name: String) -> Option<gtk::ListStore> {
        let mut provider = None;
        for prov in &self.list {
            if name.eq(&prov.get_name()) {
                provider = Some(prov);
            }
        }
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
