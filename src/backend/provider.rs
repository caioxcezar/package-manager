use gtk::TextBuffer;
use secstr::SecVec;

use crate::backend::package::Package;

pub trait Provider {
    fn load_packages(&mut self);
    fn get_name(&self) -> String;
    fn is_root_required(&self) -> bool;
    fn get_packages(&self) -> Vec<Package>;
    fn package_info(&self, package: String) -> String;
    fn install(&self, password: SecVec<u8>, packages: Vec<String>, text_buffer: &TextBuffer);
    fn remove(&self, password: SecVec<u8>, packages: Vec<String>, text_buffer: &TextBuffer);
    fn update(&self, password: SecVec<u8>, text_buffer: &TextBuffer);
}
