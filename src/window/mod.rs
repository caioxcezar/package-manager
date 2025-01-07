mod imp;

use std::{
    cell::Ref,
    thread::{self, JoinHandle},
};

use adw::subclass::prelude::*;
use glib::{clone, Object};
use gtk::{
    gio, glib,
    prelude::{
        ButtonExt, Cast, CastNone, ListModelExt, SelectionModelExt, TextBufferExt, TextViewExt,
        WidgetExt,
    },
    SingleSelection, StringList, TextBuffer,
};
use secstr::{SecStr, SecVec};

use crate::{
    application,
    backend::{package_object::PackageObject, provider::ProviderKind},
    messagebox,
};

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Application, gtk::Window, gtk::Widget, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &application::PackageManagerApplication) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn setup_splash(&self) {
        let obj = self.imp();

        obj.header_bar.set_visible(false);

        let img = gtk::Image::from_resource("/org/caioxcezar/packagemanager/package_manager.svg");
        let paintable = img.paintable().unwrap();
        obj.splash.set_paintable(Some(&paintable));

        obj.filter_list.set_incremental(true);
    }

    fn setup_signals(&self) {
        let obj = self.imp();

        obj.update_all.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                glib::spawn_future_local(async move { window.handle_update_all().await });
            }
        ));

        obj.dropdown_provider.connect_selected_item_notify(clone!(
            #[weak(rename_to = window)]
            self,
            move |dropdown| {
                window.handle_dropdown_changed(dropdown);
            }
        ));

        obj.single_selection.connect_selection_changed(clone!(
            #[weak(rename_to = window)]
            self,
            move |grid, position, _n_items| {
                window.handle_selection_changed(grid, position);
            }
        ));

        obj.action.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_button| {
                glib::spawn_future_local(async move { window.handle_action().await });
            }
        ));

        obj.update.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_button| {
                glib::spawn_future_local(async move {
                    window.handle_update().await;
                });
            }
        ));

        obj.info_bar_button.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |button| {
                window.handle_info_bar_clicked(button);
            }
        ));
    }

    fn setup_data(&self) {
        let obj = self.imp();

        let providers = ProviderKind::available_providers();

        let model = providers
            .iter()
            .map(|provider| provider.name())
            .collect::<StringList>();

        if providers.is_empty() {
            obj.update_all.set_sensitive(false);
            obj.update.set_sensitive(false);
        }
        obj.providers.replace(providers);
        obj.dropdown_provider.set_model(Some(&model));
    }

    async fn handle_update_all(&self) {
        let obj = self.imp();

        let mut password = None;
        let some_root_required = obj
            .providers
            .borrow()
            .iter()
            .any(|provider| provider.is_root_required());
        if some_root_required {
            password = messagebox::ask_password(&self.window()).await;
        }
        obj.password.replace(password.clone());
        let password = match password {
            Some(value) => value,
            _ => return,
        };
        self.goto_command();

        let text_command = obj.text_command.clone();

        let buffer = TextBuffer::builder().text("").build();
        let _ = buffer.connect_changed(move |buff| {
            let mut end = buff.end_iter();
            text_command.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
        });

        obj.text_command.set_buffer(Some(&buffer));
        for provider in obj.providers.borrow().iter() {
            let handle = provider.update(&password, &buffer);
            let _ = handle.join();
        }
        obj.dropdown_provider
            .set_selected(obj.dropdown_provider.selected());
        obj.info_bar_label.set_text("Finished");
        obj.info_bar.set_visible(true);
    }

    fn handle_dropdown_changed(&self, dropdown: &gtk::DropDown) {
        let obj = self.imp();

        let dropdown_text = match dropdown.selected_item().and_downcast::<gtk::StringObject>() {
            Some(value) => value.string(),
            None => return,
        };

        self.update_model(&dropdown_text);

        let provider = match self.provider().model() {
            Ok(value) => value,
            Err(value) => {
                messagebox::alert("Error while changing provider", &value, &self.window());
                return;
            }
        };
        obj.filter_list.set_model(Some(&provider));
        obj.single_selection.set_model(Some(&obj.filter_list));

        obj.header_bar.set_visible(true);
        let current_page = obj.stack.visible_child_name();
        let current_page = match current_page {
            Some(v) => v.to_string(),
            None => "splash".to_string(),
        };
        if current_page == "splash" {
            let widget = obj.stack.child_by_name("main_page").unwrap();
            obj.stack.set_visible_child(&widget);
        }
    }

    fn handle_selection_changed(&self, grid: &SingleSelection, position: u32) {
        let obj = self.imp();

        let item = match grid.item(position) {
            Some(value) => value.downcast::<PackageObject>().unwrap(),
            None => return,
        };
        let provider = self.provider();
        let info = provider.package_info(&item.qualifiedName());
        let buffer = TextBuffer::builder().text(info).build();

        obj.text_box.set_buffer(Some(&buffer));
        obj.text_box.set_visible(true);

        if item.installed() {
            obj.action.set_label("Remove");
        } else {
            obj.action.set_label("Install");
        }
        obj.action.set_sensitive(true);
    }

    async fn handle_action(&self) {
        let obj = self.imp();

        let item = match obj.single_selection.selected_item() {
            Some(value) => value.downcast::<PackageObject>().unwrap(),
            None => return,
        };
        let buffer = TextBuffer::builder().text("").build();
        let action = obj.action.label().unwrap();
        obj.text_command.set_buffer(Some(&buffer));
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

    async fn handle_update(&self) {
        let obj = self.imp();

        let buffer = TextBuffer::builder().text("").build();
        obj.text_command.set_buffer(Some(&buffer));
        let password = match self.password().await {
            Some(value) => value,
            _ => return,
        };
        self.goto_command();
        let handle = self.provider().update(&password, &buffer);
        self.command_finished(handle);
    }

    fn handle_info_bar_clicked(&self, _: &gtk::Button) {
        let obj = self.imp();

        let widget = obj.stack.child_by_name("main_page").unwrap();
        obj.info_bar.set_visible(false);
        obj.stack.set_visible_child(&widget);
        obj.search_entry.set_sensitive(true);
        obj.update.set_sensitive(true);
        obj.dropdown_provider.set_sensitive(true);
    }

    fn update_model(&self, provider_name: &str) {
        let mut providers = self.imp().providers.borrow_mut();
        let provider = providers
            .iter_mut()
            .find(|provider| provider.name().eq(&provider_name))
            .unwrap();
        let _ = provider.update_packages();
    }

    fn window(&self) -> gtk::Window {
        let widget = self.imp().search_entry.clone().upcast::<gtk::Widget>();
        widget
            .root()
            .map(|root| root.downcast::<gtk::Window>().unwrap())
            .unwrap()
    }

    fn provider<'a>(&'a self) -> Ref<'a, ProviderKind> {
        let providers = self.imp().providers.borrow();
        Ref::map(providers, |providers| {
            providers
                .iter()
                .find(|provider| provider.name().eq(&self.dropdown_text()))
                .unwrap()
        })
    }

    fn dropdown_text(&self) -> String {
        match self
            .imp()
            .dropdown_provider
            .selected_item()
            .and_downcast::<gtk::StringObject>()
        {
            Some(value) => value.string().to_string(),
            None => return "".to_string(),
        }
    }

    async fn password(&self) -> Option<SecVec<u8>> {
        let obj = self.imp();
        let password = obj.password.borrow().clone();
        if password.is_some() {
            return password;
        }
        let provider = self.provider();
        let is_root_required = provider.is_root_required();
        if is_root_required {
            let password = messagebox::ask_password(&self.window()).await;
            obj.password.replace(password.clone());
            password
        } else {
            Some(SecStr::from(""))
        }
    }

    pub fn goto_command(&self) {
        let obj = self.imp();

        let widget = obj.stack.child_by_name("command_page").unwrap();
        obj.stack.set_visible_child(&widget);
        obj.search_entry.set_sensitive(false);
        obj.update.set_sensitive(false);
        obj.dropdown_provider.set_sensitive(false);
    }

    fn command_finished(&self, handle: JoinHandle<bool>) {
        let obj = self.imp();
        let u32_provider = obj.dropdown_provider.selected();
        // TODO corrigir
        // combobox_provider.set_selected(None);
        let (sender, receiver) = async_channel::unbounded();
        thread::spawn(move || {
            let res = handle.join().unwrap();
            if res {
                let _ = sender.send_blocking("Finalizado com sucesso! ");
            } else {
                let _ = sender.send_blocking("Finalizado com erro ");
            }
        });
        glib::spawn_future_local(clone!(
            #[weak]
            obj,
            async move {
                if let Ok(res) = receiver.recv().await {
                    obj.info_bar.set_visible(true);
                    obj.info_bar_label.set_text(res);
                    obj.dropdown_provider.set_selected(u32_provider);
                }
            }
        ));
    }
}
