use adw::{glib, prelude::*, Application};
use window::LibWindow;

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
