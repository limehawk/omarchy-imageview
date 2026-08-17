use gtk4::prelude::*;
use gtk4::{self};
use std::cell::RefCell;
use std::rc::Rc;

use crate::state::app_state::AppState;

pub struct Toolbar {
    pub container: gtk4::Box,
    path_label: gtk4::Label,
    info_label: gtk4::Label,
    sort_btn: gtk4::Button,
    actions_box: gtk4::Box,
    state: Rc<RefCell<AppState>>,
    on_action: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
    window: RefCell<Option<gtk4::ApplicationWindow>>,
}

impl Toolbar {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        container.add_css_class("toolbar");

        let path_label = gtk4::Label::new(None);
        path_label.set_xalign(0.0);
        path_label.add_css_class("accent-text");
        path_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        path_label.set_hexpand(true);
        container.append(&path_label);

        let info_label = gtk4::Label::new(None);
        info_label.add_css_class("status-text");
        container.append(&info_label);

        let on_action: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        let sort_btn = gtk4::Button::with_label("Date");
        sort_btn.add_css_class("flat");
        sort_btn.set_tooltip_text(Some("Sorted by date — click for name"));
        {
            let cb = on_action.clone();
            sort_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f("sort");
                }
            });
        }
        container.append(&sort_btn);

        let actions_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        for (name, icon, tooltip) in [
            ("rotate", "object-rotate-right-symbolic", "Rotate (Ctrl+R)"),
            ("save", "media-floppy-symbolic", "Save (Ctrl+S)"),
            ("copy", "edit-copy-symbolic", "Copy (Ctrl+C)"),
            ("trash", "user-trash-symbolic", "Trash (Delete)"),
            ("info", "dialog-information-symbolic", "Info (Ctrl+I)"),
        ] {
            let btn = gtk4::Button::from_icon_name(icon);
            btn.set_tooltip_text(Some(tooltip));
            btn.add_css_class("flat");
            let action_name = name.to_string();
            let cb = on_action.clone();
            btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f(&action_name);
                }
            });
            actions_box.append(&btn);
        }
        container.append(&actions_box);

        Self {
            container,
            path_label,
            info_label,
            sort_btn,
            actions_box,
            state,
            on_action,
            window: RefCell::new(None),
        }
    }

    pub fn set_on_action<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_action.borrow_mut() = Some(Box::new(f));
    }

    fn sync_sort_button(&self, state: &AppState) {
        self.sort_btn.set_label(state.sort.label());
        self.sort_btn.set_tooltip_text(Some(state.sort.tooltip()));
    }

    /// Wire the OS window so the toolbar can keep its title in sync with the
    /// current view (filename in single, folder in grid).
    pub fn set_window(&self, window: gtk4::ApplicationWindow) {
        *self.window.borrow_mut() = Some(window);
    }

    fn set_window_title(&self, suffix: Option<&str>, dirty: bool) {
        let Some(w) = self.window.borrow().as_ref().cloned() else { return };
        match suffix {
            Some(s) if dirty => w.set_title(Some(&format!("{s} * — Image Viewer"))),
            Some(s) => w.set_title(Some(&format!("{s} — Image Viewer"))),
            None => w.set_title(Some("Image Viewer")),
        }
    }

    pub fn update_grid_mode(&self) {
        let state = self.state.borrow();
        let folder = state
            .current_folder
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        self.path_label.set_text(&folder);
        self.info_label
            .set_text(&format!("{} images", state.files.len()));
        self.sync_sort_button(&state);
        self.actions_box.set_visible(false);

        let folder_name = state
            .current_folder
            .as_ref()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()));
        drop(state);
        self.set_window_title(folder_name.as_deref(), false);
    }

    pub fn update_single_mode(&self) {
        let state = self.state.borrow();
        let filename = state.current_file().and_then(|p| {
            p.file_name().map(|n| n.to_string_lossy().into_owned())
        });
        if let Some(ref name) = filename {
            let label = if state.pending_rotation != 0 {
                format!("{name} *")
            } else {
                name.clone()
            };
            self.path_label.set_text(&label);
        }
        self.info_label
            .set_text(&format!("{} / {}", state.index + 1, state.files.len()));
        self.sync_sort_button(&state);
        self.actions_box.set_visible(true);
        let dirty = state.pending_rotation != 0;
        drop(state);
        self.set_window_title(filename.as_deref(), dirty);
    }
}
