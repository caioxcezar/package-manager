use adw::subclass::prelude::*;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::constants;
mod imp;

glib::wrapper! {
    pub struct PackageManagerApplication(ObjectSubclass<imp::PackageManagerApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PackageManagerApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::new(&[("application-id", &application_id), ("flags", flags)])
            .expect("Failed to create PackageManagerApplication")
    }

    fn setup_gactions(&self) {
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.quit();
        }));
        self.add_action(&quit_action);

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_about();
        }));
        self.add_action(&about_action);
    }

    fn show_about(&self) {
        let img = gtk::Image::new();
        img.set_from_resource(Some("/org/caioxcezar/packagemanager/package_manager.svg"));
        let paintable = img.paintable().unwrap();
        let window = self.active_window().unwrap();
        let dialog = gtk::AboutDialog::builder()
            .transient_for(&window)
            .modal(true)
            .logo(&paintable)
            .program_name("Package Manager")
            .version(constants::VERSION)
            .comments("A simple package manager")
            .website_label("github")
            .website("https://github.com/caioxcezar/package-manager")
            .authors(vec!["Caio Rezende".into()])
            .build();

        dialog.present();
    }
}
