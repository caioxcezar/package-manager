mod imp;
use gtk::glib;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct GridText(ObjectSubclass<imp::GridText>)
        @extends gtk::Widget;
}

impl Default for GridText {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Entry {
    pub name: String,
}

impl GridText {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_entry(&self, entry: &Entry) {
        self.imp().name.set_text(Some(&entry.name));
    }
}
