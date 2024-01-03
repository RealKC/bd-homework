use adw::glib;
use schema::books::Book;

use crate::{time, user_view::UserView};

glib::wrapper! {
    pub struct BookDetailsWindow(ObjectSubclass<imp::BookDetailsWindow>)
    @extends gtk::Widget, gtk::Window, adw::Window;
}

impl BookDetailsWindow {
    pub fn new(book: &Book, user_view: UserView) -> Self {
        glib::Object::builder()
            .property("book-id", book.book_id)
            .property("title", &book.title)
            .property("author-name", &book.author.name)
            .property("author-date-of-birth", book.author.date_of_birth)
            .property(
                "author-date-of-death",
                book.author.date_of_death.map(glib::BoxedAnyObject::new),
            )
            .property("author-description", &book.author.description)
            .property(
                "publish-date",
                time::format_date(&time::date_from(book.publish_date)),
            )
            .property("publisher", &book.publisher)
            .property("count", &book.count.to_string())
            .property("can-be-borrowed", book.can_be_borrowed)
            .property("user-view", user_view)
            .build()
    }
}

mod imp {
    use std::{
        cell::{Cell, RefCell},
        marker::PhantomData,
    };

    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{
        glib::{self, g_warning, gformat, GString, WeakRef},
        CompositeTemplate,
    };

    use crate::{time, user_view::UserView};

    #[derive(Default, Debug, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::BookDetailsWindow)]
    #[template(file = "src/book_details.blp")]
    pub struct BookDetailsWindow {
        #[property(get, set)]
        book_id: RefCell<i64>,
        #[property(get, set)]
        title: RefCell<GString>,
        #[property(get, set)]
        author_name: RefCell<GString>,
        #[property(get, set)]
        author_date_of_birth: RefCell<i64>,
        #[property(get = Self::format_date_of_birth)]
        author_date_of_birth_string: PhantomData<GString>,
        #[property(get, set = Self::set_author_date_of_death)]
        author_date_of_death: RefCell<Option<glib::Object>>,
        #[property(get = Self::format_date_of_death)]
        author_date_of_death_string: PhantomData<GString>,
        #[property(get = Self::is_author_dead)]
        is_author_dead: PhantomData<bool>,
        #[property(get, set)]
        author_description: RefCell<GString>,
        #[property(get, set)]
        publish_date: RefCell<GString>,
        #[property(get, set)]
        publisher: RefCell<GString>,
        #[property(get, set)]
        count: RefCell<GString>,
        #[property(get, set)]
        can_be_borrowed: Cell<bool>,
        #[property(get, set)]
        user_view: WeakRef<UserView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BookDetailsWindow {
        const NAME: &'static str = "LibBookDetailsWindow";
        type Type = super::BookDetailsWindow;
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
    impl ObjectImpl for BookDetailsWindow {}
    impl WidgetImpl for BookDetailsWindow {}
    impl WindowImpl for BookDetailsWindow {}
    impl AdwWindowImpl for BookDetailsWindow {}

    #[gtk::template_callbacks]
    impl BookDetailsWindow {
        fn format_date_of_birth(&self) -> GString {
            let birth = time::format_date(&time::date_from(self.obj().author_date_of_birth()));

            gformat!("Data nașterii: {birth}")
        }

        fn set_author_date_of_death(&self, s: Option<glib::Object>) {
            *self.author_date_of_death.borrow_mut() = s;
            self.obj().notify_author_date_of_death();
            self.obj().notify_author_date_of_death_string();
            self.obj().notify_is_author_dead();
        }

        fn format_date_of_death(&self) -> GString {
            let birth = time::date_from(self.obj().author_date_of_birth());
            let Some(death) = self
                .obj()
                .author_date_of_death()
                .and_downcast::<glib::BoxedAnyObject>()
                .map(|obj| *obj.borrow::<i64>())
            else {
                return GString::default();
            };
            let death = time::date_from(death);
            let years = death.difference(&birth).as_days() / 365;

            gformat!("Data morții: {} ({years} ani)", time::format_date(&death))
        }

        fn is_author_dead(&self) -> bool {
            self.author_date_of_death.borrow().is_some()
        }

        #[template_callback(function)]
        fn concat_strs(#[rest] values: &[glib::Value]) -> String {
            let mut res = String::default();
            for (index, value) in values.iter().enumerate() {
                res.push_str(value.get::<&str>().unwrap_or_else(|e| {
                    panic!("Expected string value for argument {index}: {e}");
                }));
            }
            res
        }

        #[template_callback]
        async fn on_borrow_clicked(&self) {
            let Some(user_view) = self.user_view.upgrade() else {
                g_warning!("biblioteca", "Failed to upgrade user_view, parent closed?");
                return;
            };

            self.obj().close();
            user_view.borrow_book(self.obj().book_id()).await;
        }
    }
}
