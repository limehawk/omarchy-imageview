mod core;
mod state;
mod ui;
mod actions;

use gtk4::prelude::*;
use gtk4::gio;
use std::cell::RefCell;
use std::rc::Rc;

use state::app_state::AppState;
use ui::window::ImageViewerWindow;

const APP_ID: &str = "com.omarchy.imageview";

fn main() {
    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    let state = Rc::new(RefCell::new(AppState::new(None)));

    let state_activate = state.clone();
    app.connect_activate(move |app| {
        let viewer = ImageViewerWindow::new(app, state_activate.clone());
        let v = viewer.borrow();

        // Restore last folder
        state_activate.borrow_mut().restore();
        let last = state_activate.borrow().last_folder.clone();

        if let Some(folder) = last {
            if folder.is_dir() {
                state_activate.borrow_mut().load_folder(&folder, None);
                if !state_activate.borrow().files.is_empty() {
                    v.show_grid();
                    v.present();
                    return;
                }
            }
        }

        // No last folder — just show window with grid
        v.show_grid();
        v.present();
    });

    let state_open = state.clone();
    app.connect_open(move |app, files, _hint| {
        let viewer = ImageViewerWindow::new(app, state_open.clone());
        let v = viewer.borrow();

        if let Some(gfile) = files.first() {
            if let Some(path) = gfile.path() {
                if path.is_file() {
                    if let Some(parent) = path.parent() {
                        state_open.borrow_mut().load_folder(parent, Some(&path));
                        v.show_single();
                        v.present();
                        return;
                    }
                } else if path.is_dir() {
                    state_open.borrow_mut().load_folder(&path, None);
                    v.show_grid();
                    v.present();
                    return;
                }
            }
        }

        v.show_grid();
        v.present();
    });

    // Save state on shutdown
    let state_shutdown = state.clone();
    app.connect_shutdown(move |_| {
        let s = state_shutdown.borrow();
        if s.current_folder.is_some() {
            s.save();
        }
    });

    app.run();
}
