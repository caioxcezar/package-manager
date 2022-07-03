use crate::backend::{api, command, package::Package, provider::Provider};
use serde::Deserialize;

pub struct ProtonGE {
    name: String,
    packages: Vec<Package>,
    installed: usize,
    total: usize,
    root_required: bool,
    endpoint: String,
    packages_description: Vec<ApiResponse>,
}

#[derive(Deserialize, Debug)]
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

pub fn init() -> ProtonGE {
    ProtonGE {
        name: String::from("Proton GE"),
        packages: Vec::new(),
        root_required: false,
        installed: 0,
        total: 0,
        endpoint: String::from(
            "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases",
        ),
        packages_description: Vec::new(),
    }
}

impl Provider for ProtonGE {
    fn load_packages(&mut self) -> Result<(), String> {
        let home = command::run("echo $HOME");
        let home = match home {
            Ok(value) => value,
            Err(value) => {
                return Err(value.to_string());
            }
        };
        let home = format!("{}/.local/share/Steam/compatibilitytools.d/", home.trim());
        let proton = command::run(&format!("ls '{}'", home));
        let proton = match proton {
            Ok(value) => value,
            Err(value) => {
                return Err(value.to_string());
            }
        };
        let resp = api::get::<Vec<ApiResponse>>(&self.endpoint);
        self.packages_description = match resp {
            Ok(value) => value,
            Err(value) => {
                return Err(value);
            }
        };
        self.packages = Vec::new();
        for package in &self.packages_description {
            let name = &package.tag_name;
            let name = if name.contains("GE-Proton") {
                package.tag_name.to_owned()
            } else {
                format!("GE-Proton{}", name)
            };
            let version = &name[9..];
            self.packages.push(Package {
                provider: "GloriousEggroll".to_owned(),
                name: name.to_owned(),
                qualified_name: name.to_owned(),
                repository: "GloriousEggroll".to_owned(),
                version: version.to_string(),
                is_installed: proton.contains(&name),
            });
        }
        self.installed = self.packages.iter().filter(|&p| p.is_installed).count();
        self.total = self.packages.len();
        Ok(())
    }
    fn name(&self) -> String {
        self.name.to_owned()
    }
    fn is_root_required(&self) -> bool {
        self.root_required
    }
    fn packages(&self) -> Vec<Package> {
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
        command::run_stream(format!("echo 'TODO' {}", package), text_buffer) //TODO
    }
    fn update(
        &self,
        _password: &secstr::SecVec<u8>,
        text_buffer: &gtk::TextBuffer,
    ) -> std::thread::JoinHandle<bool> {
        command::run_stream(format!("echo 'TODO'"), text_buffer) //TODO
    }
    fn installed(&self) -> usize {
        self.installed
    }
    fn total(&self) -> usize {
        self.total
    }
}
impl ProtonGE {
    fn api_package_data(&self, tag_name: &str) -> Option<&ApiResponse> {
        for response in &self.packages_description {
            if response.tag_name.eq(tag_name) {
                return Some(response);
            }
        }
        None
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

        let home = command::run("echo $HOME").unwrap();
        let home = format!(
            "{}/.local/share/Steam/compatibilitytools.d/{}",
            home, package
        );

        api::download(url.unwrap(), home, text_buffer)
    }
}
pub fn is_available() -> bool {
    true //TODO
}
