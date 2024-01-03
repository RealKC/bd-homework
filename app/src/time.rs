use gtk::glib::{self, GString};

pub fn now() -> glib::DateTime {
    glib::DateTime::now(&glib::TimeZone::local()).unwrap()
}

pub fn format_date(date: &glib::DateTime) -> GString {
    date.format("%d %B %Y").unwrap()
}
