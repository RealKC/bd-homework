use adw::glib;

glib::wrapper! {
    pub struct UserView(ObjectSubclass<imp::UserView>)
    @extends gtk::Widget;
}

mod imp {
    use std::cell::{OnceCell, RefCell};

    use adw::{glib, prelude::*, subclass::prelude::*};
    use gtk::{
        gio,
        glib::{g_warning, BoxedAnyObject, MainContext},
        CompositeTemplate,
    };
    use schema::books::Book;

    use crate::{
        http::{Session, SessionCookie},
        window::ShowToastExt,
    };

    #[derive(Default, Debug, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::UserView)]
    #[template(file = "src/user_view.blp")]
    pub struct UserView {
        #[template_child]
        list_store: TemplateChild<gio::ListStore>,

        #[property(get, set)]
        soup_session: OnceCell<Session>,
        #[property(get, set)]
        session_cookie: RefCell<Option<SessionCookie>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UserView {
        const NAME: &'static str = "LibUserView";
        type Type = super::UserView;
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
    impl ObjectImpl for UserView {}
    impl WidgetImpl for UserView {}

    #[gtk::template_callbacks]
    impl UserView {
        fn soup_session(&self) -> &Session {
            self.soup_session.get().unwrap()
        }

        #[template_callback]
        async fn on_show(&self) {
            self.refresh().await;
        }

        #[template_callback]
        async fn on_refresh_clicked(&self, _: &gtk::Button) {
            self.refresh().await;
        }

        async fn refresh(&self) {
            let books = self.soup_session().get::<Vec<Book>>("/books").await;

            match books {
                Ok(books) => {
                    self.list_store.remove_all();
                    let books = books
                        .into_iter()
                        .map(BoxedAnyObject::new)
                        .collect::<Vec<_>>();
                    self.list_store.extend_from_slice(&books);
                }
                Err(err) => {
                    self.obj().show_toast_msg("oops");
                    g_warning!("biblioteca", "Failed to fetch books: {err}")
                }
            }
        }

        #[template_callback]
        fn on_setup_title(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            list_item.set_child(Some(&gtk::Label::new(None)));
        }

        #[template_callback]
        fn on_bind_title(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(book) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&book.borrow::<Book>().title);
            }
        }

        #[template_callback]
        fn on_setup_author(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            list_item.set_child(Some(&gtk::Label::new(None)));
        }

        #[template_callback]
        fn on_bind_author(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(book) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&book.borrow::<Book>().author.name);
            }
        }

        #[template_callback]
        fn on_setup_description(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            list_item.set_child(Some(&gtk::Label::new(None)));
        }

        #[template_callback]
        fn on_bind_description(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(book) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                let label = list_item.child().and_downcast::<gtk::Label>().unwrap();

                label.set_label(&book.borrow::<Book>().synopsis);
                label.set_width_request(200);
            }
        }

        #[template_callback]
        fn on_setup_borrow(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            list_item.set_child(Some(
                &gtk::Button::builder()
                    .label("Împrumută")
                    .width_request(100)
                    .halign(gtk::Align::Center)
                    .build(),
            ));
        }

        #[template_callback]
        fn on_bind_borrow(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(book) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                let button = list_item.child().and_downcast::<gtk::Button>().unwrap();
                let book = book.borrow::<Book>();

                button.set_sensitive(book.can_be_borrowed);
                button.connect_clicked({
                    let this = self.obj().clone();
                    let book = book.to_owned();
                    move |_| {
                        let this = this.clone();
                        let book = book.clone();
                        MainContext::default().spawn_local(async move {
                            this.imp().borrow_book(book.clone()).await;
                        });
                    }
                });
            }
        }

        #[template_callback]
        fn on_show_book_information(&self, position: u32, _: &gtk::ColumnView) {
            println!("Gonna show info about book @ {position}");
        }

        async fn borrow_book(&self, book: Book) {
            println!("time to borrow: {book:?}");
        }
    }
}
