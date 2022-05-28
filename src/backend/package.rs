#[derive(Clone, Debug)]
pub struct Package {
    pub provider: String,
    pub repository: String,
    pub name: String,
    pub version: String,
    pub qualified_name: String,
    pub is_installed: bool,
}
#[derive(Clone, Debug)]
pub struct PendingPackage {
    pub is_installing: bool,
    pub package_name: String,
}
