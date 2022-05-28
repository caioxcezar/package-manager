mod imp;

use glib::Object;
use gtk::{gio, glib};

use crate::application;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Application, gtk::Window, gtk::Widget, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &application::PackageManagerApplication) -> Self {
        // Create new window
        Object::new(&[("application", app)]).expect("Failed to create Window")
    }
}
