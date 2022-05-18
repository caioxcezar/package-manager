use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/caioxcezar/packagemanager/window.ui")]
    pub struct PackageManagerWindow {
        // Template widgets
        #[template_child]
        pub header_bar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub tree_view: TemplateChild<gtk::TreeView>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub dropdown_provider: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub update: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PackageManagerWindow {
        const NAME: &'static str = "PackageManagerWindow";
        type Type = super::PackageManagerWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PackageManagerWindow {}
    impl WidgetImpl for PackageManagerWindow {}
    impl WindowImpl for PackageManagerWindow {}
    impl ApplicationWindowImpl for PackageManagerWindow {}
}

glib::wrapper! {
    pub struct PackageManagerWindow(ObjectSubclass<imp::PackageManagerWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PackageManagerWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new(&[("application", application)])
            .expect("Failed to create PackageManagerWindow")
    }
}
