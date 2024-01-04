use adw::prelude::*;
use gtk::glib::{self, g_warning, GString};

use crate::window::ShowToastExt;

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

pub fn date_from_entries(
    widget: &gtk::Widget,
    year_entry: &adw::EntryRow,
    month_entry: &adw::ComboRow,
    day_entry: &adw::EntryRow,
    context: &str,
    allow_empty: bool,
) -> Result<Option<glib::DateTime>, ()> {
    if allow_empty && year_entry.text().is_empty() && day_entry.text().is_empty() {
        return Ok(None);
    }

    let year = match year_entry.text().parse() {
        Ok(year) => year,
        Err(err) => {
            g_warning!(
                "biblioteca",
                "Failed to parse {}, err={err}",
                year_entry.text()
            );
            widget.show_toast_msg(&format!("Anul {context} trebuie să fie un număr"));
            return Err(());
        }
    };
    let month = month_entry.selected() as i32 + 1;
    let day = match day_entry.text().parse() {
        Ok(day) => day,
        Err(err) => {
            g_warning!(
                "biblioteca",
                "Failed to parse {}, err={err}",
                day_entry.text()
            );
            widget.show_toast_msg(&format!("Ziua {context} trebuie să fie un număr"));
            return Err(());
        }
    };

    match glib::DateTime::new(&glib::TimeZone::local(), year, month, day, 0, 0, 0.0) {
        Ok(publish_date) => Ok(Some(publish_date)),
        Err(err) => {
            g_warning!("biblioteca", "Failed to parse date: {err}");
            widget.show_toast_msg(&format!(
                "Detaliile introduse nu au format o dată validă: {}",
                err
            ));
            Err(())
        }
    }
}
