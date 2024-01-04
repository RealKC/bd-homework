use adw::glib;
use gtk::glib::subclass::types::ObjectSubclassIsExt;

glib::wrapper! {
    pub struct LibrarianView(ObjectSubclass<imp::LibrarianView>)
    @extends gtk::Widget;
}

impl LibrarianView {
    pub async fn refresh_books(&self) {
        self.imp().refresh_books().await;
    }
}

mod imp {
    use std::{
        cell::{OnceCell, RefCell},
        cmp::Ordering,
    };

    use adw::{glib, prelude::*, subclass::prelude::*};
    use gtk::{
        gio,
        glib::{g_warning, BoxedAnyObject, GString},
        CompositeTemplate,
    };
    use schema::{
        auth::{
            DeleteUserReply, DeleteUserRequest, GetAllUsersReply, GetAllUsersRequest,
            PromoteUserRequest, User,
        },
        books::{Book, Borrow, BorrowsReply, BorrowsRequest},
        LIBRARIAN, NORMAL_USER,
    };
    use soup::Status;

    use crate::{
        confirmation_dialog::ConfirmationDialogBuilder,
        edit_author_details::EditAuthorDetailsWindow,
        edit_book_details::EditBookDetailsWindow,
        http::{Error, Session, SessionCookie},
        time,
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
            self.refresh_borrows().await;
        }

        #[template_callback]
        async fn on_refresh_clicked(&self, _: &gtk::Button) {
            let Some(current_view) = self.view_stack.visible_child_name() else {
                return;
            };

            if current_view == "all-books" {
                self.refresh_books().await;
            } else if current_view == "borrows" {
                self.refresh_books().await;
                self.refresh_users().await;
                self.refresh_borrows().await;
            } else if current_view == "users" {
                self.refresh_users().await;
            }
        }

        fn book_with(&self, book_id: i64) -> Book {
            self.all_books
                .into_iter()
                .map(|obj| obj.unwrap().downcast::<BoxedAnyObject>().unwrap())
                .map(|obj| obj.borrow::<Book>().clone())
                .find(|book| book.book_id == book_id)
                .unwrap()
        }

        fn user_with(&self, user_id: i64) -> User {
            self.users
                .into_iter()
                .map(|obj| obj.unwrap().downcast::<BoxedAnyObject>().unwrap())
                .map(|obj| obj.borrow::<User>().clone())
                .find(|user| user.id == user_id)
                .unwrap()
        }

        pub(super) async fn refresh_books(&self) {
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

        async fn refresh_borrows(&self) {
            let request = BorrowsRequest {
                cookie: self.cookie().cookie().clone(),
            };
            let borrows = self
                .soup_session()
                .post::<BorrowsReply>(request, "/borrows")
                .await;

            match borrows {
                Ok(borrows) => {
                    self.borrows.remove_all();
                    let borrows = borrows
                        .into_iter()
                        .map(BoxedAnyObject::new)
                        .collect::<Vec<_>>();
                    self.borrows.extend_from_slice(&borrows);
                }
                Err(err) => {
                    self.obj().show_toast_msg(
                        "A apărut o eroare în timpul obținerii listei de împrumuturi",
                    );
                    g_warning!("biblioteca", "Failed to fetch borrows: {err}")
                }
            }
        }

        #[template_callback]
        fn on_setup_label(&self, list_item: &gtk::ListItem, _: &gtk::SignalListItemFactory) {
            list_item.set_child(Some(&gtk::Label::new(None)));
        }

        // --- BOOKS VIEW ---

        #[template_callback(function)]
        fn show_new_book_button(visible_child: GString) -> bool {
            visible_child == "all-books"
        }

        #[template_callback]
        fn on_new_book_clicked(&self, _: gtk::Button) {
            EditBookDetailsWindow::new(None, self.obj().clone()).present();
        }

        #[template_callback]
        fn on_new_author_clicked(&self, _: gtk::Button) {
            EditAuthorDetailsWindow::new(self.obj().clone()).present();
        }

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
        fn on_edit_book_clicked(button: gtk::Button, list_item: gtk::ListItem) {
            EditBookDetailsWindow::new(
                list_item
                    .item()
                    .and_downcast::<BoxedAnyObject>()
                    .map(|obj| obj.borrow::<Book>().clone()),
                button.parent_of_type::<super::LibrarianView>().unwrap(),
            )
            .present();
        }

        #[template_callback]
        fn on_delete_book_clicked(button: gtk::Button, list_item: gtk::ListItem) {
            let dialog = ConfirmationDialogBuilder::default()
                .title("Ești sigur?")
                .heading("Ești sigur că vrei să ștergi această carte?")
                .body("Această acțiune este ireversibilă")
                .confirm_text("Șterge cartea")
                .action_is_destructive(true)
                .on_confirmation(move || {
                    let this = button.parent_of_type::<super::LibrarianView>().unwrap();
                    let borrow_id = list_item
                        .item()
                        .and_downcast::<BoxedAnyObject>()
                        .map(|obj| obj.borrow::<Book>().book_id)
                        .unwrap();

                    async move {
                        this.imp().delete_book(borrow_id).await;
                    }
                })
                .build();

            dialog.present();
        }

        async fn delete_book(&self, book_id: i64) {
            if let Err(err) = self
                .soup_session()
                .post::<()>(self.cookie().cookie(), &format!("/delete-book/{book_id}"))
                .await
            {
                let extras = if let Error::Api { status, .. } = err {
                    if status == Status::BadRequest {
                        "(cartea este în împrumut)"
                    } else {
                        ""
                    }
                } else {
                    ""
                };

                self.obj()
                    .show_toast_msg(&format!("Ștergerea cărții a eșuat {extras}"));
            } else {
                self.refresh_books().await;
            }
        }

        // --- BORROWS VIEW ---
        #[template_callback]
        fn on_bind_borrow_book_title(
            &self,
            list_item: &gtk::ListItem,
            _: &gtk::SignalListItemFactory,
        ) {
            if let Some(borrow) = list_item.item().and_downcast::<BoxedAnyObject>() {
                let book_id = borrow.borrow::<Borrow>().book_id;

                let book = self.book_with(book_id);
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&book.title);
            }
        }

        #[template_callback]
        fn on_bind_borrow_borrower_name(
            &self,
            list_item: &gtk::ListItem,
            _: &gtk::SignalListItemFactory,
        ) {
            if let Some(borrow) = list_item.item().and_downcast::<BoxedAnyObject>() {
                let user_id = borrow.borrow::<Borrow>().user_id;

                let user = self.user_with(user_id);
                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_label(&user.name);
            }
        }

        #[template_callback]
        fn on_bind_borrow_valid_until(
            &self,
            list_item: &gtk::ListItem,
            _: &gtk::SignalListItemFactory,
        ) {
            if let Some(borrow) = list_item.item().and_downcast::<BoxedAnyObject>() {
                let borrow = borrow.borrow::<Borrow>();
                let valid_until = glib::DateTime::from_unix_local(borrow.valid_until).unwrap();
                let now = time::now();
                let remaining_days = valid_until.difference(&now).as_days();

                let colour = if remaining_days <= 3 {
                    r#"color="red""#
                } else {
                    ""
                };

                let valid_until = time::format_date(&valid_until);
                let label = match remaining_days.cmp(&0) {
                    Ordering::Greater => format!(
                        "{valid_until} (<span {colour}>{remaining_days} zile rămase</span>)",
                    ),
                    Ordering::Equal => {
                        format!(r#"{valid_until} (<span color="red" weight="bold">astăzi</span>)"#)
                    }
                    Ordering::Less => format!(
                        r#"{valid_until} (<span weight="bold">întarziere de {} zile</span>)"#,
                        -remaining_days
                    ),
                };

                list_item
                    .child()
                    .and_downcast::<gtk::Label>()
                    .unwrap()
                    .set_markup(&label);
            }
        }

        #[template_callback]
        fn on_lengthen_borrow_clicked(button: gtk::Button, list_item: gtk::ListItem) {
            let Some(borrow) = list_item.item().and_downcast::<BoxedAnyObject>() else {
                return;
            };
            let borrow = borrow.borrow::<Borrow>().clone();
            let valid_until = glib::DateTime::from_unix_local(borrow.valid_until)
                .unwrap()
                .add_days(30)
                .unwrap();
            let now = time::now();
            let remaining_days = valid_until.difference(&now).as_days();

            let dialog = ConfirmationDialogBuilder::default()
                .title("Ești sigur?")
                .heading("Ești sigur că vrei să lungești durata acestui împrumut?")
                .body(format!("Noua data la care va trebui înapoiată cartea este {} (în {remaining_days} de zile)", time::format_date(&valid_until)))
                .confirm_text("Da")
                .on_confirmation(move || {
                    let this = button.parent_of_type::<super::LibrarianView>().unwrap();
                    let borrow_id = borrow.borrow_id;

                    async move {
                        this.imp().lengthen_borrow(borrow_id).await;
                    }
                })
                .build();

            dialog.present();
        }

        async fn lengthen_borrow(&self, borrow_id: i64) {
            let endpoint = format!("/lengthen-borrow/{borrow_id}?days=30");
            if let Err(err) = self
                .soup_session()
                .post::<()>(self.cookie().cookie(), &endpoint)
                .await
            {
                self.obj()
                    .show_toast_msg("Prelungirea duratei împrumutului a eșuat");
                g_warning!("biblioteca", "Error on POST to {}: {}", endpoint, err);
            } else {
                self.refresh_borrows().await;
            }
        }

        #[template_callback]
        fn on_finish_borrow_clicked(button: gtk::Button, list_item: gtk::ListItem) {
            let dialog = ConfirmationDialogBuilder::default()
                .title("Ești sigur?")
                .heading("Ești sigur că vrei să închei împrumutul utilizatorului acum?")
                .body("Acest lucru îl va forța să înapoieze cartea astăzi")
                .confirm_text("Încheie împrumut")
                .action_is_destructive(true)
                .on_confirmation(move || {
                    let this = button.parent_of_type::<super::LibrarianView>().unwrap();
                    let borrow_id = list_item
                        .item()
                        .and_downcast::<BoxedAnyObject>()
                        .map(|obj| obj.borrow::<Borrow>().borrow_id)
                        .unwrap();

                    async move {
                        this.imp().finish_borrow(borrow_id).await;
                    }
                })
                .build();

            dialog.present();
        }

        async fn finish_borrow(&self, borrow_id: i64) {
            let endpoint = format!("/end-borrow/{borrow_id}");
            if let Err(err) = self
                .soup_session()
                .post::<()>(self.cookie().cookie(), &endpoint)
                .await
            {
                self.obj().show_toast_msg("Terminarea împrumutului a eșuat");
                g_warning!("biblioteca", "Error on POST to {}: {}", endpoint, err);
            } else {
                self.refresh_borrows().await;
            }
        }

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
