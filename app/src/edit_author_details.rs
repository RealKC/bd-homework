use gtk::glib;

use crate::librarian_view::LibrarianView;

glib::wrapper! {
    pub struct EditAuthorDetailsWindow(ObjectSubclass<imp::EditAuthorDetailsWindow>)
    @extends gtk::Widget, gtk::Window, adw::Window;
}

impl EditAuthorDetailsWindow {
    pub fn new(librarian_view: LibrarianView) -> Self {
        glib::Object::builder()
            .property("librarian-view", librarian_view)
            .build()
    }
}

mod imp {
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{
        glib::{self, g_warning, WeakRef},
        CompositeTemplate,
    };
    use schema::books::ChangeAuthorDetailsRequest;

    use crate::{librarian_view::LibrarianView, time, window::ShowToastExt};

    #[derive(Default, Debug, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::EditAuthorDetailsWindow)]
    #[template(file = "src/edit_author_details.blp")]
    pub struct EditAuthorDetailsWindow {
        #[property(get, set, construct_only)]
        librarian_view: WeakRef<LibrarianView>,

        #[template_child]
        name_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        description_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        birth_day_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        birth_month_entry: TemplateChild<adw::ComboRow>,
        #[template_child]
        birth_year_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        death_day_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        death_month_entry: TemplateChild<adw::ComboRow>,
        #[template_child]
        death_year_entry: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EditAuthorDetailsWindow {
        const NAME: &'static str = "LibEditAuthorDetailsWindow";
        type Type = super::EditAuthorDetailsWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for EditAuthorDetailsWindow {}
    impl WidgetImpl for EditAuthorDetailsWindow {}
    impl WindowImpl for EditAuthorDetailsWindow {}
    impl AdwWindowImpl for EditAuthorDetailsWindow {}

    #[gtk::template_callbacks]
    impl EditAuthorDetailsWindow {
        fn birth_date(&self, widget: &gtk::Widget) -> Result<i64, ()> {
            let date = time::date_from_entries(
                widget,
                &self.birth_year_entry,
                &self.birth_month_entry,
                &self.birth_day_entry,
                "nașterii",
                false,
            )?;

            Ok(date.unwrap().to_unix())
        }

        fn death_date(&self, widget: &gtk::Widget) -> Result<Option<i64>, ()> {
            let date = time::date_from_entries(
                widget,
                &self.death_year_entry,
                &self.death_month_entry,
                &self.death_day_entry,
                "decesului",
                false,
            )?;

            Ok(date.as_ref().map(glib::DateTime::to_unix))
        }

        #[template_callback]
        async fn on_save_changes_clicked(&self, widget: gtk::Button) {
            let Some(librarian_view) = self.librarian_view.upgrade() else {
                g_warning!(
                    "biblioteca",
                    "Failed to upgrade librarian_view in {:?}",
                    Self::type_()
                );
                return;
            };

            let Ok(date_of_birth) = self.birth_date(widget.upcast_ref()) else {
                return;
            };
            let Ok(date_of_death) = self.death_date(widget.upcast_ref()) else {
                return;
            };

            let request = ChangeAuthorDetailsRequest {
                author_id: None,
                name: self.name_entry.text().to_string(),
                date_of_birth,
                date_of_death,
                description: self.description_entry.to_string(),
                cookie: librarian_view.session_cookie().unwrap().cookie().clone(),
            };

            let result = librarian_view
                .soup_session()
                .post::<()>(request, "/change-author-details")
                .await;

            if let Err(err) = result {
                widget.show_toast_msg("Nu s-a putut adăuga autorul");
                g_warning!(
                    "biblioteca",
                    "Failed POST to /change-author-details: {}",
                    err
                );
            } else {
                self.obj().close();
            }
        }
    }
}
