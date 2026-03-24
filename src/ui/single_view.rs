use gdk4 as gdk;
use gtk4::prelude::*;
use gtk4::{self, glib};
use std::cell::RefCell;
use std::rc::Rc;

use super::filmstrip::Filmstrip;
use crate::state::app_state::AppState;

pub struct SingleView {
    pub container: gtk4::Box,
    picture: gtk4::Picture,
    filmstrip: Rc<Filmstrip>,
    state: Rc<RefCell<AppState>>,
    current_texture: Rc<RefCell<Option<gdk::Texture>>>,
}

impl SingleView {
    pub fn new(state: Rc<RefCell<AppState>>) -> Rc<RefCell<Self>> {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        let picture = gtk4::Picture::new();
        picture.set_content_fit(gtk4::ContentFit::Contain);
        picture.set_can_shrink(true);
        picture.set_vexpand(true);
        picture.set_hexpand(true);
        picture.add_css_class("single-image");

        let filmstrip = Rc::new(Filmstrip::new(state.clone()));

        container.append(&picture);
        container.append(filmstrip.widget());

        let current_texture: Rc<RefCell<Option<gdk::Texture>>> = Rc::new(RefCell::new(None));

        let view = Rc::new(RefCell::new(Self {
            container,
            picture,
            filmstrip,
            state,
            current_texture,
        }));

        // --- Scroll controller for navigation / zoom ---
        Self::setup_scroll_controller(&view);

        // --- Double-click for zoom toggle ---
        Self::setup_click_controller(&view);

        // --- Filmstrip selection callback ---
        Self::setup_filmstrip_callback(&view);

        view
    }

    fn setup_scroll_controller(view: &Rc<RefCell<Self>>) {
        let scroll_ctrl = gtk4::EventControllerScroll::new(
            gtk4::EventControllerScrollFlags::VERTICAL,
        );
        let view_ref = view.clone();
        scroll_ctrl.connect_scroll(move |ctrl, _dx, dy| {
            let mods = ctrl.current_event_state();
            let is_ctrl = mods.contains(gdk::ModifierType::CONTROL_MASK);

            if is_ctrl {
                let v = view_ref.borrow();
                if dy > 0.0 {
                    v.zoom_out();
                } else if dy < 0.0 {
                    v.zoom_in();
                }
                return glib::Propagation::Stop;
            }

            // Plain scroll = navigate
            {
                let v = view_ref.borrow();
                if dy > 0.0 {
                    v.state.borrow_mut().navigate_next();
                } else if dy < 0.0 {
                    v.state.borrow_mut().navigate_prev();
                }
            }
            view_ref.borrow().refresh_image();
            glib::Propagation::Stop
        });
        view.borrow().container.add_controller(scroll_ctrl);
    }

    fn setup_click_controller(view: &Rc<RefCell<Self>>) {
        let click = gtk4::GestureClick::new();
        click.set_button(1);
        let view_ref = view.clone();
        click.connect_released(move |_gesture, n_press, _x, _y| {
            if n_press == 2 {
                let v = view_ref.borrow();
                let is_fit = v.state.borrow().zoom_fit;
                if is_fit {
                    v.zoom_to_actual();
                } else {
                    v.zoom_to_fit();
                }
            }
        });
        view.borrow().picture.add_controller(click);
    }

    fn setup_filmstrip_callback(view: &Rc<RefCell<Self>>) {
        let view_ref = view.clone();
        view.borrow().filmstrip.set_on_select(move |idx| {
            {
                let v = view_ref.borrow();
                v.state.borrow_mut().navigate_to(idx as usize);
            }
            view_ref.borrow().refresh_image();
        });
    }

    pub fn load(&self) {
        self.filmstrip.load();
        self.load_current_image();
        let idx = self.state.borrow().index;
        self.filmstrip.update_selection(idx);
    }

    pub fn refresh_image(&self) {
        self.load_current_image();
        let idx = self.state.borrow().index;
        self.filmstrip.update_selection(idx);
    }

    pub fn load_current_image(&self) {
        let path = {
            let state = self.state.borrow();
            match state.current_file() {
                Some(p) => p.to_path_buf(),
                None => {
                    self.picture.set_paintable(gdk::Paintable::NONE);
                    *self.current_texture.borrow_mut() = None;
                    return;
                }
            }
        };

        // Check if SVG
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if crate::core::formats::detect_format(&filename)
            == Some(crate::core::formats::FormatGroup::Svg)
        {
            if let Some(tex) = crate::core::texture::svg_to_texture(&path) {
                self.picture.set_paintable(Some(&tex));
                *self.current_texture.borrow_mut() = Some(tex);
            }
            return;
        }

        // Async decode for raster images
        let picture = self.picture.clone();
        let current_texture = self.current_texture.clone();
        let state_rc = self.state.clone();

        let (tx, rx) = std::sync::mpsc::channel::<(Vec<u8>, u32, u32)>();

        std::thread::spawn(move || {
            if let Some(img) = crate::core::image_loader::load_image(&path) {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let raw = rgba.into_raw();
                let _ = tx.send((raw, w, h));
            }
        });

        // Poll the receiver on the main thread
        glib::idle_add_local(move || {
            match rx.try_recv() {
                Ok((raw, w, h)) => {
                    let bytes = glib::Bytes::from_owned(raw);
                    let texture = gdk::MemoryTexture::new(
                        w as i32,
                        h as i32,
                        gdk::MemoryFormat::R8g8b8a8,
                        &bytes,
                        (w * 4) as usize,
                    );
                    picture.set_paintable(Some(&texture));
                    if state_rc.borrow().zoom_fit {
                        picture.set_content_fit(gtk4::ContentFit::Contain);
                        picture.set_can_shrink(true);
                    }
                    *current_texture.borrow_mut() = Some(texture.upcast());
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            }
        });
    }

    pub fn zoom_in(&self) {
        let mut st = self.state.borrow_mut();
        st.zoom_fit = false;
        st.zoom = (st.zoom * 1.25).min(10.0);
        let z = st.zoom;
        drop(st);
        self.apply_zoom(z);
    }

    pub fn zoom_out(&self) {
        let mut st = self.state.borrow_mut();
        st.zoom_fit = false;
        st.zoom = (st.zoom / 1.25).max(0.1);
        let z = st.zoom;
        drop(st);
        self.apply_zoom(z);
    }

    pub fn zoom_to_fit(&self) {
        let mut st = self.state.borrow_mut();
        st.zoom_fit = true;
        st.zoom = 1.0;
        drop(st);
        self.picture.set_content_fit(gtk4::ContentFit::Contain);
        self.picture.set_can_shrink(true);
        self.picture.set_size_request(-1, -1);
    }

    pub fn zoom_to_actual(&self) {
        let mut st = self.state.borrow_mut();
        st.zoom_fit = false;
        st.zoom = 1.0;
        drop(st);
        self.apply_zoom(1.0);
    }

    fn apply_zoom(&self, factor: f64) {
        if let Some(tex) = self.current_texture.borrow().as_ref() {
            let w = (tex.width() as f64 * factor) as i32;
            let h = (tex.height() as f64 * factor) as i32;
            self.picture.set_content_fit(gtk4::ContentFit::Fill);
            self.picture.set_can_shrink(false);
            self.picture.set_size_request(w, h);
        }
    }

    pub fn set_filmstrip_visible(&self, visible: bool) {
        self.filmstrip.widget().set_visible(visible);
    }

    pub fn current_texture(&self) -> Option<gdk::Texture> {
        self.current_texture.borrow().clone()
    }

    pub fn navigate_next(&self) {
        self.state.borrow_mut().navigate_next();
        self.refresh_image();
    }

    pub fn navigate_prev(&self) {
        self.state.borrow_mut().navigate_prev();
        self.refresh_image();
    }

    pub fn navigate_to_start(&self) {
        self.state.borrow_mut().navigate_to(0);
        self.refresh_image();
    }

    pub fn navigate_to_end(&self) {
        let last = self.state.borrow().files.len().saturating_sub(1);
        self.state.borrow_mut().navigate_to(last);
        self.refresh_image();
    }
}
