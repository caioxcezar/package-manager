use crate::{
    backend::{package_object::PackageObject, provider::ProviderKind},
    grid_check, grid_text,
};
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{
    glib::{self, clone},
    prelude::*,
    CompositeTemplate,
};
use secstr::SecVec;
use std::cell::RefCell;
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
    pub dropdown_provider: TemplateChild<gtk::DropDown>,
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
    pub info_bar: TemplateChild<gtk::Overlay>,
    #[template_child]
    pub info_bar_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub info_bar_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub splash: TemplateChild<gtk::Picture>,
    pub filter_list: gtk::FilterListModel,
    pub providers: RefCell<Vec<ProviderKind>>,
    pub password: RefCell<Option<SecVec<u8>>>,
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

        let obj = self.obj();

        obj.setup_splash();

        glib::source::idle_add_local_once(clone!(
            #[weak(rename_to = window)]
            self,
            move || {
                let obj = window.obj();

                obj.setup_signals();
                obj.setup_data();
            }
        ));
    }
}
#[gtk::template_callbacks]
impl Window {
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
    fn handle_focused(&self) {
        self.dropdown_provider.set_selected(0);
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
