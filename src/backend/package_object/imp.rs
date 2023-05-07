use std::cell::RefCell;

use glib::{ParamSpec, Properties, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::PackageData;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::PackageObject)]
pub struct PackageObject {
    #[property(name = "installed", get, set, type = bool, member = installed)]
    #[property(name = "repository", get, set, type = String, member = repository)]
    #[property(name = "name", get, set, type = String, member = name)]
    #[property(name = "version", get, set, type = String, member = version)]
    #[property(name = "qualifiedName", get, set, type = String, member = qualified_name)]
    pub data: RefCell<PackageData>,
}

#[glib::object_subclass]
impl ObjectSubclass for PackageObject {
    const NAME: &'static str = "PackageObject";
    type Type = super::PackageObject;
}

impl ObjectImpl for PackageObject {
    fn properties() -> &'static [ParamSpec] {
        Self::derived_properties()
    }

    fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
        self.derived_set_property(id, value, pspec)
    }

    fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
        self.derived_property(id, pspec)
    }
}