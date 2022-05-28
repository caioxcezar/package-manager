use crate::backend::package::Package;

pub trait Provider {
    fn load_packages(&mut self);
    fn get_name(&self) -> String;
    fn get_packages(&self) -> Vec<Package>;
    // fn install(&self, packages: Vec<Package>, output: String, error: String) -> u64;
    // fn remove(&self, packages: Vec<Package>, output: String, error: String) -> u64;
    // fn update(&self, packages: Vec<Package>, output: String, error: String) -> u64;
}
