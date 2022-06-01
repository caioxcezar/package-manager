use gtk::{prelude::*, Dialog, ResponseType};
use secstr::{SecStr, SecVec};

pub async fn info(title: &str, body: &str, window: Option<gtk::Window>) -> ResponseType {
    let mut dialog = gtk::MessageDialog::builder()
        .text(title)
        .secondary_text(body)
        .message_type(gtk::MessageType::Info)
        .modal(true)
        .buttons(gtk::ButtonsType::Ok);
    if let Some(window) = window {
        dialog = dialog.transient_for(&window);
    }
    let dialog = dialog.build();
    let future = dialog.run_future().await;
    dialog.close();
    future
}

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
    let password = gtk::Text::builder().text("").visibility(false).build();
    let button = gtk::Button::builder().label("Ok").build();
    child.append(&text);
    child.append(&password);
    child.append(&button);
    // dialog.set_parent(&*parent);
    dialog.set_child(Some(&child));
    let dialog_clone = dialog.clone();
    button.connect_clicked(move |_| {
        dialog.response(ResponseType::Ok);
        // dialog.close();
    });
    let response = dialog_clone.run_future().await;
    dialog_clone.close();
    if response == ResponseType::Ok {
        // TODO verificar se a senha está correta
        return Some(SecStr::from(password.text().to_string()));
    }
    None
}
