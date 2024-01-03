use adw::{
    gio,
    glib::{self, Cast, IsA},
    prelude::*,
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

            if let Some(toast_overlay) = widget.downcast_ref::<adw::ToastOverlay>() {
                toast_overlay.add_toast(toast);
                return;
            }
        }

        panic!("ShowToastExt requires that one widget in our hierarchy be a LibWindow, but none was found");
    }
}

mod imp {
    use std::cell::RefCell;

    use adw::{glib, prelude::*, subclass::prelude::*};
    use gtk::CompositeTemplate;
    use schema::{LIBRARIAN, NORMAL_USER};

    use crate::{
        http::{Session, SessionCookie},
        librarian_view::LibrarianView,
        login_page::LoginPage,
        user_view::UserView,
    };

    #[derive(Default, Debug, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::LibWindow)]
    #[template(file = "src/window.blp")]
    pub struct LibWindow {
        #[template_child]
        pub(super) toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[property(get)]
        soup_session: Session,
        #[property(get, set, nullable)]
        session_cookie: RefCell<Option<SessionCookie>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LibWindow {
        const NAME: &'static str = "LibWindow";
        type Type = super::LibWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            LibrarianView::ensure_type();
            LoginPage::ensure_type();
            UserView::ensure_type();

            klass.bind_template();

            klass.install_action("win.logout", None, |this, _, _| this.imp().logout());
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for LibWindow {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj()
                .connect_session_cookie_notify(|this| this.imp().session_cookie_changed());
        }
    }

    impl LibWindow {
        fn session_cookie_changed(&self) {
            let Some(session_cookie) = self.obj().session_cookie() else {
                return;
            };
            let user_type = session_cookie.user_type();

            if user_type == NORMAL_USER {
                self.stack.set_visible_child_name("user-view");
            } else if user_type == LIBRARIAN {
                self.stack.set_visible_child_name("librarian-view");
            }
        }

        fn logout(&self) {
            self.stack.set_visible_child_name("login");
            self.obj().set_session_cookie(None::<SessionCookie>);
        }
    }

    impl WidgetImpl for LibWindow {}
    impl WindowImpl for LibWindow {}
    impl ApplicationWindowImpl for LibWindow {}
}
