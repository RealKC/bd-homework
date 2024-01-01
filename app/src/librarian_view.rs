use adw::glib;

glib::wrapper! {
    pub struct LibrarianView(ObjectSubclass<imp::LibrarianView>)
    @extends gtk::Widget;
}

mod imp {
    use std::cell::{OnceCell, RefCell};

    use adw::{glib, prelude::*, subclass::prelude::*};
    use gtk::{gio, CompositeTemplate};

    use crate::http::{Session, SessionCookie};

    #[derive(Default, Debug, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::LibrarianView)]
    #[template(file = "src/librarian_view.blp")]
    pub struct LibrarianView {
        #[template_child]
        all_books: TemplateChild<gio::ListStore>,
        #[template_child]
        borrows: TemplateChild<gio::ListStore>,
        #[template_child]
        users: TemplateChild<gio::ListStore>,

        #[property(get, set)]
        soup_session: OnceCell<Session>,
        #[property(get, set)]
        session_cookie: RefCell<Option<SessionCookie>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LibrarianView {
        const NAME: &'static str = "LibLibrarianView";
        type Type = super::LibrarianView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for LibrarianView {}
    impl WidgetImpl for LibrarianView {}

    #[gtk::template_callbacks]
    impl LibrarianView {
        fn soup_session(&self) -> &Session {
            self.soup_session.get().unwrap()
        }

        fn cookie(&self) -> SessionCookie {
            self.session_cookie.borrow().as_ref().cloned().unwrap()
        }

        #[template_callback]
        async fn on_show(&self) {}

        #[template_callback]
        async fn on_refresh_clicked(&self) {}
    }
}
