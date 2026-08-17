use gtk4::prelude::*;
use gtk4::gio;
use std::cell::RefCell;
use std::rc::Rc;

use omarchy_imageview::state::app_state::AppState;
use omarchy_imageview::ui::window::ImageViewerWindow;

const APP_ID: &str = "com.omarchy.imageview";

fn main() {
    init_logging();

    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    let state = Rc::new(RefCell::new(AppState::new(None)));
    state.borrow_mut().restore();

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

/// Initialise env_logger (default: info) and route panics through it with a
/// backtrace so crashes show up in the log instead of vanishing.
///
/// Override via `RUST_LOG=debug` (or `omarchy_imageview=debug`).
fn init_logging() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .format_timestamp_millis()
    .init();

    // Force backtraces unless the user has set their own preference.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "<unknown>".to_string());
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic payload>".to_string());
        let backtrace = std::backtrace::Backtrace::force_capture();
        log::error!("PANIC at {location}: {payload}\n{backtrace}");
        default_hook(info);
    }));

    log::info!(
        "omarchy-imageview {} starting",
        env!("CARGO_PKG_VERSION")
    );
}
