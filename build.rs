use gtk::gio;

fn main() {
    gio::compile_resources(
        "resources",
        "resources/package_manager.gresource.xml",
        "package_manager.gresource",
    );
}
