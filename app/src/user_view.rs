use adw::glib;

glib::wrapper! {
    pub struct UserView(ObjectSubclass<imp::UserView>)
    @extends gtk::Widget;
}

mod imp {
    use std::cell::RefCell;

    use adw::{glib, prelude::*, subclass::prelude::*};
    use gtk::CompositeTemplate;

    use crate::http::SessionCookie;

    #[derive(Default, Debug, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::UserView)]
    #[template(file = "src/user_view.blp")]
    pub struct UserView {
        #[property(get, set)]
        session_cookie: RefCell<Option<SessionCookie>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UserView {
        const NAME: &'static str = "LibUserView";
        type Type = super::UserView;
        type ParentType = gtk::Widget;
    }

    #[glib::derived_properties]
    impl ObjectImpl for UserView {}
    impl WidgetImpl for UserView {}
}
