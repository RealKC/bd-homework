use adw::glib;

glib::wrapper! {
    pub struct LibrarianView(ObjectSubclass<imp::LibrarianView>)
    @extends gtk::Widget;
}

mod imp {
    use std::cell::{OnceCell, RefCell};

    use adw::{glib, prelude::*, subclass::prelude::*};
    use gtk::{
        gio,
        glib::{g_warning, BoxedAnyObject},
        CompositeTemplate,
    };
    use schema::{
        auth::{
            DeleteUserReply, DeleteUserRequest, GetAllUsersReply, GetAllUsersRequest,
            PromoteUserRequest, User,
        },
        books::Book,
        LIBRARIAN, NORMAL_USER,
    };

    use crate::{
        confirmation_dialog::ConfirmationDialogBuilder,
        http::{Session, SessionCookie},
        widget_ext::WidgetUtilsExt,
        window::ShowToastExt,
    };

    #[derive(Default, Debug, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::LibrarianView)]
    #[template(file = "src/librarian_view.blp")]
    pub struct LibrarianView {
        #[template_child]
        view_stack: TemplateChild<adw::ViewStack>,
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
        async fn on_show(&self) {
            self.refresh_books().await;
            self.refresh_users().await;
        }

        #[template_callback]
        async fn on_refresh_clicked(&self, _: &gtk::Button) {
            let Some(current_view) = self.view_stack.visible_child_name() else {
                return;
            };

            if current_view == "all-books" {
                self.refresh_books().await;
            } else if current_view == "borrows" {
                // TODO:
            } else if current_view == "users" {
                self.refresh_users().await;
            }
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

        async fn refresh_users(&self) {
            let request = GetAllUsersRequest {
                cookie: self.cookie().cookie().clone(),
            };
            let users = self
                .soup_session()
                .post::<GetAllUsersReply>(request, "/auth/all-users")
                .await;

            match users {
                Ok(users) => {
                    self.users.remove_all();
                    let users = users
                        .into_iter()
                        .map(BoxedAnyObject::new)
                        .collect::<Vec<_>>();
                    self.users.extend_from_slice(&users);
                }
                Err(err) => {
                    self.obj().show_toast_msg(
                        "A apărut o eroare în timpul obținerii listei de utilizatori",
                    );
                    g_warning!("biblioteca", "Failed to fetch users: {err}")
                }
            }
        }

        #[template_callback]
        fn on_setup_label(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            list_item.set_child(Some(&gtk::Label::new(None)));
        }

        // --- BOOKS VIEW ---

        #[template_callback]
        fn on_bind_title(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(object) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_text(&object.borrow::<Book>().title);
            }
        }

        #[template_callback]
        fn on_bind_author(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(object) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_text(&object.borrow::<Book>().author.name);
            }
        }

        #[template_callback]
        fn on_bind_publisher(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(object) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_text(&object.borrow::<Book>().publisher);
            }
        }

        #[template_callback]
        fn on_edit_book_clicked(_: &gtk::Button, list_item: &gtk::ListItem) {
            if let Some(object) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                println!("TODO: Book edit dialog: {:?}", object);
            }
        }

        #[template_callback]
        fn on_delete_book_clicked(_: &gtk::Button, list_item: &gtk::ListItem) {
            if let Some(object) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                println!("TODO: Delete book dialog: {:?}", object);
            }
        }

        // --- USERS VIEW ---
        #[template_callback]
        fn on_bind_borrow_book_title(
            &self,
            list_item: &gtk::ListItem,
            _: &gtk::SignalListItemFactory,
        ) {
            if let Some(object) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {}
        }

        #[template_callback]
        fn on_bind_borrow_borrower_name(
            &self,
            list_item: &gtk::ListItem,
            _: &gtk::SignalListItemFactory,
        ) {
            if let Some(object) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {}
        }

        #[template_callback]
        fn on_lenghten_borrown_clicked(_: &gtk::Button, list_item: &gtk::ListItem) {}

        #[template_callback]
        fn on_finish_borrow_clicked(_: &gtk::Button, list_item: &gtk::ListItem) {}

        // --- USERS VIEW ---

        #[template_callback]
        fn on_bind_user_name(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(object) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                let user = object.borrow::<User>();

                let label = if user.id == self.cookie().cookie().id {
                    format!("{} (eu)", user.name)
                } else {
                    user.name.clone()
                };

                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&label);
            }
        }

        #[template_callback]
        fn on_bind_user_email(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(object) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&object.borrow::<User>().email);
            }
        }

        #[template_callback]
        fn on_bind_borrowed_book_count(
            &self,
            list_item: &gtk::ListItem,
            _: &gtk::SignalListItemFactory,
        ) {
            if let Some(object) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&object.borrow::<User>().borrowed_book_count.to_string());
            }
        }

        #[template_callback]
        fn on_bind_user_type(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            if let Some(object) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                let user_type = object.borrow::<User>().kind;
                let user_type = if user_type == NORMAL_USER {
                    "Utilizator normal".to_string()
                } else if user_type == LIBRARIAN {
                    "Bibliotecar".to_string()
                } else {
                    format!("Invalid ({user_type})")
                };

                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&user_type);
            }
        }

        #[template_callback(function)]
        fn show_promote_button(object: Option<BoxedAnyObject>) -> bool {
            object
                .map(|obj| obj.borrow::<User>().kind)
                .map(|kind| kind == NORMAL_USER)
                .unwrap_or(false)
        }

        #[template_callback]
        fn on_promote_clicked(button: gtk::Button, list_item: gtk::ListItem) {
            let dialog = ConfirmationDialogBuilder::default()
                .title("Confirmare promovare utilizator")
                .heading("Ești sigur că vrei să promovezi acest utilizator la bibliotecar?")
                .body("Acest utilizator va avea aceleași drepturi ca dvs.")
                .confirm_text("Promovează")
                .action_is_destructive(true)
                .on_confirmation(move || {
                    let librarian_view = button.parent_of_type::<super::LibrarianView>();
                    let user_id = list_item
                        .item()
                        .and_downcast::<BoxedAnyObject>()
                        .map(|obj| obj.borrow::<User>().id)
                        .unwrap();

                    async move {
                        librarian_view
                            .unwrap()
                            .imp()
                            .promote_user_account(user_id)
                            .await;
                    }
                })
                .build();

            dialog.present();
        }

        async fn promote_user_account(&self, user_id: i64) {
            let request = PromoteUserRequest {
                user_to_be_promoted: user_id,
                cookie: self.cookie().cookie().clone(),
            };
            let reply = self
                .soup_session()
                .post::<()>(request, "/auth/promote-user")
                .await;
            if let Err(reply) = reply {
                self.obj()
                    .show_toast_msg("Nu s-a putut realiza promovarea utilizatorului");
                g_warning!("biblioteca", "Failed to promote user: {}", reply);
                return;
            }

            self.refresh_users().await;
        }

        #[template_callback]
        fn on_delete_user_clicked(button: gtk::Button, list_item: gtk::ListItem) {
            let dialog = ConfirmationDialogBuilder::default()
                .title("Confirmare ștergere cont")
                .heading("Ești sigur că vrei să ștergi utilizatorul?")
                .body("Această acțiune este ireversibilă și, în cocordanță cu RGPD, toate datele utilizatorului vor fi șterse.")
                .confirm_text("Șterge contul")
                .on_confirmation(move || {
                    let librarian_view = button.parent_of_type::<super::LibrarianView>();
                    let user_id = list_item
                        .item()
                        .and_downcast::<BoxedAnyObject>()
                        .map(|obj| obj.borrow::<User>().id)
                        .unwrap();

                    async move {
                        librarian_view.unwrap().imp().delete_user_account(user_id).await;
                    }
                })
                .build();

            dialog.present();
        }

        async fn delete_user_account(&self, user_id: i64) {
            let request = DeleteUserRequest {
                user_to_be_deleted: user_id,
                cookie: self.cookie().cookie().clone(),
            };
            let reply = self
                .soup_session()
                .post::<DeleteUserReply>(request, "/auth/delete-user")
                .await;
            match reply {
                Ok(reply) => match reply {
                    DeleteUserReply::Ok => self.refresh_users().await,
                    DeleteUserReply::UsersStillHadBooks => self.obj().show_toast_msg("Nu s-a putut realiza ștergerea contului deoarece utilizatorul încă are cărți împrumutate"),
                    DeleteUserReply::CannotDeleteSelf => self.obj().show_toast_msg("Contul propriu nu poate fi șters în acest fel")
                },
                Err(err) => {
                    self.obj().show_toast_msg("Nu s-a putut realiza ștergerea contului");
                    g_warning!("biblioteca", "Failed to delete account: {err}")
                },
            }
        }
    }
}
