mod core;
mod state;
mod ui;
mod actions;

use gtk4::prelude::*;
use gtk4::{gio, Application};

const APP_ID: &str = "com.omarchy.imageview";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_activate(|app| {
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("Image Viewer")
            .default_width(1200)
            .default_height(800)
            .build();

        let label = gtk4::Label::new(Some("Rust scaffold — GTK4 working"));
        window.set_child(Some(&label));
        window.present();
    });

    app.run();
}
