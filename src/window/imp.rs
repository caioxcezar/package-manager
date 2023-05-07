use crate::backend::package_object::PackageObject;
use crate::backend::providers::{self, Providers};
use crate::{grid_check, grid_text, messagebox};
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate, TextBuffer};
use secstr::{SecStr, SecVec};
use std::cell::RefCell;
use std::thread::{self, JoinHandle};
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
    providers: RefCell<Providers>,
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
            let providers = providers::init();

            for provider in &providers.list {
                let p = provider.name();
                self.combobox_provider.append_text(&p);
            }

            if providers.list.len() == 0 {
                self.update_all.set_sensitive(false);
                self.update.set_sensitive(false);
            }
            self.providers.replace(providers);
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
        let providers = self.providers.borrow();
        let info = providers.package_info(&item.qualifiedName(), &self.combobox_text());
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

        if item.installed() {
            self.action.set_label("Remove");
        } else {
            self.action.set_label("Install");
        }
        self.action.set_sensitive(true);
    }
    #[template_callback]
    async fn handle_update_all(&self, _button: gtk::Button) {
        let providers = self.providers.borrow();
        let mut password = None;
        if providers.some_root_required() {
            password = messagebox::ask_password(self.window()).await;
        }
        let password = match password {
            Some(value) => value,
            _ => return,
        };
        self.goto_command();

        let combobox_provider = self.combobox_provider.clone();
        let info_bar_label = self.info_bar_label.clone();
        let info_bar = self.info_bar.clone();
        let text_command = self.text_command.clone();

        let u32_provider = self.combobox_provider.active();
        let buffer = TextBuffer::builder().text("").build();

        let _ = buffer.connect_changed(move |buff| {
            let start = buff.iter_at_line(buff.line_count() - 1).unwrap();
            let mut end = buff.end_iter();
            let line = buff.text(&start, &end, false);
            let line = line.as_str();
            if line.contains(":::: All Updated ::::") {
                combobox_provider.set_active(u32_provider);
                info_bar_label.set_text("Finished");
                info_bar.set_visible(true);
            }

            text_command.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
        });

        self.text_command.set_buffer(Some(&buffer));
        providers.update_all(&buffer, &password);
    }
    #[template_callback]
    async fn handle_action(&self, button: gtk::Button) {
        let item = self.selected_item();
        let item = match item {
            Some(value) => value,
            None => return,
        };
        let provider_name = self.combobox_text();
        let buffer = TextBuffer::builder().text("").build();
        let action = button.label().unwrap();
        let providers = self.providers.borrow();
        self.text_command.set_buffer(Some(&buffer));
        let password = match self.password(&providers).await {
            Some(value) => value,
            _ => return,
        };
        self.goto_command();
        let handle = match action.as_str() {
            "Install" => {
                providers.install(&provider_name, &item.qualifiedName(), &buffer, &password)
            }
            "Remove" => providers.remove(&provider_name, &item.qualifiedName(), &buffer, &password),
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
        let combobox_text = combobox_text.as_str();
        let mut providers = self.providers.borrow_mut();
        let provider = match providers.model(combobox_text) {
            Ok(value) => value,
            Err(value) => {
                messagebox::error("Error while changing provider", &value, self.window());
                return;
            }
        };
        self.filter_list.set_model(Some(&provider));
        self.single_selection.set_model(Some(&self.filter_list));

        self.header_bar.show();
        let widget = self.stack.child_by_name("main_page").unwrap();
        self.stack.set_visible_child(&widget);
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
        signal_text_bind_handler(item, entry.name().to_string());
    }
    #[template_callback]
    fn signal_version_bind_handler(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
        let entry = item.item().and_downcast::<PackageObject>().unwrap();
        signal_text_bind_handler(item, entry.version().to_string());
    }
    #[template_callback]
    fn signal_repository_bind_handler(_factory: gtk::SignalListItemFactory, item: gtk::ListItem) {
        let entry = item.item().and_downcast::<PackageObject>().unwrap();
        signal_text_bind_handler(item, entry.repository().to_string());
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
        let providers = self.providers.borrow();
        let buffer = TextBuffer::builder().text("").build();
        self.text_command.set_buffer(Some(&buffer));
        let password = match self.password(&providers).await {
            Some(value) => value,
            _ => return,
        };
        let str_prd = self.combobox_text();
        self.goto_command();
        let handle = providers.update(&str_prd, &buffer, &password);
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
    async fn password(&self, providers: &Providers) -> Option<SecVec<u8>> {
        if providers.is_root_required(&self.combobox_text()) {
            return messagebox::ask_password(self.window()).await;
        }
        Some(SecStr::from(""))
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
