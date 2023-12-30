use adw::{
    gio,
    glib::{self, Cast, IsA},
    prelude::*,
    subclass::prelude::*,
};

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

    fn show_toast(&self, toast: adw::Toast) {
        self.imp().toast_overlay.add_toast(toast);
    }
}

pub trait ShowToastExt {
    fn show_toast(&self, toast: adw::Toast);

    fn show_toast_msg(&self, msg: &str) {
        self.show_toast(adw::Toast::new(msg));
    }
}

impl<T> ShowToastExt for T
where
    T: IsA<gtk::Widget>,
{
    fn show_toast(&self, toast: adw::Toast) {
        let mut widget: gtk::Widget = self.clone().upcast();
        while let Some(parent) = widget.parent() {
            widget = parent;

            if let Some(lib_window) = widget.downcast_ref::<LibWindow>() {
                lib_window.show_toast(toast);
                return;
            }
        }

        panic!("ShowToastExt requires that one widget in our hierarchy be a LibWindow, but none was found");
    }
}

mod imp {
    use adw::{glib, prelude::*, subclass::prelude::*};
    use gtk::CompositeTemplate;

    use crate::{http::Session, login_page::LoginPage};

    #[derive(Default, Debug, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::LibWindow)]
    #[template(file = "src/window.blp")]
    pub struct LibWindow {
        #[template_child]
        pub(super) toast_overlay: TemplateChild<adw::ToastOverlay>,

        #[property(get)]
        soup_session: Session,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LibWindow {
        const NAME: &'static str = "LibWindow";
        type Type = super::LibWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            LoginPage::ensure_type();

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for LibWindow {}

    impl WidgetImpl for LibWindow {}
    impl WindowImpl for LibWindow {}
    impl ApplicationWindowImpl for LibWindow {}
}
