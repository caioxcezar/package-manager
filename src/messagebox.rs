use gtk::{prelude::*, Dialog, ResponseType};
use secstr::{SecStr, SecVec};

use crate::backend::command;

pub fn error(title: &str, body: &str, window: Option<gtk::Window>) {
    let mut dialog = gtk::MessageDialog::builder()
        .text(title)
        .message_type(gtk::MessageType::Error)
        .secondary_text(body)
        .modal(true)
        .buttons(gtk::ButtonsType::Ok);
    if let Some(window) = window {
        dialog = dialog.transient_for(&window);
    }
    let dialog = dialog.build();
    dialog.run_async(|obj, _| {
        obj.close();
    });
}

pub async fn ask_password(window: Option<gtk::Window>) -> Option<SecVec<u8>> {
    let window_clone = window.clone();
    let mut dialog = Dialog::builder().modal(true);
    if let Some(window) = window {
        dialog = dialog.transient_for(&window);
    }
    let dialog = dialog.build();
    let child = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(5)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_start(10)
        .build();
    let text = gtk::Label::builder().label("Password").build();
    let password = gtk::Entry::builder().text("").visibility(false).build();
    let button = gtk::Button::builder().label("Ok").build();
    child.append(&text);
    child.append(&password);
    child.append(&button);
    dialog.set_child(Some(&child));

    let dialog_button = dialog.clone();
    let dialog_password = dialog.clone();

    button.connect_clicked(move |_| {
        dialog_button.response(ResponseType::Ok);
    });

    password.connect_activate(move |_| {
        dialog_password.response(ResponseType::Ok);
    });

    let response = dialog.run_future().await;
    if response == ResponseType::Ok {
        let pass = password.text().to_string();
        let check_password = command::run(&format!("echo '{}' | sudo -S su", &pass));
        dialog.close();
        let res = match check_password {
            Ok(_) => Some(SecStr::from(pass)),
            _ => {
                error(
                    "Wrong Password",
                    "Please provide the currect password",
                    window_clone,
                );
                return None;
            }
        };
        let _ = command::run("sudo -k");
        return res;
    }
    None
}
