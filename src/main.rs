#![windows_subsystem = "windows"]

mod application;
mod backend;
mod constants;
mod grid_check;
mod grid_text;
mod messagebox;
mod window;

use application::PackageManagerApplication;
use gdk::Display;
use gtk::{gdk, gio, prelude::*, CssProvider};

fn main() {
    // Register and include resources
    gio::resources_register_include!("package_manager.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = PackageManagerApplication::new(constants::APP_ID);
    app.connect_startup(|program| {
        load_css(program);
    });
    std::process::exit(app.run().value());
}

fn load_css(_program: &PackageManagerApplication) {
    let provider = CssProvider::new();
    provider.load_from_resource("/org/caioxcezar/packagemanager/styles.css");

    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
