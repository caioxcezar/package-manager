use gtk::{traits::WidgetExt, Dialog, Label};

pub fn show(message: &str) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let label = Label::builder()
        .label(message)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let dialog = Dialog::builder().child(&label).build();
    dialog.show();
}
