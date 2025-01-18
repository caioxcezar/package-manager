mod imp;

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct PackageObject(ObjectSubclass<imp::PackageObject>);
}

impl PackageObject {
    pub fn new(
        installed: bool,
        repository: String,
        name: String,
        version: String,
        qualified_name: String,
    ) -> Self {
        Object::builder()
            .property("installed", installed)
            .property("repository", repository)
            .property("name", name)
            .property("version", version)
            .property("qualifiedName", qualified_name)
            .build()
    }
}
#[derive(Default, Clone, Debug)]
pub struct PackageData {
    pub installed: bool,
    pub repository: String,
    pub name: String,
    pub version: String,
    pub qualified_name: String,
}

impl PackageData {
    pub fn cast(&self) -> PackageObject {
        PackageObject::new(
            self.installed,
            self.repository.to_owned(),
            self.name.to_owned(),
            self.version.to_owned(),
            self.qualified_name.to_owned(),
        )
    }
}
