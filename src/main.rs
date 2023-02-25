mod application;
mod backend;
mod constants;
mod messagebox;
mod window;
use application::PackageManagerApplication;
use gtk::gio;
use gtk::prelude::*;

fn main() {
    // Register and include resources
    gio::resources_register_include!("package_manager.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = PackageManagerApplication::new(constants::APP_ID);
    std::process::exit(app.run().value());
}
