use std::thread::JoinHandle;

use gtk::TextBuffer;
use secstr::SecVec;

use super::package_object::PackageData;

pub trait Provider {
    fn load_packages(&mut self) -> Result<(), String>;
    fn name(&self) -> String;
    fn is_root_required(&self) -> bool;
    fn packages(&self) -> Vec<PackageData>;
    fn package_info(&self, package: &str) -> String;
    fn install(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool>;
    fn remove(
        &self,
        password: &SecVec<u8>,
        package: &str,
        text_buffer: &TextBuffer,
    ) -> JoinHandle<bool>;
    fn update(&self, password: &SecVec<u8>, text_buffer: &TextBuffer) -> JoinHandle<bool>;
    fn installed(&self) -> usize;
    fn total(&self) -> usize;
}
