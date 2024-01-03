use adw::{glib, prelude::*, Application};
use window::LibWindow;

mod book_details;
mod confirmation_dialog;
mod http;
mod librarian_view;
mod login_page;
mod time;
mod user_view;
mod widget_ext;
mod window;

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("ro.tuiasi.kc.LibraryClient")
        .build();

    app.connect_activate(|app| {
        let window = LibWindow::new(app);

        window.present();
    });

    app.run()
}
