use crate::backend::command;
use crate::backend::package::PendingPackage;
use crate::backend::providers::{self, Providers};
use crate::messagebox;
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate, TextBuffer};
use gtk::{prelude::*, ListStore};
use std::sync::Mutex;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/caioxcezar/packagemanager/window.ui")]
pub struct Window {
    #[template_child]
    pub header_bar: TemplateChild<gtk::HeaderBar>,
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
    pub providers: Mutex<Providers>,
    pub pending_packages: Mutex<Vec<PendingPackage>>,
    pub list_store: Mutex<Option<ListStore>>,
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

            *providers = providers::init();

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
                let info = providers.package_info(package, self.combobox_text());
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
            _ => messagebox::error("Toggle error", "List Store not found"),
        }
    }
    #[template_callback]
    fn handle_update_all(&self, _button: gtk::Button) {
        let buffer = TextBuffer::builder().text(&"").build();
        self.text_box.set_buffer(Some(&buffer));
        let _ = command::run_stream("cat /home/caior/paru_list".to_owned(), &buffer);
        self.text_box.set_visible(true);
    }
    #[template_callback]
    fn handle_action(&self, _: gtk::Button) {
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
        messagebox::info("Please, confirm the changes", &text, |value| {
            if value.eq(&gtk::ResponseType::Ok) {
            } else {
                println!("Canceling...");
            }
        });
        let text = self.combobox_text();
        let providers = self.providers.lock();
        if let Ok(providers) = providers {
            let buffer = TextBuffer::builder().text(&"").build();
            self.text_box.set_buffer(Some(&buffer));
            providers.update(text, &buffer);
        }
    }
    #[template_callback]
    fn handle_combobox_changed(&self, combobox: gtk::ComboBoxText) {
        let mut pending_packages = self.pending_packages.lock().unwrap();
        let mut list_store = self.list_store.lock().unwrap();
        let providers = self.providers.lock().unwrap();

        pending_packages.clear();
        let combobox_text = combobox.active_text().unwrap();
        let combobox_text = String::from(combobox_text.as_str());
        let provider = providers.get_model(combobox_text).unwrap();
        self.tree_view.set_model(Some(&provider));
        *list_store = Some(provider);
    }
    #[template_callback]
    fn handle_update(&self, _button: gtk::Button) {
        let providers = self.providers.lock();
        if let Ok(providers) = providers {
            let buffer = TextBuffer::builder().text(&"").build();
            self.text_box.set_buffer(Some(&buffer));
            providers.update(self.combobox_text(), &buffer);
        }
    }
    fn combobox_text(&self) -> String {
        let combobox_text = self.combobox_provider.active_text().unwrap();
        String::from(combobox_text.as_str())
    }
}

impl WidgetImpl for Window {}

impl WindowImpl for Window {}

impl ApplicationWindowImpl for Window {}
