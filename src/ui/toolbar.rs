use gtk4::prelude::*;
use gtk4::{self};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::core::folder::SortMode;
use crate::state::app_state::AppState;

pub struct Toolbar {
    pub container: gtk4::Box,
    path_label: gtk4::Label,
    info_label: gtk4::Label,
    sort_btn: gtk4::MenuButton,
    sort_date: gtk4::CheckButton,
    sort_name: gtk4::CheckButton,
    sort_syncing: Rc<Cell<bool>>,
    actions_box: gtk4::Box,
    state: Rc<RefCell<AppState>>,
    on_action: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
    window: RefCell<Option<gtk4::ApplicationWindow>>,
}

impl Toolbar {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        container.add_css_class("toolbar");

        let copy = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
        copy.add_css_class("toolbar-copy");
        copy.set_hexpand(true);
        copy.set_valign(gtk4::Align::Center);
        copy.set_halign(gtk4::Align::Start);

        let path_label = gtk4::Label::new(None);
        path_label.set_xalign(0.0);
        path_label.add_css_class("toolbar-identity");
        path_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        path_label.set_hexpand(true);
        copy.append(&path_label);

        let info_label = gtk4::Label::new(None);
        info_label.set_xalign(0.0);
        info_label.add_css_class("toolbar-meta");
        copy.append(&info_label);

        container.append(&copy);

        let on_action: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        let tools = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        tools.add_css_class("toolbar-tools");
        tools.set_valign(gtk4::Align::Center);
        tools.set_halign(gtk4::Align::End);

        let sort_btn = gtk4::MenuButton::new();
        sort_btn.set_always_show_arrow(false);
        sort_btn.set_has_frame(false);
        sort_btn.add_css_class("pixel-btn");
        sort_btn.set_valign(gtk4::Align::Center);
        sort_btn.set_child(Some(&super::icons::image(&super::icons::SORT)));
        sort_btn.set_tooltip_text(Some(state.borrow().sort.tooltip()));

        let sort_date = gtk4::CheckButton::with_label("Date");
        let sort_name = gtk4::CheckButton::with_label("Name");
        sort_name.set_group(Some(&sort_date));

        let sort_menu = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        sort_menu.add_css_class("sort-menu");
        sort_menu.append(&sort_date);
        sort_menu.append(&sort_name);

        let popover = gtk4::Popover::new();
        popover.set_child(Some(&sort_menu));
        sort_btn.set_popover(Some(&popover));

        match state.borrow().sort {
            SortMode::Date => sort_date.set_active(true),
            SortMode::Name => sort_name.set_active(true),
        }

        let sort_syncing = Rc::new(Cell::new(false));
        {
            let cb = on_action.clone();
            let pop = popover.clone();
            let syncing = sort_syncing.clone();
            sort_date.connect_toggled(move |btn| {
                if syncing.get() || !btn.is_active() {
                    return;
                }
                if let Some(ref f) = *cb.borrow() {
                    f("sort-date");
                }
                pop.popdown();
            });
        }
        {
            let cb = on_action.clone();
            let pop = popover.clone();
            let syncing = sort_syncing.clone();
            sort_name.connect_toggled(move |btn| {
                if syncing.get() || !btn.is_active() {
                    return;
                }
                if let Some(ref f) = *cb.borrow() {
                    f("sort-name");
                }
                pop.popdown();
            });
        }
        tools.append(&sort_btn);

        let actions_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        for (name, glyph, tooltip) in [
            ("save", &super::icons::SAVE, "Save (Ctrl+S)"),
            ("rotate", &super::icons::ROTATE, "Rotate (Ctrl+R)"),
            ("copy", &super::icons::COPY, "Copy (Ctrl+C)"),
            ("trash", &super::icons::TRASH, "Trash (Delete)"),
            ("filmstrip", &super::icons::FILMSTRIP, "Filmstrip"),
            ("info", &super::icons::INFO, "Info (Ctrl+I)"),
        ] {
            let btn = super::icons::button(name, glyph, tooltip);
            let action_name = name.to_string();
            let cb = on_action.clone();
            btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f(&action_name);
                }
            });
            actions_box.append(&btn);
        }
        tools.append(&actions_box);
        container.append(&tools);

        Self {
            container,
            path_label,
            info_label,
            sort_btn,
            sort_date,
            sort_name,
            sort_syncing,
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
        self.sort_syncing.set(true);
        match state.sort {
            SortMode::Date => self.sort_date.set_active(true),
            SortMode::Name => self.sort_name.set_active(true),
        }
        self.sort_syncing.set(false);
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
        let folder_path = state
            .current_folder
            .as_ref()
            .map(|p| p.display().to_string());
        let folder_name = state
            .current_folder
            .as_ref()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()));
        self.path_label
            .set_text(folder_name.as_deref().unwrap_or("--"));
        self.path_label
            .set_tooltip_text(folder_path.as_deref());
        self.info_label
            .set_text(&format!("{} images", state.files.len()));
        self.sync_sort_button(&state);
        self.actions_box.set_visible(false);

        drop(state);
        self.set_window_title(folder_name.as_deref(), false);
    }

    pub fn update_single_mode(&self) {
        let state = self.state.borrow();
        let filename = state.current_file().and_then(|p| {
            p.file_name().map(|n| n.to_string_lossy().into_owned())
        });
        if let Some(ref name) = filename {
            let label = if state.is_dirty() {
                format!("{name} *")
            } else {
                name.clone()
            };
            self.path_label.set_text(&label);
        } else {
            self.path_label.set_text("--");
        }
        self.path_label.set_tooltip_text(
            state
                .current_file()
                .map(|p| p.display().to_string())
                .as_deref(),
        );
        // Position always. Scale only when it isn't Fit. Slideshow only when running.
        let mut info = format!("{} / {}", state.index + 1, state.files.len());
        if state.scale_mode != crate::state::app_state::ScaleMode::Fit {
            info.push_str("  ·  ");
            info.push_str(state.scale_mode.label());
        }
        if state.slideshow_secs > 0 {
            info.push_str(&format!("  ·  {}s", state.slideshow_secs));
        }
        self.info_label.set_text(&info);
        self.sync_sort_button(&state);
        self.actions_box.set_visible(true);
        let dirty = state.is_dirty();
        drop(state);
        self.set_window_title(filename.as_deref(), dirty);
    }
}
