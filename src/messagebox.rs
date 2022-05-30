use gtk::prelude::*;
use secstr::{SecStr, SecVec};

pub fn info(title: &str, body: &str, callback: fn(response: gtk::ResponseType)) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let dialog = gtk::MessageDialog::builder()
        .text(title)
        .secondary_text(body)
        .message_type(gtk::MessageType::Info)
        .modal(true)
        .buttons(gtk::ButtonsType::Ok)
        .build();
    dialog.run_async(move |obj, response| {
        callback(response);
        obj.close();
    });
}

pub fn error(title: &str, body: &str) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let dialog = gtk::MessageDialog::builder()
        .text(title)
        .message_type(gtk::MessageType::Error)
        .secondary_text(body)
        .modal(true)
        .buttons(gtk::ButtonsType::Ok)
        .build();
    dialog.run_async(|obj, _| {
        obj.close();
    });
}

pub fn ask_password() -> SecVec<u8> {
    SecStr::from("<<Uma Senha segura>>")
}
