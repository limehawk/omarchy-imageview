use gtk4::prelude::*;
use gtk4::{self};
use std::cell::RefCell;
use std::rc::Rc;

use crate::state::app_state::AppState;

pub struct Toolbar {
    pub container: gtk4::Box,
    path_label: gtk4::Label,
    info_label: gtk4::Label,
    actions_box: gtk4::Box,
    state: Rc<RefCell<AppState>>,
    on_action: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
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

        let actions_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        let on_action: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        for (name, icon, tooltip) in [
            ("rotate", "object-rotate-right-symbolic", "Rotate (Ctrl+R)"),
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
            actions_box,
            state,
            on_action,
        }
    }

    pub fn set_on_action<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_action.borrow_mut() = Some(Box::new(f));
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
        self.actions_box.set_visible(false);
    }

    pub fn update_single_mode(&self) {
        let state = self.state.borrow();
        if let Some(path) = state.current_file() {
            self.path_label.set_text(
                &path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
            );
        }
        self.info_label
            .set_text(&format!("{} / {}", state.index + 1, state.files.len()));
        self.actions_box.set_visible(true);
    }
}
