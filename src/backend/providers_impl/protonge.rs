use crate::backend::{api, command, package_object::PackageData, provider::ProviderActions};
use gtk::glib;
use gtk::prelude::TextBufferExt;
use rayon::prelude::*;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::thread;

#[derive(Clone, Debug)]
pub struct ProtonGE {
    name: String,
    packages: Vec<PackageData>,
    installed: usize,
    total: usize,
    root_required: bool,
    endpoint: String,
    folder_path: String,
    packages_description: Vec<ApiResponse>,
}

#[derive(Deserialize, Debug, Default, Clone)]
struct ApiResponse {
    tag_name: String,
    body: String,
    html_url: String,
    assets_url: String,
}

#[derive(Deserialize, Debug)]
struct ApiAssets {
    browser_download_url: String,
    name: String,
}

impl Default for ProtonGE {
    fn default() -> Self {
        let home = command::run("echo $HOME").unwrap();
        let home = home.trim();
        let mut folder_path: &str = "";
        if Path::new(&format!("{}{}", home, "/.steam")).exists() {
            folder_path = "/.steam/root/compatibilitytools.d";
        }
        if Path::new(&format!("{}{}", home, "/.var/app/com.valvesoftware.Steam")).exists() {
            folder_path = "/.var/app/com.valvesoftware.Steam/data/Steam/compatibilitytools.d";
        }
        ProtonGE {
            name: String::from("Proton GE"),
            packages: Vec::new(),
            root_required: false,
            installed: 0,
            total: 0,
            endpoint: String::from(
                "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases",
            ),
            folder_path: folder_path.to_string(),
            packages_description: Vec::new(),
        }
    }
}

impl ProviderActions for ProtonGE {
    fn load_packages(&mut self) -> Result<(), String> {
        let proton_location = self.proton_location();
        let proton_dir = fs::read_dir(proton_location).unwrap();
        let proton: Vec<String> = proton_dir
            .filter_map(|dir| {
                let entry = dir.unwrap();
                let file_type = entry.file_type().unwrap();
                if file_type.is_dir() {
                    let name = entry.file_name().to_str().unwrap().to_owned();
                    Some(name)
                } else {
                    None
                }
            })
            .collect();
        let resp = api::get::<Vec<ApiResponse>>(&self.endpoint);
        self.packages_description = match resp {
            Ok(value) => value,
            Err(value) => {
                return Err(value);
            }
        };
        self.packages = self
            .packages_description
            .par_iter()
            .map(|package| {
                let name = &package.tag_name;
                let name = if name.contains("GE-Proton") {
                    package.tag_name.to_owned()
                } else {
                    format!("GE-Proton{}", name)
                };
                let version = &name[9..];
                PackageData {
                    name: name.to_owned(),
                    qualified_name: name.to_owned(),
                    repository: "GloriousEggroll".to_owned(),
                    version: version.to_string(),
                    installed: proton.contains(&name),
                }
            })
            .collect();
        self.installed = self.packages.par_iter().filter(|&p| p.installed).count();
        self.total = self.packages.len();
        Ok(())
    }
    fn name(&self) -> String {
        self.name.to_owned()
    }
    fn is_root_required(&self) -> bool {
        self.root_required
    }
    fn packages(&self) -> Vec<PackageData> {
        self.packages.clone()
    }
    fn package_info(&self, package: &str) -> String {
        match self.api_package_data(package) {
            Some(value) => format!("URL: {}\n{}", value.html_url, value.body),
            None => "Description not found".to_owned(),
        }
    }
    fn install(
        &self,
        _password: &secstr::SecVec<u8>,
        package: &str,
        text_buffer: &gtk::TextBuffer,
    ) -> std::thread::JoinHandle<bool> {
        self.download(package, text_buffer)
    }
    fn remove(
        &self,
        _password: &secstr::SecVec<u8>,
        package: &str,
        text_buffer: &gtk::TextBuffer,
    ) -> std::thread::JoinHandle<bool> {
        let pkg = self.package(package).unwrap().clone();
        let txt_buffer = text_buffer.clone();
        let proton_location = self.proton_location();
        let (sender, receiver) = async_channel::unbounded();
        let join_handle = thread::spawn(move || {
            let proton_dir = fs::read_dir(&proton_location).unwrap();
            for dir in proton_dir {
                let entry = dir.unwrap();
                let file_type = entry.file_type().unwrap();
                if file_type.is_dir() {
                    let name = entry.file_name().to_str().unwrap().to_owned();
                    if name.contains(&pkg.version) {
                        let result = fs::remove_dir_all(format!("{}/{}", &proton_location, &name));
                        match result {
                            Ok(_) => {
                                let _ = sender.send_blocking("Removido com sucesso. ".to_owned());
                                return true;
                            }
                            Err(value) => {
                                let _ = sender.send_blocking(value.to_string());
                                return false;
                            }
                        }
                    }
                }
            }
            false
        });

        glib::MainContext::default().spawn_local(async move {
            while let Ok(text) = receiver.recv().await {
                let mut text_iter = txt_buffer.end_iter();
                txt_buffer.insert(&mut text_iter, &text);
            }
        });

        join_handle
    }
    fn update(
        &self,
        _password: &secstr::SecVec<u8>,
        text_buffer: &gtk::TextBuffer,
    ) -> std::thread::JoinHandle<bool> {
        let pkg = self.packages[0].clone();

        if pkg.installed {
            let mut text_iter = text_buffer.end_iter();
            text_buffer.insert(&mut text_iter, "Nothing to do. ");
            return thread::spawn(|| true);
        }
        self.download(&pkg.name, text_buffer)
    }
    fn installed(&self) -> usize {
        self.installed
    }
    fn total(&self) -> usize {
        self.total
    }
    fn is_available(&self) -> bool {
        if cfg!(windows) {
            return false;
        }
        let home = command::run("echo $HOME").unwrap();
        let home = home.trim();
        if !Path::new(&format!("{}{}", home, "/.steam")).exists()
            && !Path::new(&format!("{}{}", home, "/.var/app/com.valvesoftware.Steam")).exists()
        {
            return false;
        }
        api::get_str("https://api.github.com/zen").is_ok()
    }
}
impl ProtonGE {
    fn api_package_data(&self, tag_name: &str) -> Option<&ApiResponse> {
        self.packages_description
            .par_iter()
            .find_any(|response| response.tag_name.eq(tag_name))
    }
    fn package(&self, name: &str) -> Option<&PackageData> {
        self.packages
            .par_iter()
            .find_any(|package| package.name.eq(name))
    }
    fn download(
        &self,
        package: &str,
        text_buffer: &gtk::TextBuffer,
    ) -> std::thread::JoinHandle<bool> {
        let mut url: Option<String> = None;
        let api_response = self.api_package_data(package).unwrap();
        let assets = api::get::<Vec<ApiAssets>>(&api_response.assets_url);
        if let Ok(assets) = assets {
            for response in assets {
                if response.name.contains(".tar.gz") {
                    url = Some(response.browser_download_url);
                }
            }
        }
        api::download_and_extract(url.unwrap(), self.proton_location(), text_buffer)
    }
    fn proton_location(&self) -> String {
        let home = command::run("echo $HOME").unwrap();
        let home = home.trim();
        let path = format!("{}{}", home, &self.folder_path);
        if !Path::new(&path).exists() {
            let _ = fs::create_dir_all(format!("{}{}", home, &self.folder_path));
        }
        path
    }
}
