use gtk::{prelude::*, AlertDialog};
use secstr::{SecStr, SecVec};

use crate::{backend::command, window::Window};

pub fn alert(title: &str, body: &str, window: &Window) {
    AlertDialog::builder()
        .message(title)
        .detail(body)
        .modal(true)
        .build()
        .show(Some(window));
}

pub async fn ask_password(window: &Window) -> Option<SecVec<u8>> {
    let (sender, receiver) = async_channel::unbounded();

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

    let dialog = gtk::Window::builder()
        .transient_for(window)
        .child(&child)
        .modal(true)
        .build();

    let btn_sender = sender.clone();
    button.connect_clicked(move |_| {
        let _ = btn_sender.send_blocking(true);
    });

    let pass_sender = sender.clone();
    password.connect_activate(move |_| {
        let _ = pass_sender.send_blocking(true);
    });

    dialog.connect_close_request(move |_| {
        let _ = sender.send_blocking(false);
        gtk::glib::Propagation::Proceed
    });

    dialog.set_visible(true);

    let response = receiver.recv().await;
    let _ = match response {
        Ok(value) => value,
        Err(err) => {
            println!("{:?}", err);
            return None;
        }
    };

    let pass = password.text().to_string();
    let check_password = command::run(&format!("echo '{}' | sudo -S su", &pass));
    dialog.close();
    let res = match check_password {
        Ok(_) => SecStr::from(pass),
        Err(err) => {
            alert(
                "Wrong Password",
                &format!("Please provide the currect password\n.{:?}", err),
                &window,
            );
            return None;
        }
    };
    let _ = command::run("sudo -k");
    Some(res)
}
