use adw::subclass::prelude::*;
use anyhow::{Context, Result};
use glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::constants;
mod imp;

glib::wrapper! {
    pub struct PackageManagerApplication(ObjectSubclass<imp::PackageManagerApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PackageManagerApplication {
    pub fn new(application_id: &str) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| {
                app.quit();
            }
        ));
        self.add_action(&quit_action);

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| {
                if let Err(msg) = app.show_about() {
                    println!("{}", msg);
                }
            }
        ));
        self.add_action(&about_action);
    }

    fn show_about(&self) -> Result<()> {
        let img = gtk::Image::from_resource("/org/caioxcezar/packagemanager/package_manager.svg");
        let paintable = img
            .paintable()
            .context("Failed to load image package_manager.svg")?;
        let window = self
            .active_window()
            .context("Failed to find the active window")?;
        let dialog = gtk::AboutDialog::builder()
            .transient_for(&window)
            .modal(true)
            .logo(&paintable)
            .program_name("Package Manager")
            .version(constants::VERSION)
            .comments("A simple package manager")
            .website_label("github")
            .website("https://github.com/caioxcezar/package-manager")
            .authors(vec!["Caio Rezende"])
            .build();

        dialog.present();

        Ok(())
    }
}
