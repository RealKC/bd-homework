use adw::glib;
use gtk::glib::BoxedAnyObject;
use schema::books::Book;

use crate::librarian_view::LibrarianView;

glib::wrapper! {
    pub struct EditBookDetailsWindow(ObjectSubclass<imp::EditBookDetailsWindow>)
    @extends gtk::Widget, gtk::Window, adw::Window;
}

impl EditBookDetailsWindow {
    pub fn new(book: Option<Book>, librarian_view: LibrarianView) -> Self {
        glib::Object::builder()
            .property(
                "book-id",
                book.as_ref()
                    .map(|book| book.book_id)
                    .map(BoxedAnyObject::new),
            )
            .property(
                "author-id",
                book.as_ref()
                    .map(|book| book.author.author_id)
                    .map(BoxedAnyObject::new),
            )
            .property(
                "title",
                book.as_ref()
                    .map(|book| book.title.clone())
                    .unwrap_or(imp::DEFAULT_TITLE.into()),
            )
            .property(
                "author-name",
                book.as_ref().map(|book| book.author.name.clone()),
            )
            .property("count", book.as_ref().map(|book| book.count).unwrap_or(0))
            .property(
                "synopsis",
                book.as_ref()
                    .map(|book| book.synopsis.clone())
                    .unwrap_or_default(),
            )
            .property(
                "publisher",
                book.as_ref()
                    .map(|book| book.publisher.clone())
                    .unwrap_or_default(),
            )
            .property(
                "publish-date",
                book.as_ref()
                    .map(|book| book.publish_date)
                    .unwrap_or(i64::MAX),
            )
            .property("librarian-view", librarian_view)
            .build()
    }
}

mod imp {
    use std::cell::RefCell;

    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{
        gio,
        glib::{self, g_warning, BoxedAnyObject, GString, WeakRef},
        CompositeTemplate,
    };
    use schema::books::{Author, ChangeBookDetailsRequest};

    use crate::{librarian_view::LibrarianView, time, window::ShowToastExt};

    pub(super) const DEFAULT_TITLE: &str = "Carte nouă";

    #[derive(Default, Debug, CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::EditBookDetailsWindow)]
    #[template(file = "src/edit_book_details.blp")]
    pub struct EditBookDetailsWindow {
        #[property(get, set, construct_only)]
        book_id: RefCell<Option<glib::Object>>,
        #[property(get, set, construct_only)]
        author_id: RefCell<Option<glib::Object>>,
        #[property(get, set, construct_only)]
        title: RefCell<GString>,
        #[property(get, set, construct_only)]
        count: RefCell<i64>,
        #[property(get, set, construct_only)]
        author_name: RefCell<Option<GString>>,
        #[property(get, set, construct_only)]
        synopsis: RefCell<GString>,
        #[property(get, set, construct_only)]
        librarian_view: WeakRef<LibrarianView>,
        #[property(get, set, construct_only)]
        publisher: RefCell<GString>,
        #[property(get, set, construct_only)]
        publish_date: RefCell<i64>,

        #[template_child]
        title_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        synopsis_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        count_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        authors_dropdown: TemplateChild<adw::ComboRow>,
        #[template_child]
        publisher_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        day_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        month_entry: TemplateChild<adw::ComboRow>,
        #[template_child]
        year_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        authors: TemplateChild<gio::ListStore>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EditBookDetailsWindow {
        const NAME: &'static str = "LibEditBookDetailsWindow";
        type Type = super::EditBookDetailsWindow;
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
    impl ObjectImpl for EditBookDetailsWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            if let (Some(_), Some(author_name)) = (obj.author_id(), obj.author_name()) {
                self.authors.append(&BoxedAnyObject::new(Author {
                    author_id: self.author_id(),
                    name: author_name.to_string(),
                    date_of_birth: 0,
                    date_of_death: None,
                    description: "".into(),
                }));
            }

            if obj.title() != DEFAULT_TITLE {
                self.title_entry.set_text(&obj.title());
            }

            self.count_entry.set_text(&obj.count().to_string());
            self.synopsis_entry.set_text(&obj.synopsis());

            self.publisher_entry.set_text(&obj.publisher());

            if obj.publish_date() != i64::MAX {
                let date = time::date_from(obj.publish_date());
                self.day_entry.set_text(&date.day_of_month().to_string());
                self.month_entry.set_selected(date.month() as u32 - 1);
                self.year_entry.set_text(&date.year().to_string());
            }
        }
    }
    impl WidgetImpl for EditBookDetailsWindow {}
    impl WindowImpl for EditBookDetailsWindow {}
    impl AdwWindowImpl for EditBookDetailsWindow {}

    #[gtk::template_callbacks]
    impl EditBookDetailsWindow {
        fn author_id(&self) -> i64 {
            *self
                .obj()
                .author_id()
                .unwrap()
                .downcast::<BoxedAnyObject>()
                .unwrap()
                .borrow()
        }

        fn book_id(&self) -> Option<i64> {
            Some(
                *self
                    .obj()
                    .book_id()?
                    .downcast::<BoxedAnyObject>()
                    .ok()?
                    .borrow(),
            )
        }

        #[template_callback]
        async fn on_show(&self) {
            self.refresh_authors().await;
        }

        async fn refresh_authors(&self) {
            let Some(librarian_view) = self.librarian_view.upgrade() else {
                g_warning!("biblioteca", "Failed to upgrade librarian_view");
                return;
            };
            let soup = librarian_view.soup_session();

            let authors = soup.get::<Vec<Author>>("/authors").await;

            match authors {
                Ok(authors) => {
                    self.authors.remove_all();

                    let author_id = self
                        .obj()
                        .author_id()
                        .and_downcast::<BoxedAnyObject>()
                        .map(|obj| *obj.borrow::<i64>());

                    let our_author_index = author_id.and_then(|author_id| {
                        authors
                            .iter()
                            .enumerate()
                            .find(|(_, author)| author.author_id == author_id)
                            .map(|(idx, _)| idx)
                    });

                    let authors = authors
                        .into_iter()
                        .map(BoxedAnyObject::new)
                        .collect::<Vec<_>>();
                    self.authors.extend_from_slice(&authors);

                    if let Some(author_idx) = our_author_index {
                        self.authors_dropdown.set_selected(author_idx as u32);
                    }
                }
                Err(err) => {
                    self.obj()
                        .show_toast_msg("Nu s-a putut prelua lista de autori");
                    g_warning!("biblioteca", "Failed to fetch authors: {err}");
                }
            }
        }

        #[template_callback]
        fn author_name(object: Option<gtk::ListItem>) -> String {
            let Some(object) = object.and_then(|obj| obj.item().and_downcast::<BoxedAnyObject>())
            else {
                return "".into();
            };
            let author = object.borrow::<Author>();

            author.name.clone()
        }

        #[template_callback]
        async fn on_save_changes_clicked(&self, button: gtk::Button) {
            let Some(librarian_view) = self.librarian_view.upgrade() else {
                g_warning!("biblioteca", "Failed to upgrade librarian_view");
                return;
            };

            let soup = librarian_view.soup_session();

            let Some(author_id) = self
                .authors
                .item(self.authors_dropdown.selected())
                .and_downcast::<BoxedAnyObject>()
                .map(|obj| obj.borrow::<Author>().author_id)
            else {
                g_warning!("biblioteca", "no author selected?");
                button.show_toast_msg("Trebuie să alegi un autor");
                return;
            };
            let year = match self.year_entry.text().parse() {
                Ok(year) => year,
                Err(err) => {
                    g_warning!(
                        "biblioteca",
                        "Failed to parse {}, err={err}",
                        self.year_entry.text()
                    );
                    button.show_toast_msg("Anul trebuie să fie un număr");
                    return;
                }
            };
            let month = self.month_entry.selected() as i32 + 1;
            let day = match self.day_entry.text().parse() {
                Ok(day) => day,
                Err(err) => {
                    g_warning!(
                        "biblioteca",
                        "Failed to parse {}, err={err}",
                        self.day_entry.text()
                    );
                    button.show_toast_msg("Ziua trebuie să fie un număr");
                    return;
                }
            };
            let publish_date =
                match glib::DateTime::new(&glib::TimeZone::local(), year, month, day, 0, 0, 0.0) {
                    Ok(publish_date) => publish_date,
                    Err(err) => {
                        g_warning!("biblioteca", "Failed to parse date: {err}");
                        button.show_toast_msg(&format!(
                            "Detaliile introduse nu au format o dată validă: {}",
                            err
                        ));
                        return;
                    }
                }
                .to_unix();
            let count = match self.count_entry.text().parse() {
                Ok(count) => count,
                Err(_) => {
                    button.show_toast_msg("Numărul de cărți trebuie să fie un număr întreg");
                    return;
                }
            };
            let request = ChangeBookDetailsRequest {
                book_id: self.book_id(),
                title: self.title_entry.text().into(),
                author_id,
                publish_date,
                publisher: self.publisher_entry.text().into(),
                count,
                synopsis: self.synopsis_entry.text().into(),
                cookie: librarian_view.session_cookie().unwrap().cookie().clone(),
            };
            if let Err(err) = soup.post::<()>(request, "/change-book-details").await {
                g_warning!(
                    "biblioteca",
                    "Failed request to /change-book-details: {}",
                    err
                );
                button.show_toast_msg("Nu au putut fi salvate schimbările");
                return;
            }

            self.obj().close();
            librarian_view.refresh_books().await;
        }
    }
}
