use crate::backend::package::PendingPackage;
use crate::backend::providers::{self, Providers};
use crate::messagebox;
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate, ResponseType, TextBuffer};
use gtk::{prelude::*, ListStore};
use secstr::{SecStr, SecVec};
use std::sync::Mutex;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/caioxcezar/packagemanager/window.ui")]
pub struct Window {
    #[template_child]
    pub header_bar: TemplateChild<gtk::HeaderBar>,
    #[template_child]
    pub stack: TemplateChild<gtk::Stack>,
    #[template_child]
    pub tree_view: TemplateChild<gtk::TreeView>,
    #[template_child]
    pub search_entry: TemplateChild<gtk::SearchEntry>,
    #[template_child]
    pub combobox_provider: TemplateChild<gtk::ComboBoxText>,
    #[template_child]
    pub installed_renderer: TemplateChild<gtk::CellRendererToggle>,
    #[template_child]
    pub update_all: TemplateChild<gtk::Button>,
    #[template_child]
    pub action: TemplateChild<gtk::Button>,
    #[template_child]
    pub update: TemplateChild<gtk::Button>,
    #[template_child]
    pub text_box: TemplateChild<gtk::TextView>,
    #[template_child]
    pub tree_selection: TemplateChild<gtk::TreeSelection>,
    #[template_child]
    pub text_command: TemplateChild<gtk::TextView>,
    #[template_child]
    pub info_bar: TemplateChild<gtk::InfoBar>,
    #[template_child]
    pub info_bar_label: TemplateChild<gtk::Label>,
    providers: Mutex<Providers>,
    pending_packages: Mutex<Vec<PendingPackage>>,
    list_store: Mutex<Option<ListStore>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "PackageManagerWindow";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Window {
    fn constructed(&self, obj: &Self::Type) {
        // Call "constructed" on parent
        self.parent_constructed(obj);
        {
            let mut providers = self.providers.lock().unwrap();

            *providers = match providers::init() {
                Ok(value) => value,
                Err(value) => {
                    messagebox::error("Error while loading", &value, self.window());
                    return;
                }
            };

            for provider in &providers.list {
                let p = provider.get_name();
                self.combobox_provider.append_text(&p);
            }

            if providers.list.len() == 0 {
                self.update_all.set_sensitive(false);
                self.update.set_sensitive(false);
            }
        }
        self.combobox_provider.set_active(Some(0));
    }
}
#[gtk::template_callbacks]
impl Window {
    #[template_callback]
    fn treeview_selection_changed(&self, tree_selection: gtk::TreeSelection) {
        if let Some((tree_model, tree_iter)) = tree_selection.selected() {
            let package = tree_model
                .get_value(&tree_iter, 4 as i32)
                .get::<String>()
                .unwrap();
            {
                let providers = self.providers.lock().unwrap();
                let info = providers.package_info(&package, &self.combobox_text());
                let info = match info {
                    Ok(value) => value,
                    Err(value) => {
                        messagebox::error(
                            "Error While Changing",
                            &format!("{:?}", value),
                            self.window(),
                        );
                        return;
                    }
                };
                let buffer = TextBuffer::builder().text(&info).build();
                self.text_box.set_buffer(Some(&buffer));
                self.text_box.set_visible(true);
            }
        };
    }
    #[template_callback]
    fn handle_toggle(&self, index: &str, _: gtk::CellRendererToggle) {
        let mut list_store = self.list_store.lock().unwrap();
        match &mut *list_store {
            Some(model) => {
                let mut pending_packages = self.pending_packages.lock().unwrap();
                let path = gtk::TreePath::from_string(index).unwrap();
                let iter = model.iter(&path).unwrap();
                let mut installed = model.get_value(&iter, 0 as i32).get::<bool>().unwrap();
                let package_name = model.get_value(&iter, 4 as i32).get::<String>().unwrap();
                let mut add = true;
                for i in 0..pending_packages.len() {
                    if pending_packages[i].package_name.eq(&package_name) {
                        pending_packages.remove(i);
                        add = false;
                        break;
                    }
                }
                if add {
                    pending_packages.push(PendingPackage {
                        is_installing: !installed,
                        package_name: package_name,
                    });
                }
                self.action.set_sensitive(pending_packages.len().gt(&0));
                installed = !installed;
                model.set_value(&iter, 0 as u32, &installed.to_value());
            }
            _ => messagebox::error("Toggle error", "List Store not found", self.window()),
        }
    }
    #[template_callback]
    async fn handle_update_all(&self, _button: gtk::Button) {
        let providers = self.providers.try_lock();
        if let Ok(providers) = providers {
            let mut password = None;
            if providers.some_root_required() {
                password = messagebox::ask_password(self.window()).await;
            }
            let password = match password {
                Some(value) => value,
                _ => return,
            };
            self.goto_command();
            let buffer = TextBuffer::builder().text(&"").build();
            self.text_command.set_buffer(Some(&buffer));
            providers.update_all(&buffer, &password);
            self.show_info("Finalizado"); // FIXME não esta esperando o termindo da execução
        }
    }
    #[template_callback]
    async fn handle_action(&self, _: gtk::Button) {
        let mut install: Vec<String> = Vec::new();
        let mut remove: Vec<String> = Vec::new();
        self.pending_packages
            .lock()
            .unwrap()
            .iter()
            .for_each(|package| {
                if package.is_installing {
                    install.push(package.package_name.clone());
                } else {
                    remove.push(package.package_name.clone());
                }
            });

        let install_clone: Vec<String> = install.clone();
        let remove_clone: Vec<String> = remove.clone();

        let mut text: String = "".to_owned();
        if install.len() > 0 {
            text = "To be installed: ".to_owned();
            text = format!(
                "{} {}",
                text,
                install
                    .into_iter()
                    .reduce(|a, b| format!("{} {}", a, b))
                    .unwrap()
            );
        }
        if remove.len() > 0 {
            text = format!("{} {}", text, "\nTo be removed: ");
            text = format!(
                "{} {}",
                text,
                remove
                    .into_iter()
                    .reduce(|a, b| format!("{} {}", a, b))
                    .unwrap()
            );
        }
        let response = messagebox::info("Please, confirm the changes", &text, self.window()).await;
        if response == ResponseType::Ok {
            let text = self.combobox_text();
            let providers = self.providers.lock();
            if let Ok(providers) = providers {
                let buffer = TextBuffer::builder().text(&"").build();
                self.text_command.set_buffer(Some(&buffer));
                let password = match self.password(&providers).await {
                    Some(value) => value,
                    _ => return,
                };
                self.goto_command();
                if install_clone.len() > 0 {
                    providers.install(&text, &install_clone, &buffer, &password);
                }
                if remove_clone.len() > 0 {
                    providers.remove(&text, &remove_clone, &buffer, &password);
                }
                self.show_info("Finalizado"); // FIXME não esta esperando o termindo da execução
            }
        }
    }
    #[template_callback]
    fn handle_combobox_changed(&self, combobox: gtk::ComboBoxText) {
        let mut pending_packages = self.pending_packages.lock().unwrap();
        let mut list_store = self.list_store.lock().unwrap();
        let providers = self.providers.lock().unwrap();

        pending_packages.clear();
        let combobox_text = combobox.active_text().unwrap();
        let combobox_text = combobox_text.as_str();
        let provider = providers.get_model(combobox_text);
        let provider = match provider {
            Ok(value) => value,
            Err(value) => {
                messagebox::error("Error while changing provider", &value, self.window());
                return;
            }
        };
        self.tree_view.set_model(Some(&provider));
        *list_store = Some(provider);
    }
    #[template_callback]
    async fn handle_update(&self, _button: gtk::Button) {
        let providers = self.providers.try_lock();
        if let Ok(providers) = providers {
            let buffer = TextBuffer::builder().text(&"").build();
            self.text_command.set_buffer(Some(&buffer));
            let password = match self.password(&providers).await {
                Some(value) => value,
                _ => return,
            };
            self.goto_command();
            providers.update(&self.combobox_text(), &buffer, &password);
            self.show_info("Finalizado"); // FIXME não esta esperando o termindo da execução
        }
    }
    #[template_callback]
    fn handle_search(&self, entry: gtk::SearchEntry) {
        let value = entry.text().to_string();
        // let mut list_store = match self.list_store.try_lock() {
        //     Ok(value) => value,
        //     _ => return,
        // };
    }
    fn combobox_text(&self) -> String {
        let combobox_text = self.combobox_provider.active_text().unwrap();
        combobox_text.as_str().to_owned()
    }
    fn show_info(&self, message: &str) {
        let widget = self.stack.child_by_name("main_page").unwrap();
        let search_entry = self.search_entry.clone();
        let update = self.update.clone();
        let combobox_provider = self.combobox_provider.clone();
        let stack = self.stack.clone();
        self.info_bar.set_visible(true);
        self.info_bar_label.set_text(message);
        self.info_bar.connect_response(move |info_bar, _| {
            info_bar.set_visible(false);
            stack.set_visible_child(&widget);
            search_entry.set_sensitive(true);
            update.set_sensitive(true);
            combobox_provider.set_sensitive(true);
        });
    }
    fn goto_command(&self) {
        let widget = self.stack.child_by_name("command_page").unwrap();
        self.stack.set_visible_child(&widget);
        self.search_entry.set_sensitive(false);
        self.update.set_sensitive(false);
        self.combobox_provider.set_sensitive(false);
    }
    fn window(&self) -> Option<gtk::Window> {
        let search_entry = self.search_entry.clone();
        let widget = search_entry.upcast::<gtk::Widget>();
        widget
            .root()
            .map(|root| root.downcast::<gtk::Window>().unwrap())
    }
    async fn password(&self, providers: &Providers) -> Option<SecVec<u8>> {
        if providers.is_root_required(&self.combobox_text()) {
            return messagebox::ask_password(self.window()).await;
        }
        Some(SecStr::from(""))
    }
}

impl WidgetImpl for Window {}

impl WindowImpl for Window {}

impl ApplicationWindowImpl for Window {}
