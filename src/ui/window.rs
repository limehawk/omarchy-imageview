use gtk4::prelude::*;
use gtk4::{self, gdk, gio, glib};
use std::cell::RefCell;
use std::rc::Rc;

use crate::core::folder::FolderMonitor;
use crate::state::app_state::{AppState, ViewMode};
use super::grid_view::GridView;
use super::info_panel::InfoPanel;
use super::single_view::SingleView;
use super::theme;
use super::toolbar::Toolbar;

pub struct ImageViewerWindow {
    pub window: gtk4::ApplicationWindow,
    pub stack: gtk4::Stack,
    state: Rc<RefCell<AppState>>,
    grid_view: GridView,
    single_view: Rc<RefCell<SingleView>>,
    toolbar: Rc<Toolbar>,
    info_panel: Rc<InfoPanel>,
    _monitor: RefCell<Option<FolderMonitor>>,
}

impl ImageViewerWindow {
    pub fn new(app: &gtk4::Application, state: Rc<RefCell<AppState>>) -> Rc<RefCell<Self>> {
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("Image Viewer")
            .default_width(1200)
            .default_height(800)
            .build();

        // Apply theme
        let css = theme::load_theme();
        let provider = gtk4::CssProvider::new();
        provider.load_from_string(&css);
        gtk4::style_context_add_provider_for_display(
            &gdk::Display::default().unwrap(),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Main layout: vertical box holding toolbar + horizontal content area
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        // Toolbar
        let toolbar = Rc::new(Toolbar::new(state.clone()));
        toolbar.set_window(window.clone());
        main_box.append(&toolbar.container);

        // Horizontal content area: stack on left, info panel on right
        let content_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

        // Stack with views
        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_transition_duration(150);
        stack.set_hexpand(true);
        stack.set_vexpand(true);

        let grid_view = GridView::new(state.clone());
        stack.add_named(&grid_view.container, Some("grid"));

        let single_view = SingleView::new(state.clone());
        stack.add_named(&single_view.borrow().container, Some("single"));

        content_box.append(&stack);

        // Info panel (hidden by default)
        let info_panel = Rc::new(InfoPanel::new());
        info_panel.container.set_visible(false);
        content_box.append(&info_panel.container);

        main_box.append(&content_box);
        window.set_child(Some(&main_box));

        let viewer = Rc::new(RefCell::new(Self {
            window: window.clone(),
            stack: stack.clone(),
            state: state.clone(),
            grid_view,
            single_view: single_view.clone(),
            toolbar: toolbar.clone(),
            info_panel: info_panel.clone(),
            _monitor: RefCell::new(None),
        }));

        // Wire single-view navigation (filmstrip click, scroll wheel) -> refresh toolbar + info panel
        {
            let viewer_nav = viewer.clone();
            single_view.borrow().set_on_navigate(move || {
                let v = viewer_nav.borrow();
                v.toolbar.update_single_mode();
                if v.info_panel.container.is_visible() {
                    let path = v.state.borrow().current_file().map(|p| p.to_path_buf());
                    v.info_panel.update(path.as_deref());
                }
            });
        }

        // Wire grid activate -> switch to single view
        {
            let viewer_activate = viewer.clone();
            let state_activate = state.clone();
            viewer.borrow().grid_view.set_on_activate(move |index| {
                state_activate.borrow_mut().navigate_to(index as usize);
                viewer_activate.borrow().show_single();
            });
        }

        // Wire toolbar action callback
        {
            let viewer_ref = viewer.clone();
            let single_ref2 = single_view.clone();
            let state_ref2 = state.clone();
            toolbar.set_on_action(move |action| {
                match action {
                    "info" => {
                        let v = viewer_ref.borrow();
                        let panel = &v.info_panel;
                        let currently_visible = panel.container.is_visible();
                        panel.container.set_visible(!currently_visible);
                        if !currently_visible {
                            let path = v.state.borrow().current_file().map(|p| p.to_path_buf());
                            panel.update(path.as_deref());
                        }
                    }
                    "trash" => {
                        let path = state_ref2.borrow().current_file().map(|p| p.to_path_buf());
                        if let Some(path) = path {
                            if crate::actions::file_ops::trash_file(&path) {
                                state_ref2.borrow_mut().remove_file(&path);
                                single_ref2.borrow().refresh_image();
                                let v = viewer_ref.borrow();
                                v.toolbar.update_single_mode();
                            }
                        }
                    }
                    "rotate" => {
                        single_ref2.borrow().rotate_displayed(90);
                        viewer_ref.borrow().toolbar.update_single_mode();
                    }
                    "save" => {
                        single_ref2.borrow().save_rotation();
                        viewer_ref.borrow().toolbar.update_single_mode();
                    }
                    "sort" => {
                        let current = state_ref2.borrow().current_file().map(|p| p.to_path_buf());
                        let folder = state_ref2.borrow().current_folder.clone();
                        state_ref2.borrow_mut().cycle_sort();
                        state_ref2.borrow().save();
                        if let Some(folder) = folder {
                            state_ref2.borrow_mut().load_folder(&folder, current.as_deref());
                        }
                        let v = viewer_ref.borrow();
                        match state_ref2.borrow().view_mode {
                            ViewMode::Grid => {
                                v.grid_view.load();
                                v.toolbar.update_grid_mode();
                            }
                            ViewMode::Single => {
                                v.single_view.borrow().load();
                                v.toolbar.update_single_mode();
                            }
                        }
                    }
                    "copy" => {
                        let tex = single_ref2.borrow().current_texture();
                        if let Some(tex) = tex {
                            crate::actions::clipboard::copy_texture_to_clipboard(&tex);
                        }
                    }
                    _ => {}
                }
            });
        }

        // Keybindings
        let controller = gtk4::EventControllerKey::new();
        let viewer_ref = viewer.clone();
        let state_ref = state.clone();
        let window_ref = window.clone();
        let single_ref = single_view.clone();

        controller.connect_key_pressed(move |_ctrl, keyval, _keycode, modifiers| {
            let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
            let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
            let key_name = keyval.name().unwrap_or_default();
            let is_single = state_ref.borrow().view_mode == ViewMode::Single;

            match key_name.as_str() {
                "Escape" => {
                    if is_single {
                        viewer_ref.borrow().show_grid();
                    } else {
                        window_ref.close();
                    }
                    glib::Propagation::Stop
                }
                "q" if !ctrl => {
                    window_ref.close();
                    glib::Propagation::Stop
                }
                "BackSpace" if is_single => {
                    single_ref.borrow().navigate_prev();
                    let v = viewer_ref.borrow();
                    v.toolbar.update_single_mode();
                    if v.info_panel.container.is_visible() {
                        let path = v.state.borrow().current_file().map(|p| p.to_path_buf());
                        v.info_panel.update(path.as_deref());
                    }
                    glib::Propagation::Stop
                }
                "g" if !ctrl => {
                    viewer_ref.borrow().show_grid();
                    glib::Propagation::Stop
                }
                "F11" | "f" if !ctrl => {
                    if window_ref.is_fullscreen() {
                        window_ref.unfullscreen();
                        if is_single {
                            single_ref.borrow().set_filmstrip_visible(true);
                        }
                        let v = viewer_ref.borrow();
                        v.toolbar.container.set_visible(true);
                    } else {
                        window_ref.fullscreen();
                        if is_single {
                            single_ref.borrow().set_filmstrip_visible(false);
                        }
                        let v = viewer_ref.borrow();
                        v.toolbar.container.set_visible(false);
                    }
                    glib::Propagation::Stop
                }
                "i" if ctrl => {
                    let v = viewer_ref.borrow();
                    if is_single {
                        let panel = &v.info_panel;
                        let currently_visible = panel.container.is_visible();
                        panel.container.set_visible(!currently_visible);
                        if !currently_visible {
                            let path = v.state.borrow().current_file().map(|p| p.to_path_buf());
                            panel.update(path.as_deref());
                        }
                    }
                    glib::Propagation::Stop
                }
                // Single view navigation
                "Right" | "Down" if is_single && !ctrl => {
                    single_ref.borrow().navigate_next();
                    let v = viewer_ref.borrow();
                    v.toolbar.update_single_mode();
                    if v.info_panel.container.is_visible() {
                        let path = v.state.borrow().current_file().map(|p| p.to_path_buf());
                        v.info_panel.update(path.as_deref());
                    }
                    glib::Propagation::Stop
                }
                "Left" | "Up" if is_single && !ctrl => {
                    single_ref.borrow().navigate_prev();
                    let v = viewer_ref.borrow();
                    v.toolbar.update_single_mode();
                    if v.info_panel.container.is_visible() {
                        let path = v.state.borrow().current_file().map(|p| p.to_path_buf());
                        v.info_panel.update(path.as_deref());
                    }
                    glib::Propagation::Stop
                }
                "space" if is_single => {
                    single_ref.borrow().navigate_next();
                    let v = viewer_ref.borrow();
                    v.toolbar.update_single_mode();
                    if v.info_panel.container.is_visible() {
                        let path = v.state.borrow().current_file().map(|p| p.to_path_buf());
                        v.info_panel.update(path.as_deref());
                    }
                    glib::Propagation::Stop
                }
                "Home" if is_single => {
                    single_ref.borrow().navigate_to_start();
                    let v = viewer_ref.borrow();
                    v.toolbar.update_single_mode();
                    if v.info_panel.container.is_visible() {
                        let path = v.state.borrow().current_file().map(|p| p.to_path_buf());
                        v.info_panel.update(path.as_deref());
                    }
                    glib::Propagation::Stop
                }
                "End" if is_single => {
                    single_ref.borrow().navigate_to_end();
                    let v = viewer_ref.borrow();
                    v.toolbar.update_single_mode();
                    if v.info_panel.container.is_visible() {
                        let path = v.state.borrow().current_file().map(|p| p.to_path_buf());
                        v.info_panel.update(path.as_deref());
                    }
                    glib::Propagation::Stop
                }
                // Zoom keys (Ctrl+=/+, Ctrl+-, Ctrl+0, Ctrl+1)
                "plus" | "equal" if is_single && ctrl => {
                    single_ref.borrow().zoom_in();
                    glib::Propagation::Stop
                }
                "minus" if is_single && ctrl => {
                    single_ref.borrow().zoom_out();
                    glib::Propagation::Stop
                }
                "0" if is_single => {
                    single_ref.borrow().zoom_to_fit();
                    glib::Propagation::Stop
                }
                "1" if is_single && ctrl => {
                    single_ref.borrow().zoom_to_actual();
                    glib::Propagation::Stop
                }
                // Plain zoom keys (no modifier)
                "plus" | "equal" if is_single && !ctrl => {
                    single_ref.borrow().zoom_in();
                    glib::Propagation::Stop
                }
                "minus" if is_single && !ctrl => {
                    single_ref.borrow().zoom_out();
                    glib::Propagation::Stop
                }
                "1" if is_single && !ctrl => {
                    single_ref.borrow().zoom_to_actual();
                    glib::Propagation::Stop
                }
                // Delete -> trash current file
                "Delete" if is_single => {
                    let path = state_ref.borrow().current_file().map(|p| p.to_path_buf());
                    if let Some(path) = path {
                        if crate::actions::file_ops::trash_file(&path) {
                            state_ref.borrow_mut().remove_file(&path);
                            single_ref.borrow().refresh_image();
                            let v = viewer_ref.borrow();
                            v.toolbar.update_single_mode();
                        }
                    }
                    glib::Propagation::Stop
                }
                // Ctrl+r -> rotate 90, Ctrl+Shift+r -> rotate 270 (display only until save)
                "r" if ctrl && is_single => {
                    let degrees = if shift { 270 } else { 90 };
                    single_ref.borrow().rotate_displayed(degrees);
                    viewer_ref.borrow().toolbar.update_single_mode();
                    glib::Propagation::Stop
                }
                "s" if ctrl && is_single => {
                    single_ref.borrow().save_rotation();
                    viewer_ref.borrow().toolbar.update_single_mode();
                    glib::Propagation::Stop
                }
                // Ctrl+Shift+x -> trash and advance
                "x" if ctrl && shift && is_single => {
                    let path = state_ref.borrow().current_file().map(|p| p.to_path_buf());
                    if let Some(path) = path {
                        if crate::actions::file_ops::trash_file(&path) {
                            state_ref.borrow_mut().remove_file(&path);
                            single_ref.borrow().refresh_image();
                            let v = viewer_ref.borrow();
                            v.toolbar.update_single_mode();
                        }
                    }
                    glib::Propagation::Stop
                }
                // Ctrl+c -> copy texture, Ctrl+Shift+c -> copy file path
                "c" if ctrl && is_single => {
                    if shift {
                        let path = state_ref.borrow().current_file().map(|p| p.to_path_buf());
                        if let Some(path) = path {
                            crate::actions::clipboard::copy_text_to_clipboard(
                                &path.display().to_string(),
                            );
                        }
                    } else {
                        let tex = single_ref.borrow().current_texture();
                        if let Some(tex) = tex {
                            crate::actions::clipboard::copy_texture_to_clipboard(&tex);
                        }
                    }
                    glib::Propagation::Stop
                }
                // Ctrl+w -> set as wallpaper
                "w" if ctrl && is_single => {
                    let path = state_ref.borrow().current_file().map(|p| p.to_path_buf());
                    if let Some(path) = path {
                        crate::actions::wallpaper::set_wallpaper(&path);
                    }
                    glib::Propagation::Stop
                }
                // Ctrl+e -> open in Pinta (not xdg-open — we are the default handler)
                "e" if ctrl && is_single => {
                    let path = state_ref.borrow().current_file().map(|p| p.to_path_buf());
                    if let Some(path) = path {
                        crate::actions::file_ops::open_in_editor(&path);
                    }
                    glib::Propagation::Stop
                }
                // F2 -> rename dialog
                "F2" if is_single => {
                    let v = viewer_ref.borrow();
                    show_rename_dialog(
                        &v.window,
                        &state_ref,
                        &single_ref,
                        &v.toolbar,
                        &v.info_panel,
                    );
                    glib::Propagation::Stop
                }
                // Ctrl+m -> move to folder, Ctrl+Shift+m -> copy to folder
                "m" if ctrl && is_single => {
                    let v = viewer_ref.borrow();
                    if shift {
                        show_folder_dialog(
                            &v.window,
                            &state_ref,
                            &single_ref,
                            &v.toolbar,
                            false, // copy mode
                        );
                    } else {
                        show_folder_dialog(
                            &v.window,
                            &state_ref,
                            &single_ref,
                            &v.toolbar,
                            true, // move mode
                        );
                    }
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        window.add_controller(controller);

        viewer
    }

    pub fn show_grid(&self) {
        self.state.borrow_mut().view_mode = ViewMode::Grid;
        self.grid_view.load();
        self.stack.set_visible_child_name("grid");
        self.toolbar.update_grid_mode();
        // Hide info panel when switching to grid
        self.info_panel.container.set_visible(false);
        // Start filesystem monitoring
        self.start_monitor();
    }

    pub fn show_single(&self) {
        self.state.borrow_mut().view_mode = ViewMode::Single;
        self.stack.set_visible_child_name("single");
        self.single_view.borrow().load();
        self.toolbar.update_single_mode();
        // Start filesystem monitoring
        self.start_monitor();
    }

    fn start_monitor(&self) {
        let folder = self.state.borrow().current_folder.clone();
        if let Some(folder_path) = folder {
            let state = self.state.clone();
            let grid_view = self.grid_view.clone();
            let single_view = self.single_view.clone();
            let toolbar = self.toolbar.clone();
            let monitor = FolderMonitor::new(&folder_path, move || {
                let current = state.borrow().current_file().map(|p| p.to_path_buf());
                if let Some(ref folder) = state.borrow().current_folder.clone() {
                    state.borrow_mut().load_folder(folder, current.as_deref());
                }
                let mode = state.borrow().view_mode;
                match mode {
                    ViewMode::Grid => {
                        grid_view.load();
                        toolbar.update_grid_mode();
                    }
                    ViewMode::Single => {
                        single_view.borrow().refresh_image();
                        toolbar.update_single_mode();
                    }
                }
            });
            *self._monitor.borrow_mut() = monitor;
        }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn show_rename_dialog(
    window: &gtk4::ApplicationWindow,
    state: &Rc<RefCell<AppState>>,
    single_view: &Rc<RefCell<SingleView>>,
    toolbar: &Rc<Toolbar>,
    info_panel: &Rc<InfoPanel>,
) {
    let path = state.borrow().current_file().map(|p| p.to_path_buf());
    let path = match path {
        Some(p) => p,
        None => return,
    };

    let dialog = gtk4::Window::builder()
        .title("Rename")
        .transient_for(window)
        .modal(true)
        .default_width(400)
        .build();

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);

    let entry = gtk4::Entry::new();
    let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    entry.set_text(&filename);
    // Select just the stem, not extension
    let stem_len = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .len();
    entry.select_region(0, stem_len as i32);
    vbox.append(&entry);

    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk4::Align::End);
    let cancel = gtk4::Button::with_label("Cancel");
    let ok = gtk4::Button::with_label("Rename");
    ok.add_css_class("suggested-action");
    btn_box.append(&cancel);
    btn_box.append(&ok);
    vbox.append(&btn_box);

    dialog.set_child(Some(&vbox));

    let dialog_cancel = dialog.clone();
    cancel.connect_clicked(move |_| dialog_cancel.close());

    // Wire ok button to perform rename
    let do_rename = {
        let dialog = dialog.clone();
        let state = state.clone();
        let single_view = single_view.clone();
        let toolbar = toolbar.clone();
        let info_panel = info_panel.clone();
        let path = path.clone();
        let entry = entry.clone();
        move || {
            let new_name = entry.text().to_string();
            if new_name.is_empty() || new_name == path.file_name().unwrap_or_default().to_string_lossy().as_ref() {
                dialog.close();
                return;
            }
            if let Some(new_path) = crate::actions::file_ops::rename_file(&path, &new_name) {
                // Update state: remove old, re-scan folder to pick up new name
                let folder = path.parent().map(|p| p.to_path_buf());
                if let Some(folder) = folder {
                    state.borrow_mut().load_folder(&folder, Some(&new_path));
                }
                single_view.borrow().refresh_image();
                toolbar.update_single_mode();
                if info_panel.container.is_visible() {
                    let p = state.borrow().current_file().map(|p| p.to_path_buf());
                    info_panel.update(p.as_deref());
                }
            }
            dialog.close();
        }
    };

    let do_rename_ok = do_rename.clone();
    ok.connect_clicked(move |_| do_rename_ok());

    let do_rename_enter = do_rename;
    entry.connect_activate(move |_| do_rename_enter());

    dialog.present();
}

fn show_folder_dialog(
    window: &gtk4::ApplicationWindow,
    state: &Rc<RefCell<AppState>>,
    single_view: &Rc<RefCell<SingleView>>,
    toolbar: &Rc<Toolbar>,
    is_move: bool,
) {
    let path = state.borrow().current_file().map(|p| p.to_path_buf());
    let path = match path {
        Some(p) => p,
        None => return,
    };

    let file_dialog = gtk4::FileDialog::new();
    file_dialog.set_title(if is_move { "Move to folder" } else { "Copy to folder" });

    let state = state.clone();
    let single_view = single_view.clone();
    let toolbar = toolbar.clone();
    file_dialog.select_folder(Some(window), gio::Cancellable::NONE, move |result| {
        if let Ok(folder) = result {
            if let Some(dest_dir) = folder.path() {
                if is_move {
                    if crate::actions::file_ops::move_file(&path, &dest_dir).is_some() {
                        state.borrow_mut().remove_file(&path);
                        single_view.borrow().refresh_image();
                        toolbar.update_single_mode();
                    }
                } else {
                    crate::actions::file_ops::copy_file(&path, &dest_dir);
                }
            }
        }
    });
}
