use super::*;
use gtk::glib;
#[derive(Debug, Default)]
pub struct PackageManagerApplication {}

#[glib::object_subclass]
impl ObjectSubclass for PackageManagerApplication {
    const NAME: &'static str = "PackageManagerApplication";
    type Type = super::PackageManagerApplication;
    type ParentType = adw::Application;
}

impl ObjectImpl for PackageManagerApplication {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        obj.setup_gactions();
        obj.set_accels_for_action("app.quit", &["<primary>q"]);
    }
}

impl ApplicationImpl for PackageManagerApplication {
    fn activate(&self, application: &Self::Type) {
        let window = if let Some(window) = application.active_window() {
            window
        } else {
            let window = crate::Window::new(application);
            window.upcast()
        };
        window.present();
    }
}

impl GtkApplicationImpl for PackageManagerApplication {}
impl AdwApplicationImpl for PackageManagerApplication {}
