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
    use schema::books::{Book, BorrowReply, BorrowRequest, BorrowedBook, BorrowedByReply};

    use crate::{
        http::{Session, SessionCookie},
        window::ShowToastExt,
    };

    #[derive(Default, Debug, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::UserView)]
    #[template(file = "src/user_view.blp")]
    pub struct UserView {
        #[template_child]
        all_books: TemplateChild<gio::ListStore>,
        #[template_child]
        borrowed_books: TemplateChild<gio::ListStore>,

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

        fn cookie(&self) -> SessionCookie {
            self.session_cookie.borrow().as_ref().cloned().unwrap()
        }

        fn book_for_id(&self, id: i64) -> Book {
            self.all_books
                .clone()
                .into_iter()
                .map(|elem| {
                    elem.unwrap()
                        .downcast::<BoxedAnyObject>()
                        .map(|book| book.borrow::<Book>().to_owned())
                        .unwrap()
                })
                .find(|book| book.book_id == id)
                .unwrap()
        }

        #[template_callback]
        async fn on_show(&self) {
            self.refresh_books().await;
            self.refresh_borrowed_books().await;
        }

        #[template_callback]
        async fn on_refresh_clicked(&self, _: &gtk::Button) {
            self.refresh_books().await;
            self.refresh_borrowed_books().await;
        }

        async fn refresh_books(&self) {
            let books = self.soup_session().get::<Vec<Book>>("/books").await;

            match books {
                Ok(books) => {
                    self.all_books.remove_all();
                    let books = books
                        .into_iter()
                        .map(BoxedAnyObject::new)
                        .collect::<Vec<_>>();
                    self.all_books.extend_from_slice(&books);
                }
                Err(err) => {
                    self.obj().show_toast_msg("oops");
                    g_warning!("biblioteca", "Failed to fetch books: {err}")
                }
            }
        }

        async fn refresh_borrowed_books(&self) {
            let books = self
                .soup_session()
                .post::<BorrowedByReply>("", &format!("/borrowed-by/{}", self.cookie().user_id()))
                .await;

            match books {
                Ok(books) => {
                    self.borrowed_books.remove_all();
                    let books = books
                        .into_iter()
                        .map(BoxedAnyObject::new)
                        .collect::<Vec<_>>();
                    self.borrowed_books.extend_from_slice(&books);
                }
                Err(err) => {
                    self.obj().show_toast_msg("oops");
                    g_warning!("biblioteca", "Failed to fetch books: {err}")
                }
            }
        }

        // --- ALL BOOKS VIEW ---

        #[template_callback]
        fn on_setup_title_all(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            list_item.set_child(Some(&gtk::Label::new(None)));
        }

        #[template_callback]
        fn on_bind_title_all(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(book) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&book.borrow::<Book>().title);
            }
        }

        #[template_callback]
        fn on_setup_author_all(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            list_item.set_child(Some(&gtk::Label::new(None)));
        }

        #[template_callback]
        fn on_bind_author_all(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
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

            let request = BorrowRequest {
                cookie: self.cookie().cookie().clone(),
                book_id: book.book_id,
            };

            let reply = self
                .soup_session()
                .post::<BorrowReply>(request, "/borrow")
                .await;

            match reply {
                Ok(reply) => {
                    if reply.already_borrowed {
                        self.obj()
                            .show_toast_msg("Nu poți împrumuta aceeași carte de mai multe ori");
                    }
                }
                Err(e) => g_warning!("biblioteca", "we got the error: {}", e),
            }

            self.refresh_borrowed_books().await;
        }

        // --- BORROWED BOOKS VIEW ---
        #[template_callback]
        fn on_setup_title_borrowed(
            &self,
            list_item: &gtk::ListItem,
            _: &gtk::SignalListItemFactory,
        ) {
            list_item.set_child(Some(&gtk::Label::new(None)));
        }

        #[template_callback]
        fn on_bind_title_borrowed(
            &self,
            list_item: &gtk::ListItem,
            _: &gtk::SignalListItemFactory,
        ) {
            if let Some(borrowed_book) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                let book = self.book_for_id(borrowed_book.borrow::<BorrowedBook>().book_id);
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&book.title);
            }
        }

        #[template_callback]
        fn on_setup_author_borrowed(
            &self,
            list_item: &gtk::ListItem,
            _: &gtk::SignalListItemFactory,
        ) {
            list_item.set_child(Some(&gtk::Label::new(None)));
        }

        #[template_callback]
        fn on_bind_author_borrowed(
            &self,
            list_item: &gtk::ListItem,
            _: &gtk::SignalListItemFactory,
        ) {
            if let Some(borrowed_book) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                let book = self.book_for_id(borrowed_book.borrow::<BorrowedBook>().book_id);
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&book.author.name);
            }
        }

        #[template_callback]
        fn on_setup_chapters_read(
            &self,
            list_item: &gtk::ListItem,
            _: &gtk::SignalListItemFactory,
        ) {
            let button = gtk::SpinButton::builder()
                .adjustment(&gtk::Adjustment::new(0.0, 0.0, 1000.0, 1.0, 0.0, 0.0))
                .build();
            list_item.set_child(Some(&button));
        }

        #[template_callback]
        fn on_bind_chapters_read(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(borrowed_book) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                let borrowed_book = borrowed_book.borrow::<BorrowedBook>();
                list_item
                    .child()
                    .and_downcast::<gtk::SpinButton>()
                    .unwrap()
                    .set_value(borrowed_book.chapters_read as f64);
            }
        }

        #[template_callback]
        fn on_setup_date(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            list_item.set_child(Some(&gtk::Label::new(None)));
        }

        #[template_callback]
        fn on_bind_date(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(borrowed_book) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                let borrowed_book = borrowed_book.borrow::<BorrowedBook>();
                let return_on = glib::DateTime::from_unix_local(borrowed_book.valid_until).unwrap();
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&return_on.format("%d %B %Y").unwrap());
            }
        }
    }
}
