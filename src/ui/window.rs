use gtk4::prelude::*;
use gtk4::{self, gdk, glib};
use std::cell::RefCell;
use std::rc::Rc;

use crate::state::app_state::{AppState, ViewMode};
use super::theme;

pub struct ImageViewerWindow {
    pub window: gtk4::ApplicationWindow,
    pub stack: gtk4::Stack,
    state: Rc<RefCell<AppState>>,
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

        // Main layout
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        // Stack with placeholder views
        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_transition_duration(150);

        let grid_label = gtk4::Label::new(Some("Grid View"));
        grid_label.add_css_class("empty-state");
        let single_label = gtk4::Label::new(Some("Single View"));
        single_label.add_css_class("empty-state");

        stack.add_named(&grid_label, Some("grid"));
        stack.add_named(&single_label, Some("single"));

        main_box.append(&stack);
        window.set_child(Some(&main_box));

        let viewer = Rc::new(RefCell::new(Self {
            window: window.clone(),
            stack: stack.clone(),
            state: state.clone(),
        }));

        // Keybindings
        let controller = gtk4::EventControllerKey::new();
        let viewer_ref = viewer.clone();
        let state_ref = state.clone();
        let window_ref = window.clone();

        controller.connect_key_pressed(move |_ctrl, keyval, _keycode, modifiers| {
            let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
            let key_name = keyval.name().unwrap_or_default();

            match key_name.as_str() {
                "Escape" => {
                    let is_single = state_ref.borrow().view_mode == ViewMode::Single;
                    if is_single {
                        viewer_ref.borrow().show_grid();
                    } else {
                        window_ref.close();
                    }
                    glib::Propagation::Stop
                }
                "BackSpace" => {
                    let is_single = state_ref.borrow().view_mode == ViewMode::Single;
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
                    } else {
                        window_ref.fullscreen();
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
        self.stack.set_visible_child_name("grid");
    }

    pub fn show_single(&self) {
        self.state.borrow_mut().view_mode = ViewMode::Single;
        self.stack.set_visible_child_name("single");
    }

    pub fn present(&self) {
        self.window.present();
    }
}
