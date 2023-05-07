mod imp;
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::traits::{CheckButtonExt, WidgetExt};

glib::wrapper! {
    pub struct GridCheck(ObjectSubclass<imp::GridCheck>)
        @extends gtk::Widget;
}

impl Default for GridCheck {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Entry {
    pub check: bool,
    pub sensitive: bool,
}

impl GridCheck {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_entry(&self, entry: &Entry) {
        self.imp().check.set_active(entry.check);
        self.imp().check.set_sensitive(entry.sensitive);
    }
}
