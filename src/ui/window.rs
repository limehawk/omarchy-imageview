use gtk4::prelude::*;
use gtk4::{self, gdk, glib};
use std::cell::RefCell;
use std::rc::Rc;

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
        }));

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
                    // Stubs for future tasks
                    "rotate" | "copy" | "trash" => {}
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
                "BackSpace" => {
                    if is_single {
                        viewer_ref.borrow().show_grid();
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
                // Zoom keys
                "plus" | "equal" if is_single && ctrl => {
                    single_ref.borrow().zoom_in();
                    glib::Propagation::Stop
                }
                "minus" if is_single && ctrl => {
                    single_ref.borrow().zoom_out();
                    glib::Propagation::Stop
                }
                "0" if is_single && ctrl => {
                    single_ref.borrow().zoom_to_fit();
                    glib::Propagation::Stop
                }
                "1" if is_single && ctrl => {
                    single_ref.borrow().zoom_to_actual();
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
    }

    pub fn show_single(&self) {
        self.state.borrow_mut().view_mode = ViewMode::Single;
        self.stack.set_visible_child_name("single");
        self.single_view.borrow().load();
        self.toolbar.update_single_mode();
    }

    pub fn present(&self) {
        self.window.present();
    }
}
