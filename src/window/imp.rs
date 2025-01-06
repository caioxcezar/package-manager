use crate::{
    backend::{package_object::PackageObject, provider::ProviderKind},
    grid_check, grid_text, messagebox,
};
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{glib, prelude::*, CompositeTemplate, TextBuffer};
use secstr::{SecStr, SecVec};
use std::{
    cell::{Ref, RefCell, RefMut},
    thread::{self, JoinHandle},
};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/caioxcezar/packagemanager/window.ui")]
pub struct Window {
    #[template_child]
    pub header_bar: TemplateChild<gtk::HeaderBar>,
    #[template_child]
    pub stack: TemplateChild<gtk::Stack>,
    #[template_child]
    pub column_view: TemplateChild<gtk::ColumnView>,
    #[template_child]
    pub search_entry: TemplateChild<gtk::SearchEntry>,
    #[template_child]
    pub combobox_provider: TemplateChild<gtk::ComboBoxText>,
    #[template_child]
    pub update_all: TemplateChild<gtk::Button>,
    #[template_child]
    pub action: TemplateChild<gtk::Button>,
    #[template_child]
    pub update: TemplateChild<gtk::Button>,
    #[template_child]
    pub text_box: TemplateChild<gtk::TextView>,
    #[template_child]
    pub single_selection: TemplateChild<gtk::SingleSelection>,
    #[template_child]
    pub text_command: TemplateChild<gtk::TextView>,
    #[template_child]
    pub info_bar: TemplateChild<gtk::InfoBar>,
    #[template_child]
    pub info_bar_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub splash: TemplateChild<gtk::Picture>,
    pub filter_list: gtk::FilterListModel,
    providers: RefCell<Vec<ProviderKind>>,
    password: RefCell<Option<SecVec<u8>>>,
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
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        self.header_bar.hide();
        let img = gtk::Image::new();
        img.set_from_resource(Some("/org/caioxcezar/packagemanager/package_manager.svg"));
        let paintable = img.paintable().unwrap();
        self.splash.set_paintable(Some(&paintable));

        self.filter_list.set_incremental(true);
        {
            let providers = ProviderKind::available_providers();

            for provider in &providers {
                self.combobox_provider.append_text(&provider.name());
            }

            if providers.is_empty() {
                self.update_all.set_sensitive(false);
                self.update.set_sensitive(false);
            }
            // self.providers.replace(providers);
        }
        let combobox_provider = self.combobox_provider.clone();
        glib::source::idle_add_local_once(move || {
            combobox_provider.set_active(Some(0));
        });
    }
}
#[gtk::template_callbacks]
impl Window {
    fn selected_item(&self) -> Option<PackageObject> {
        let item = self.single_selection.selected_item();
        match item {
            Some(value) => {
                let value = value.downcast::<PackageObject>().unwrap();
                Some(value)
            }
            None => None,
        }
    }
    #[template_callback]
    fn selection_changed(&self) {
        let item = self.selected_item();
        let item = match item {
            Some(value) => value,
            None => return,
        };
        let provider = self.provider();
        let info = provider.package_info(&item.qualifiedName());
        let buffer = TextBuffer::builder().text(info).build();
        self.text_box.set_buffer(Some(&buffer));
        self.text_box.set_visible(true);

        if item.installed() {
            self.action.set_label("Remove");
        } else {
            self.action.set_label("Install");
        }
        self.action.set_sensitive(true);
    }
    #[template_callback]
    async fn handle_update_all(&self, _button: gtk::Button) {
        let mut password = None;
        let some_root_required = self
            .providers
            .borrow()
            .iter()
            .any(|provider| provider.is_root_required());
        if some_root_required {
            password = messagebox::ask_password(self.window()).await;
        }
        self.password.replace(password.clone());
        let password = match password {
            Some(value) => value,
            _ => return,
        };
        self.goto_command();

        let text_command = self.text_command.clone();

        let buffer = TextBuffer::builder().text("").build();
        let _ = buffer.connect_changed(move |buff| {
            let mut end = buff.end_iter();
            text_command.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
        });

        self.text_command.set_buffer(Some(&buffer));
        for provider in self.providers.borrow().iter() {
            let handle = provider.update(&password, &buffer);
            let _ = handle.join();
        }
        self.combobox_provider
            .set_active(self.combobox_provider.active());
        self.info_bar_label.set_text("Finished");
        self.info_bar.set_visible(true);
    }
    #[template_callback]
    async fn handle_action(&self, button: gtk::Button) {
        let item = self.selected_item();
        let item = match item {
            Some(value) => value,
            None => return,
        };
        let buffer = TextBuffer::builder().text("").build();
        let action = button.label().unwrap();
        self.text_command.set_buffer(Some(&buffer));
        let password = match self.password().await {
            Some(value) => value,
            _ => return,
        };
        self.goto_command();
        let handle = match action.as_str() {
            "Install" => self
                .provider()
                .install(&password, &item.qualifiedName(), &buffer),
            "Remove" => self
                .provider()
                .remove(&password, &item.qualifiedName(), &buffer),
            _ => return,
        };
        self.command_finished(handle);
    }
    #[template_callback]
    fn handle_combobox_changed(&self, combobox: gtk::ComboBoxText) {
        let combobox_text = match combobox.active_text() {
            Some(value) => value,
            None => return,
        };

        self.update_model(&combobox_text);

        let provider = match self.provider().model() {
            Ok(value) => value,
            Err(value) => {
                messagebox::error("Error while changing provider", &value, self.window());
                return;
            }
        };
        self.filter_list.set_model(Some(&provider));
        self.single_selection.set_model(Some(&self.filter_list));

        self.header_bar.show();
        let current_page = self.stack.visible_child_name();
        let current_page = match current_page {
            Some(v) => v.to_string(),
            None => "splash".to_string(),
        };
        if current_page == "splash" {
            let widget = self.stack.child_by_name("main_page").unwrap();
            self.stack.set_visible_child(&widget);
        }
    }
    #[template_callback]
    fn signal_check_setup_handler(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
        item.set_child(Some(&grid_check::GridCheck::new()))
    }
    #[template_callback]
    fn signal_text_setup_handler(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
        item.set_child(Some(&grid_text::GridText::new()))
    }
    #[template_callback]
    fn signal_installed_bind_handler(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
        let entry = item.item().and_downcast::<PackageObject>().unwrap();
        let child = item
            .child()
            .and_downcast::<grid_check::GridCheck>()
            .unwrap();
        let ent = grid_check::Entry {
            check: entry.installed(),
            sensitive: false,
        };
        child.set_entry(&ent);
    }
    #[template_callback]
    fn signal_name_bind_handler(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
        let entry = item.item().and_downcast::<PackageObject>().unwrap();
        signal_text_bind_handler(item, entry.name());
    }
    #[template_callback]
    fn signal_version_bind_handler(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
        let entry = item.item().and_downcast::<PackageObject>().unwrap();
        signal_text_bind_handler(item, entry.version());
    }
    #[template_callback]
    fn signal_repository_bind_handler(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
        let entry = item.item().and_downcast::<PackageObject>().unwrap();
        signal_text_bind_handler(item, entry.repository());
    }
    #[template_callback]
    fn handle_search(&self, search: &gtk::SearchEntry) {
        self.single_selection.unselect_all();
        let value = search.text().to_ascii_lowercase();
        let filter = gtk::CustomFilter::new(move |obj| {
            let obj = obj.downcast_ref::<PackageObject>().unwrap();
            obj.qualifiedName().to_ascii_lowercase().contains(&value)
        });
        self.filter_list.set_filter(Some(&filter));
    }
    #[template_callback]
    async fn handle_update(&self, _button: gtk::Button) {
        let buffer = TextBuffer::builder().text("").build();
        self.text_command.set_buffer(Some(&buffer));
        let password = match self.password().await {
            Some(value) => value,
            _ => return,
        };
        self.goto_command();
        let handle = self.provider().update(&password, &buffer);
        self.command_finished(handle);
    }
    #[template_callback]
    fn handle_info_bar_response(&self, _: i32) {
        let widget = self.stack.child_by_name("main_page").unwrap();
        self.info_bar.set_visible(false);
        self.stack.set_visible_child(&widget);
        self.search_entry.set_sensitive(true);
        self.update.set_sensitive(true);
        self.combobox_provider.set_sensitive(true);
    }
    #[template_callback]
    fn handle_focused(&self) {
        self.combobox_provider.set_active(Some(0));
    }
    fn combobox_text(&self) -> String {
        let combobox_text = self.combobox_provider.active_text().unwrap();
        combobox_text.as_str().to_owned()
    }
    fn command_finished(&self, handle: JoinHandle<bool>) {
        let combobox_provider = self.combobox_provider.clone();
        let u32_provider = self.combobox_provider.active();
        let info_bar = self.info_bar.clone();
        let info_bar_label = self.info_bar_label.clone();
        combobox_provider.set_active(None);
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        thread::spawn(move || {
            let res = handle.join().unwrap();
            if res {
                let _ = tx.send("Finalizado com sucesso!");
            } else {
                let _ = tx.send("Finalizado com erro");
            }
        });
        rx.attach(None, move |res| {
            info_bar.set_visible(true);
            info_bar_label.set_text(res);
            combobox_provider.set_active(u32_provider);
            glib::Continue(false)
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
    async fn password(&self) -> Option<SecVec<u8>> {
        let password: Option<SecVec<u8>> = self.password.borrow().clone();
        if password.is_some() {
            return password;
        }
        let is_root_required;
        {
            let provider = self.provider();
            is_root_required = provider.is_root_required();
        }
        if is_root_required {
            let password = messagebox::ask_password(self.window()).await;
            self.password.replace(password.clone());
            password
        } else {
            Some(SecStr::from(""))
        }
    }
    fn provider<'a>(&'a self) -> Ref<'a, ProviderKind> {
        let providers = self.providers.borrow();
        Ref::map(providers, |providers| {
            providers
                .iter()
                .find(|provider| provider.name().eq(&self.combobox_text()))
                .unwrap()
        })
    }
    fn update_model(&self, provider_name: &str) {
        let providers = self.providers.borrow_mut();
        let mut provider = RefMut::map(providers, |providers| {
            providers
                .iter_mut()
                .find(|provider| provider.name().eq(&provider_name))
                .unwrap()
        });
        let _ = provider.update_packages();
    }
}
fn signal_text_bind_handler(item: gtk::ListItem, value: String) {
    let child = item.child().and_downcast::<grid_text::GridText>().unwrap();
    let ent = grid_text::Entry { name: value };
    child.set_entry(&ent);
}

impl WidgetImpl for Window {}

impl WindowImpl for Window {}

impl ApplicationWindowImpl for Window {}
