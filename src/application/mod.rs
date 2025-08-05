use adw::subclass::prelude::*;
use anyhow::{Context, Result};
use glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::backend::settings;
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
                    println!("{msg}");
                }
            }
        ));
        self.add_action(&about_action);

        let initial_state = if let Ok(value) = settings::get() {
            value.fuzzy_search
        } else {
            false
        };
        let search_type_action = gio::SimpleAction::new_stateful(
            "search-type",
            None,
            &glib::Variant::from(initial_state),
        );
        search_type_action.connect_change_state(move |action, value| {
            let new_value = value.unwrap();
            let bool_value = new_value.get::<bool>().unwrap();

            if let Ok(mut value) = settings::get() {
                let _ = value.set_bool("fuzzy_search", bool_value);
            }

            action.set_state(new_value);
        });
        self.add_action(&search_type_action);
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
