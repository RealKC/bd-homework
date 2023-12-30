use adw::glib;

glib::wrapper! {
    pub struct LoginPage(ObjectSubclass<imp::LoginPage>)
        @extends gtk::Widget;
}

mod imp {
    use std::cell::OnceCell;

    use adw::{glib, prelude::*, subclass::prelude::*};
    use gtk::CompositeTemplate;
    use schema::auth::{CreateAccount, Login};

    use crate::{http::Session, window::ShowToastExt as _};

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
        async fn on_login_clicked(&self, _: gtk::Button) {
            let request = Login {
                email: self.login_email.text().to_string(),
                password: self.login_password.text().to_string(),
            };

            let id = self
                .soup_session()
                .post::<i64>(request, "/auth/login")
                .await;

            match id {
                Ok(id) => println!("our id is: {id}"),
                Err(_) => self.obj().show_toast_msg("Autentificare eșuată"),
            }
        }

        #[template_callback]
        async fn on_signup_clicked(&self, _: gtk::Button) {
            let request = CreateAccount {
                name: self.signup_name.text().to_string(),
                email: self.signup_email.text().to_string(),
                password: self.signup_password.text().to_string(),
            };

            let id = self
                .soup_session()
                .post::<i64>(request, "/auth/create-account")
                .await;

            match id {
                Ok(id) => println!("our id is: {id}"),
                Err(_) => self.obj().show_toast_msg("Nu s-a putut creea contul"),
            }
        }
    }
}
