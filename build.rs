fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/package_manager.gresource.xml",
        "package_manager.gresource",
    );
}
