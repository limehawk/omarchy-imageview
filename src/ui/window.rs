use gtk4::prelude::*;
use gtk4::{self, gdk, glib};
use std::cell::RefCell;
use std::rc::Rc;

use crate::state::app_state::{AppState, ViewMode};
use super::grid_view::GridView;
use super::single_view::SingleView;
use super::theme;

pub struct ImageViewerWindow {
    pub window: gtk4::ApplicationWindow,
    pub stack: gtk4::Stack,
    state: Rc<RefCell<AppState>>,
    grid_view: GridView,
    single_view: Rc<RefCell<SingleView>>,
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

        let grid_view = GridView::new(state.clone());
        stack.add_named(&grid_view.container, Some("grid"));

        let single_view = SingleView::new(state.clone());
        stack.add_named(&single_view.borrow().container, Some("single"));

        main_box.append(&stack);
        window.set_child(Some(&main_box));

        let viewer = Rc::new(RefCell::new(Self {
            window: window.clone(),
            stack: stack.clone(),
            state: state.clone(),
            grid_view,
            single_view: single_view.clone(),
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
                    } else {
                        window_ref.fullscreen();
                        if is_single {
                            single_ref.borrow().set_filmstrip_visible(false);
                        }
                    }
                    glib::Propagation::Stop
                }
                // Single view navigation
                "Right" | "Down" if is_single && !ctrl => {
                    single_ref.borrow().navigate_next();
                    glib::Propagation::Stop
                }
                "Left" | "Up" if is_single && !ctrl => {
                    single_ref.borrow().navigate_prev();
                    glib::Propagation::Stop
                }
                "Home" if is_single => {
                    single_ref.borrow().navigate_to_start();
                    glib::Propagation::Stop
                }
                "End" if is_single => {
                    single_ref.borrow().navigate_to_end();
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
    }

    pub fn show_single(&self) {
        self.state.borrow_mut().view_mode = ViewMode::Single;
        self.stack.set_visible_child_name("single");
        self.single_view.borrow().load();
    }

    pub fn present(&self) {
        self.window.present();
    }
}
