use gtk::glib::{self, GString};

pub fn now() -> glib::DateTime {
    glib::DateTime::now(&glib::TimeZone::local()).unwrap()
}

#[track_caller]
pub fn date_from(unix: i64) -> glib::DateTime {
    glib::DateTime::from_unix_local(unix).unwrap()
}

pub fn format_date(date: &glib::DateTime) -> GString {
    date.format("%d %B %Y").unwrap()
}
