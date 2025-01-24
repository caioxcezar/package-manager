mod imp;

use adw::{prelude::*, subclass::prelude::*};
use async_channel::unbounded;
use gtk::{
    gio,
    glib::{self, clone, GString, Object},
};
use std::{
    cell::{Ref, RefMut},
    thread::spawn,
};

use anyhow::{anyhow, Context, Result};
use rust_fuzzy_search::fuzzy_compare;
use secstr::{SecStr, SecVec};

use crate::{
    application,
    backend::{command::CommandStream, settings},
    backend::{package_object::PackageObject, provider::ProviderKind},
    messagebox,
};

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &application::PackageManagerApplication) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn setup_splash(&self) -> Result<()> {
        let obj = self.imp();

        obj.header_bar.set_visible(false);

        let img = gtk::Image::from_resource("/org/caioxcezar/packagemanager/package_manager.svg");
        let paintable = img
            .paintable()
            .context("Failed to load image package_manager.svg")?;
        obj.splash.set_paintable(Some(&paintable));

        obj.filter_list.set_incremental(true);

        Ok(())
    }

    // FIXME Sorter not working
    fn setup_sorter(&self) {
        let obj = self.imp();

        let sorter = sorter_installed_package();
        obj.column_installed.set_sorter(Some(&sorter));

        let sorter = sorter_string_package("name");
        obj.column_name.set_sorter(Some(&sorter));

        let sorter = sorter_string_package("version");
        obj.column_version.set_sorter(Some(&sorter));

        let sorter = sorter_string_package("repository");
        obj.column_repository.set_sorter(Some(&sorter));
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
            move |_| {
                if let Err(err) = window.handle_dropdown_changed() {
                    messagebox::alert(
                        "Failed to change the dropdown",
                        &format!("{err:?}"),
                        &window,
                    );
                };
            }
        ));

        obj.single_selection.connect_selection_changed(clone!(
            #[weak(rename_to = window)]
            self,
            move |grid, _position, _n_items| {
                if let Err(err) = window.handle_selection_changed(grid) {
                    messagebox::alert(
                        "Failed to change the dropdown",
                        &format!("{err:?}"),
                        &window,
                    );
                };
            }
        ));

        obj.action.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_button| {
                glib::spawn_future_local(async move {
                    if let Err(err) = window.handle_action().await {
                        messagebox::alert("Failed to execute action", &format!("{err:?}"), &window);
                    }
                });
            }
        ));

        obj.update.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_button| {
                glib::spawn_future_local(async move {
                    if let Err(err) = window.handle_update().await {
                        messagebox::alert("Failed to update", &format!("{err:?}"), &window);
                    }
                });
            }
        ));

        obj.info_bar_button.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |button| {
                if let Err(err) = window.handle_info_bar_clicked(button) {
                    messagebox::alert("Failed to change page", &format!("{err:?}"), &window);
                }
            }
        ));

        obj.search_entry.connect_search_changed(clone!(
            #[weak(rename_to = window)]
            self,
            move |entry| {
                if let Err(err) = window.handle_search(entry) {
                    messagebox::alert("Error while searching", &format!("{err:?}"), &window);
                }
            }
        ));
    }

    fn setup_data(&self) {
        let obj = self.imp();

        let providers = ProviderKind::available_providers();

        let model = providers
            .iter()
            .map(|provider| provider.name())
            .collect::<gtk::StringList>();

        if providers.is_empty() {
            obj.update_all.set_sensitive(false);
            obj.update.set_sensitive(false);
        }
        obj.providers.replace(providers);
        obj.dropdown_provider.set_model(Some(&model));
    }

    async fn handle_update_all(&self) -> Result<()> {
        let obj = self.imp();

        let mut password = obj.password.borrow().clone();

        let some_root_required = obj
            .providers
            .borrow()
            .iter()
            .any(|provider| provider.is_root_required());

        if some_root_required && password.is_none() {
            password = messagebox::ask_password(&self).await;
            obj.password.replace(password.clone());
            if password.is_none() {
                return Err(anyhow!("Failed to get password"));
            }
        }

        self.goto_command()?;

        let providers = obj.providers.borrow().clone();
        let count = providers.len();
        for (index, provider) in providers.iter().enumerate() {
            let stream = provider.update(password.clone())?;
            let join_handle = self.write_command_page(index == count - 1, stream);
            let _ = join_handle.await;
        }

        Ok(())
    }

    fn handle_dropdown_changed(&self) -> Result<()> {
        let obj = self.imp();

        let dropdown_text = obj
            .dropdown_provider
            .selected_item()
            .and_downcast::<gtk::StringObject>()
            .context("Failed to get current provider")?
            .string();

        self.update_model(&dropdown_text)?;

        let store = self.provider().model()?;
        let sorter = obj.column_view.sorter();
        let model = gtk::SortListModel::new(Some(store), sorter);
        obj.filter_list.set_model(Some(&model));
        obj.single_selection.set_model(Some(&obj.filter_list));

        obj.header_bar.set_visible(true);
        let current_page = match obj.stack.visible_child_name() {
            Some(v) => v.to_string(),
            None => "splash".to_string(),
        };
        if current_page == "splash" {
            let widget = self.page_by_name("main_page")?;
            obj.stack.set_visible_child(&widget);
        }
        Ok(())
    }

    fn handle_selection_changed(&self, grid: &gtk::SingleSelection) -> Result<()> {
        let obj = self.imp();

        let item = grid
            .selected_item()
            .and_downcast::<PackageObject>()
            .context("Failed to get item")?;
        let provider = self.provider();
        let info = provider.package_info(item.qualifiedName())?;
        let buffer = gtk::TextBuffer::builder().text(info).build();

        obj.text_box.set_buffer(Some(&buffer));
        obj.text_box.set_visible(true);

        if item.installed() {
            obj.action.set_label("Remove");
        } else {
            obj.action.set_label("Install");
        }
        obj.action.set_sensitive(true);

        Ok(())
    }

    async fn handle_action(&self) -> Result<()> {
        let obj = self.imp();

        let item = obj
            .single_selection
            .selected_item()
            .and_downcast::<PackageObject>()
            .context("Failed to get item")?;
        let buffer = gtk::TextBuffer::builder().text("").build();
        let action = obj
            .action
            .label()
            .context("Unable to identify the action (Install or Remove)")?;
        obj.text_command.set_buffer(Some(&buffer));
        let password = self.password().await.context("Failed to get password")?;
        self.goto_command()?;

        let stream = match action.as_str() {
            "Install" => self
                .provider()
                .install(Some(password), item.qualifiedName()),
            "Remove" => self.provider().remove(Some(password), item.qualifiedName()),
            _ => Err(anyhow!("Invalid Action. ")),
        }?;

        let _ = self.write_command_page(true, stream);

        Ok(())
    }

    async fn handle_update(&self) -> Result<()> {
        let password = self.password().await.context("Failed to get password")?;

        self.goto_command()?;

        let stream = self.provider().update(Some(password))?;
        let _ = self.write_command_page(true, stream);

        Ok(())
    }

    fn handle_info_bar_clicked(&self, _: &gtk::Button) -> Result<()> {
        let obj = self.imp();

        let widget = self.page_by_name("main_page")?;
        obj.info_bar.set_visible(false);
        obj.stack.set_visible_child(&widget);
        obj.search_entry.set_sensitive(true);
        obj.update.set_sensitive(true);
        obj.dropdown_provider.set_sensitive(true);

        Ok(())
    }

    fn handle_search(&self, search: &gtk::SearchEntry) -> Result<()> {
        let obj = self.imp();

        obj.single_selection.unselect_all();
        let value = search.text();
        let use_fuzzy = settings::get()?.fuzzy_search;

        let filter = if use_fuzzy {
            fuzzy_search(value)
        } else {
            simple_search(value)
        };
        obj.filter_list.set_filter(Some(&filter));
        Ok(())
    }

    fn update_model(&self, provider_name: &str) -> Result<()> {
        let mut providers = self.imp().providers.borrow_mut();
        let provider = providers
            .iter_mut()
            .find(|provider| provider.name().eq(&provider_name))
            .context("Provider not found")?;
        provider.update_packages()
    }

    fn write_command_page(&self, finish: bool, mut stream: CommandStream) -> glib::JoinHandle<()> {
        let (sender, receiver) = unbounded();
        let obj = self.imp();

        let buffer = gtk::TextBuffer::builder().text("").build();
        obj.text_command.set_buffer(Some(&buffer));
        let _ = buffer.connect_changed(clone!(
            #[weak(rename_to = text_command)]
            obj.text_command,
            move |buff| {
                let mut end = buff.end_iter();
                text_command.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
            }
        ));

        obj.text_command.set_buffer(Some(&buffer));

        spawn(move || {
            while let Some(value) = stream.next() {
                let _ = sender.send_blocking(value);
            }
            let message = match stream.close() {
                Ok(_) => "Command completed successfully. ".to_string(),
                Err(err) => format!("{err:?}\nCommand ended with failure. "),
            };
            let _ = sender.send_blocking(message);
        });

        glib::spawn_future_local(clone!(
            #[weak(rename_to = window)]
            self,
            async move {
                while let Ok(result) = receiver.recv().await {
                    buffer.insert(&mut buffer.end_iter(), &format!("{result}\n"));
                }
                if !finish {
                    return;
                }
                let obj = window.imp();
                obj.info_bar.set_visible(true);
                obj.info_bar_label.set_text("Finished.  ");
                let _ = window.handle_dropdown_changed();
            }
        ))
    }

    fn provider<'a>(&'a self) -> Ref<'a, ProviderKind> {
        let providers = self.imp().providers.borrow();
        Ref::map(providers, |providers| {
            providers
                .iter()
                .find(|provider| provider.name().eq(&self.dropdown_text()))
                .unwrap() // TODO remover
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
            let password = messagebox::ask_password(&self).await;
            obj.password.replace(password.clone());
            password
        } else {
            Some(SecStr::from(""))
        }
    }

    pub fn goto_command(&self) -> Result<()> {
        let obj = self.imp();

        let widget = self.page_by_name("command_page")?;
        obj.stack.set_visible_child(&widget);
        obj.search_entry.set_sensitive(false);
        obj.update.set_sensitive(false);
        obj.dropdown_provider.set_sensitive(false);
        Ok(())
    }

    fn page_by_name(&self, name: &str) -> Result<gtk::Widget> {
        self.imp()
            .stack
            .child_by_name(name)
            .context("Failed to get page")
    }
}

fn sorter_string_package(name: &str) -> gtk::StringSorter {
    gtk::StringSorter::builder()
        .ignore_case(true)
        .expression(gtk::PropertyExpression::new(
            PackageObject::static_type(),
            gtk::Expression::NONE,
            name,
        ))
        .build()
}

fn sorter_installed_package() -> gtk::CustomSorter {
    gtk::CustomSorter::new(move |obj1, obj2| {
        let package_1 = obj1
            .downcast_ref::<PackageObject>()
            .expect("The object needs to be of type `PackageObject`.");
        let package_2 = obj2
            .downcast_ref::<PackageObject>()
            .expect("The object needs to be of type `PackageObject`.");

        let bool_1 = package_1.installed();
        let bool_2 = package_2.installed();

        bool_2.cmp(&bool_1).into()
    })
}

fn fuzzy_search(value: GString) -> gtk::CustomFilter {
    gtk::CustomFilter::new(move |obj| {
        if value.is_empty() {
            true
        } else if let Some(obj) = obj.downcast_ref::<PackageObject>() {
            let prec = fuzzy_compare(&obj.qualifiedName(), &value);
            prec > 0.115
        } else {
            false
        }
    })
}

fn simple_search(value: GString) -> gtk::CustomFilter {
    let value = value.to_ascii_lowercase();
    gtk::CustomFilter::new(move |obj| {
        if value.is_empty() {
            true
        } else if let Some(obj) = obj.downcast_ref::<PackageObject>() {
            obj.qualifiedName().contains(&value)
        } else {
            false
        }
    })
}
