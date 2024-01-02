use adw::glib;

glib::wrapper! {
    pub struct LoginPage(ObjectSubclass<imp::LoginPage>)
        @extends gtk::Widget;
}

mod imp {
    use std::cell::{OnceCell, RefCell};

    use adw::{
        glib::{self, g_warning},
        prelude::*,
        subclass::prelude::*,
    };
    use gtk::CompositeTemplate;
    use schema::auth::{CreateAccount, Login, LoginReply};

    use crate::{
        http::{Session, SessionCookie},
        window::ShowToastExt as _,
    };

    #[derive(Default, Debug, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::LoginPage)]
    #[template(file = "src/login_page.blp")]
    pub struct LoginPage {
        #[template_child]
        login_email: TemplateChild<adw::EntryRow>,
        #[template_child]
        login_password: TemplateChild<adw::PasswordEntryRow>,

        #[template_child]
        signup_name: TemplateChild<adw::EntryRow>,
        #[template_child]
        signup_email: TemplateChild<adw::EntryRow>,
        #[template_child]
        signup_password: TemplateChild<adw::PasswordEntryRow>,

        #[property(get, set)]
        soup_session: OnceCell<Session>,
        #[property(get, set)]
        session_cookie: RefCell<Option<SessionCookie>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LoginPage {
        const NAME: &'static str = "LibLoginPage";
        type Type = super::LoginPage;
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
    impl ObjectImpl for LoginPage {}

    impl WidgetImpl for LoginPage {}

    #[gtk::template_callbacks]
    impl LoginPage {
        fn soup_session(&self) -> &Session {
            self.soup_session.get().unwrap()
        }

        #[template_callback]
        fn on_unmap(&self) {
            self.login_email.set_text("");
            self.login_password.set_text("");
            self.signup_name.set_text("");
            self.signup_email.set_text("");
            self.signup_password.set_text("");
        }

        #[template_callback]
        async fn on_login_clicked(&self, _: gtk::Button) {
            let password = self.login_password.text().to_string();
            let request = Login {
                email: self.login_email.text().to_string(),
                password: password.clone(),
            };

            let reply = self
                .soup_session()
                .post::<LoginReply>(request, "/auth/login")
                .await;

            match reply {
                Ok(reply) => {
                    self.obj()
                        .set_session_cookie(SessionCookie::new(reply.id, password, reply.kind));
                }
                Err(err) => {
                    g_warning!("biblioteca", "login returned an error: {}", err);
                    self.obj().show_toast_msg("Autentificare eșuată");
                }
            }
        }

        #[template_callback]
        async fn on_signup_clicked(&self, _: gtk::Button) {
            let password = self.signup_password.text().to_string();
            let request = CreateAccount {
                name: self.signup_name.text().to_string(),
                email: self.signup_email.text().to_string(),
                password: password.clone(),
            };

            let reply = self
                .soup_session()
                .post::<LoginReply>(request, "/auth/create-account")
                .await;

            match reply {
                Ok(reply) => {
                    self.obj()
                        .set_session_cookie(SessionCookie::new(reply.id, password, reply.kind));
                }
                Err(_) => self.obj().show_toast_msg("Nu s-a putut creea contul"),
            }
        }
    }
}
