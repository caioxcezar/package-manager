use crate::backend::providers::{self, Providers};
use crate::messagebox;
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate, TextBuffer, TreeModelFilter};
use gtk::{prelude::*, TreeModelSort};
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
    pub tree_view: TemplateChild<gtk::TreeView>,
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
    pub tree_selection: TemplateChild<gtk::TreeSelection>,
    #[template_child]
    pub text_command: TemplateChild<gtk::TextView>,
    #[template_child]
    pub info_bar: TemplateChild<gtk::InfoBar>,
    #[template_child]
    pub info_bar_label: TemplateChild<gtk::Label>,
    providers: RefCell<Providers>,
    list_filter: RefCell<Option<TreeModelFilter>>,
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
    #[template_callback]
    fn treeview_selection_changed(&self, tree_selection: gtk::TreeSelection) {
        if let Some((model, iter)) = tree_selection.selected() {
            let package = model.get_value(&iter, 4 as i32).get::<String>().unwrap();
            let providers = self.providers.borrow();
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

            let installed = model.get_value(&iter, 0 as i32).get::<bool>().unwrap();

            if installed {
                self.action.set_label("Remove");
            } else {
                self.action.set_label("Install");
            }
            self.action.set_sensitive(true);
        }
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
        let buffer = TextBuffer::builder().text(&"").build();

        let _ = buffer.connect_changed(move |buff| {
            let start = buff.iter_at_line(buff.line_count() - 1).unwrap();
            let mut end = buff.end_iter();
            let line = buff.text(&start, &end, false);
            let line = line.as_str();
            if line.contains(":::: Updated All ::::") {
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
        if let Some((tree_model, tree_iter)) = self.tree_view.selection().selected() {
            let provider_name = self.combobox_text();
            let package = tree_model
                .get_value(&tree_iter, 4 as i32)
                .get::<String>()
                .unwrap();
            let buffer = TextBuffer::builder().text(&"").build();
            let action = button.label().unwrap();
            let providers = self.providers.borrow();
            self.text_command.set_buffer(Some(&buffer));
            let password = match self.password(&providers).await {
                Some(value) => value,
                _ => return,
            };
            self.goto_command();
            let handle = match action.as_str() {
                "Install" => providers.install(&provider_name, &package, &buffer, &password),
                "Remove" => providers.remove(&provider_name, &package, &buffer, &password),
                _ => return,
            };
            self.command_finished(handle);
        }
    }
    #[template_callback]
    fn handle_combobox_changed(&self, combobox: gtk::ComboBoxText) {
        if let Some(combobox_text) = combobox.active_text() {
            let mut providers = self.providers.borrow_mut();
            let combobox_text = combobox_text.as_str();
            self.update.set_sensitive(true);
            let provider = providers.model(combobox_text);
            let provider = match provider {
                Ok(value) => value,
                Err(value) => {
                    messagebox::error("Error while changing provider", &value, self.window());
                    return;
                }
            };
            let filter = TreeModelFilter::new(&provider, None);
            let sort = TreeModelSort::with_model(&filter);
            let search = self.search_entry.clone();
            filter.set_visible_func(move |model, iter| {
                let value = search.text();
                let value = value.as_str();
                let package = model.get_value(iter, 4 as i32).get::<String>().unwrap();
                package.contains(value)
            });
            self.tree_view.set_model(Some(&sort));
            self.list_filter.replace(Some(filter));
        }
    }
    #[template_callback]
    fn handle_search(&self, _search: &gtk::SearchEntry) {
        self.tree_selection.unselect_all();
        self.tree_selection.set_mode(gtk::SelectionMode::None);
        if let Ok(mut filter) = self.list_filter.try_borrow_mut() {
            let list_filter = match &mut *filter {
                Some(value) => value,
                _ => return,
            };
            list_filter.refilter();
            self.tree_selection.set_mode(gtk::SelectionMode::Single);
        }
    }
    #[template_callback]
    async fn handle_update(&self, _button: gtk::Button) {
        let providers = self.providers.borrow();
        let buffer = TextBuffer::builder().text(&"").build();
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

impl WidgetImpl for Window {}

impl WindowImpl for Window {}

impl ApplicationWindowImpl for Window {}
