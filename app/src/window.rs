use adw::{gio, glib};

glib::wrapper! {
    pub struct LibWindow(ObjectSubclass<imp::LibWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl LibWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }
}

mod imp {
    use adw::{glib, prelude::*, subclass::prelude::*};
    use gtk::CompositeTemplate;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(file = "src/window.blp")]
    pub struct LibWindow {}

    #[glib::object_subclass]
    impl ObjectSubclass for LibWindow {
        const NAME: &'static str = "LibWindow";
        type Type = super::LibWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LibWindow {}

    impl WidgetImpl for LibWindow {}
    impl WindowImpl for LibWindow {}
    impl ApplicationWindowImpl for LibWindow {}
}
