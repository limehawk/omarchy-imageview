use gdk4 as gdk;
use gdk4::prelude::*;
use gtk4::prelude::*;
use gtk4::{self, glib};
use std::cell::RefCell;
use std::rc::Rc;

use super::filmstrip::Filmstrip;
use crate::state::app_state::{AppState, ScaleMode};

pub struct SingleView {
    pub container: gtk4::Box,
    scrolled_window: gtk4::ScrolledWindow,
    picture: gtk4::Picture,
    filmstrip: Rc<Filmstrip>,
    state: Rc<RefCell<AppState>>,
    current_texture: Rc<RefCell<Option<gdk::Texture>>>,
    /// Bumped on every load to invalidate any in-flight animation timers.
    load_epoch: Rc<std::cell::Cell<u64>>,
    anim_playing: Rc<std::cell::Cell<bool>>,
    is_animated: Rc<std::cell::Cell<bool>>,
    on_navigate: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl SingleView {
    pub fn new(state: Rc<RefCell<AppState>>) -> Rc<RefCell<Self>> {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        let picture = gtk4::Picture::new();
        picture.set_content_fit(gtk4::ContentFit::Contain);
        picture.set_can_shrink(true);
        picture.add_css_class("single-image");

        let scrolled_window = gtk4::ScrolledWindow::new();
        scrolled_window.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled_window.set_vexpand(true);
        scrolled_window.set_hexpand(true);
        scrolled_window.set_child(Some(&picture));

        let filmstrip = Rc::new(Filmstrip::new(state.clone()));

        container.append(&scrolled_window);
        container.append(filmstrip.widget());

        let current_texture: Rc<RefCell<Option<gdk::Texture>>> = Rc::new(RefCell::new(None));
        let load_epoch = Rc::new(std::cell::Cell::new(0u64));

        let view = Rc::new(RefCell::new(Self {
            container,
            scrolled_window,
            picture,
            filmstrip,
            state,
            current_texture,
            load_epoch,
            anim_playing: Rc::new(std::cell::Cell::new(true)),
            is_animated: Rc::new(std::cell::Cell::new(false)),
            on_navigate: Rc::new(RefCell::new(None)),
        }));

        // --- Scroll controller for navigation / zoom ---
        Self::setup_scroll_controller(&view);

        // --- Double-click for zoom toggle ---
        Self::setup_click_controller(&view);

        // --- Middle-click for zoom toggle ---
        Self::setup_middle_click_controller(&view);

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
            view_ref.borrow().fire_on_navigate();
            glib::Propagation::Stop
        });
        view.borrow().scrolled_window.add_controller(scroll_ctrl);
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

    fn setup_middle_click_controller(view: &Rc<RefCell<Self>>) {
        let click = gtk4::GestureClick::new();
        click.set_button(2); // middle button
        let view_ref = view.clone();
        click.connect_released(move |_gesture, _n_press, _x, _y| {
            let v = view_ref.borrow();
            let is_fit = v.state.borrow().zoom_fit;
            if is_fit {
                v.zoom_to_actual();
            } else {
                v.zoom_to_fit();
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
            view_ref.borrow().fire_on_navigate();
        });
    }

    pub fn set_on_navigate<F: Fn() + 'static>(&self, f: F) {
        *self.on_navigate.borrow_mut() = Some(Box::new(f));
    }

    fn fire_on_navigate(&self) {
        if let Some(cb) = self.on_navigate.borrow().as_ref() {
            cb();
        }
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
        // Bump epoch — any in-flight animation timer from a previous load
        // observes this and stops itself.
        let epoch = self.load_epoch.get().wrapping_add(1);
        self.load_epoch.set(epoch);
        self.anim_playing.set(true);
        self.is_animated.set(false);

        let path = {
            let state = self.state.borrow();
            match state.current_file() {
                Some(p) => p.to_path_buf(),
                None => {
                    log::debug!("load_current_image: no current file (cleared)");
                    self.picture.set_paintable(gdk::Paintable::NONE);
                    *self.current_texture.borrow_mut() = None;
                    return;
                }
            }
        };
        log::debug!("load_current_image: epoch={epoch} path={}", path.display());

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
                self.apply_display();
            }
            return;
        }

        // Animated formats (GIF, animated WebP): kick off the animated path.
        // The worker thread itself probes the file — if it turns out to be
        // a static image, that path falls through to static decode.
        let lower = filename.to_ascii_lowercase();
        if lower.ends_with(".gif") || lower.ends_with(".webp") {
            self.load_animated(path, epoch);
            return;
        }

        // Async decode for raster images
        let picture = self.picture.clone();
        let current_texture = self.current_texture.clone();
        let load_epoch = self.load_epoch.clone();
        let state = self.state.clone();

        let (tx, rx) = std::sync::mpsc::channel::<(Vec<u8>, u32, u32)>();

        std::thread::spawn(move || {
            match crate::core::image_loader::load_image(&path) {
                Some(img) => {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let raw = rgba.into_raw();
                    if tx.send((raw, w, h)).is_err() {
                        log::debug!("decode worker: receiver dropped for {}", path.display());
                    }
                }
                None => log::warn!("decode worker: load_image returned None for {}", path.display()),
            }
        });

        // Poll the receiver on the main thread
        glib::idle_add_local(move || {
            match rx.try_recv() {
                Ok((raw, w, h)) => {
                    if load_epoch.get() != epoch {
                        return glib::ControlFlow::Break;
                    }
                    let bytes = glib::Bytes::from_owned(raw);
                    let texture = gdk::MemoryTexture::new(
                        w as i32,
                        h as i32,
                        gdk::MemoryFormat::R8g8b8a8,
                        &bytes,
                        (w * 4) as usize,
                    );
                    picture.set_paintable(Some(&texture));
                    *current_texture.borrow_mut() = Some(texture.upcast());
                    apply_picture_scale(&picture, &state.borrow());
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            }
        });
    }

    /// Decode pipeline for files that *might* be animated (gif, webp). The
    /// worker probes the file and either returns animation frames or a single
    /// statically-decoded RGBA buffer; the main thread renders accordingly.
    fn load_animated(&self, path: std::path::PathBuf, epoch: u64) {
        let picture = self.picture.clone();
        let current_texture = self.current_texture.clone();
        let load_epoch = self.load_epoch.clone();
        let state = self.state.clone();
        let is_animated = self.is_animated.clone();
        let anim_playing = self.anim_playing.clone();

        enum Decoded {
            Animated(Vec<crate::core::image_loader::AnimatedFrame>),
            Static(Vec<u8>, u32, u32),
        }

        let (tx, rx) = std::sync::mpsc::channel::<Decoded>();
        std::thread::spawn(move || {
            let decoded = match crate::core::image_loader::load_animated_frames(&path) {
                Some(frames) => Some(Decoded::Animated(frames)),
                None => crate::core::image_loader::load_image(&path).map(|img| {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    Decoded::Static(rgba.into_raw(), w, h)
                }),
            };
            match decoded {
                Some(d) => {
                    if tx.send(d).is_err() {
                        log::debug!("animated worker: receiver dropped for {}", path.display());
                    }
                }
                None => log::warn!("animated worker: decode failed for {}", path.display()),
            }
        });

        glib::idle_add_local(move || match rx.try_recv() {
            Ok(decoded) => {
                if load_epoch.get() != epoch {
                    return glib::ControlFlow::Break;
                }

                match decoded {
                    Decoded::Static(raw, w, h) => {
                        let bytes = glib::Bytes::from_owned(raw);
                        let texture = gdk::MemoryTexture::new(
                            w as i32,
                            h as i32,
                            gdk::MemoryFormat::R8g8b8a8,
                            &bytes,
                            (w * 4) as usize,
                        );
                        picture.set_paintable(Some(&texture));
                        *current_texture.borrow_mut() = Some(texture.upcast());
                        apply_picture_scale(&picture, &state.borrow());
                    }
                    Decoded::Animated(frames) => {
                        if frames.is_empty() {
                            return glib::ControlFlow::Break;
                        }
                        let textures: Vec<(gdk::MemoryTexture, u32)> = frames
                            .into_iter()
                            .map(|f| {
                                let bytes = glib::Bytes::from_owned(f.rgba);
                                let tex = gdk::MemoryTexture::new(
                                    f.width as i32,
                                    f.height as i32,
                                    gdk::MemoryFormat::R8g8b8a8,
                                    &bytes,
                                    (f.width * 4) as usize,
                                );
                                (tex, f.delay_ms)
                            })
                            .collect();

                        picture.set_paintable(Some(&textures[0].0));
                        *current_texture.borrow_mut() = Some(textures[0].0.clone().upcast());
                        apply_picture_scale(&picture, &state.borrow());

                        if textures.len() > 1 {
                            is_animated.set(true);
                            anim_playing.set(true);
                            let textures = Rc::new(textures);
                            let frame_idx = Rc::new(std::cell::Cell::new(0usize));
                            schedule_next_gif_frame(
                                picture.clone(),
                                current_texture.clone(),
                                textures,
                                frame_idx,
                                load_epoch.clone(),
                                epoch,
                                anim_playing.clone(),
                            );
                        }
                    }
                }
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
        });
    }

    pub fn zoom_in(&self) {
        let mut st = self.state.borrow_mut();
        st.zoom_fit = false;
        st.zoom = (st.zoom * 1.1).min(10.0);
        let z = st.zoom;
        drop(st);
        self.apply_zoom(z);
    }

    pub fn zoom_out(&self) {
        let mut st = self.state.borrow_mut();
        st.zoom_fit = false;
        st.zoom = (st.zoom / 1.1).max(0.1);
        let z = st.zoom;
        drop(st);
        self.apply_zoom(z);
    }

    pub fn zoom_to_fit(&self) {
        let mut st = self.state.borrow_mut();
        st.zoom_fit = true;
        st.zoom = 1.0;
        st.scale_mode = ScaleMode::Fit;
        drop(st);
        self.apply_display();
    }

    pub fn zoom_to_actual(&self) {
        let mut st = self.state.borrow_mut();
        st.zoom_fit = false;
        st.zoom = 1.0;
        st.scale_mode = ScaleMode::Actual;
        drop(st);
        self.apply_display();
    }

    pub fn cycle_scale_mode(&self) {
        self.state.borrow_mut().cycle_scale_mode();
        self.apply_display();
    }

    pub fn apply_display(&self) {
        apply_picture_scale(&self.picture, &self.state.borrow());
        if !self.state.borrow().zoom_fit && self.state.borrow().zoom != 1.0 {
            let z = self.state.borrow().zoom;
            self.apply_zoom(z);
        }
    }

    /// Space: pause/play an animation, otherwise next image. Returns true if
    /// the key was consumed as a pause toggle.
    pub fn handle_space(&self) -> bool {
        if self.is_animated.get() {
            self.anim_playing.set(!self.anim_playing.get());
            return true;
        }
        false
    }

    pub fn toggle_nearest(&self) {
        let on = {
            let mut st = self.state.borrow_mut();
            st.nearest_neighbor = !st.nearest_neighbor;
            st.nearest_neighbor
        };
        if on {
            self.picture.add_css_class("nearest");
        } else {
            self.picture.remove_css_class("nearest");
        }
    }

    pub fn flip_displayed(&self, horizontal: bool) {
        let tex = match self.current_texture.borrow().as_ref() {
            Some(t) => t.clone(),
            None => return,
        };
        let w = tex.width();
        let h = tex.height();
        if w <= 0 || h <= 0 {
            return;
        }
        let stride = w as usize * 4;
        let mut buf = vec![0u8; stride * h as usize];
        tex.download(&mut buf, stride);
        let Some(rgba) = image::RgbaImage::from_raw(w as u32, h as u32, buf) else {
            return;
        };
        let img = image::DynamicImage::ImageRgba8(rgba);
        let flipped = if horizontal { img.fliph() } else { img.flipv() };
        let out = flipped.to_rgba8();
        let (nw, nh) = out.dimensions();
        let texture = gdk::MemoryTexture::new(
            nw as i32,
            nh as i32,
            gdk::MemoryFormat::R8g8b8a8,
            &glib::Bytes::from_owned(out.into_raw()),
            (nw * 4) as usize,
        );
        self.picture.set_paintable(Some(&texture));
        *self.current_texture.borrow_mut() = Some(texture.upcast());
        self.apply_display();
        if horizontal {
            self.state.borrow_mut().toggle_flip_h();
        } else {
            self.state.borrow_mut().toggle_flip_v();
        }
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

    /// Rotate the displayed image in memory. Does not rewrite the file.
    pub fn rotate_displayed(&self, degrees: u32) {
        let tex = match self.current_texture.borrow().as_ref() {
            Some(t) => t.clone(),
            None => return,
        };
        let w = tex.width();
        let h = tex.height();
        if w <= 0 || h <= 0 {
            return;
        }
        let stride = w as usize * 4;
        let mut buf = vec![0u8; stride * h as usize];
        tex.download(&mut buf, stride);
        let Some(rgba) = image::RgbaImage::from_raw(w as u32, h as u32, buf) else {
            log::error!("rotate_displayed: could not wrap pixels");
            return;
        };
        let img = image::DynamicImage::ImageRgba8(rgba);
        let rotated = match degrees {
            90 => img.rotate90(),
            180 => img.rotate180(),
            270 => img.rotate270(),
            _ => return,
        };
        let out = rotated.to_rgba8();
        let (nw, nh) = out.dimensions();
        let texture = gdk::MemoryTexture::new(
            nw as i32,
            nh as i32,
            gdk::MemoryFormat::R8g8b8a8,
            &glib::Bytes::from_owned(out.into_raw()),
            (nw * 4) as usize,
        );
        self.picture.set_paintable(Some(&texture));
        *self.current_texture.borrow_mut() = Some(texture.upcast());

        let (zoom_fit, zoom) = {
            let st = self.state.borrow();
            (st.zoom_fit, st.zoom)
        };
        if zoom_fit {
            self.apply_display();
        } else {
            self.apply_zoom(zoom);
        }
        self.state.borrow_mut().add_rotation(degrees);
    }

    /// Write pending rotation and flips to the current file.
    pub fn save_rotation(&self) -> bool {
        let (path, degrees, flip_h, flip_v) = {
            let st = self.state.borrow();
            (
                st.current_file().map(|p| p.to_path_buf()),
                st.pending_rotation,
                st.pending_flip_h,
                st.pending_flip_v,
            )
        };
        let Some(path) = path else {
            return false;
        };
        if degrees == 0 && !flip_h && !flip_v {
            return true;
        }
        if !crate::actions::file_ops::save_transform(&path, degrees, flip_h, flip_v) {
            return false;
        }
        let mut st = self.state.borrow_mut();
        st.pending_rotation = 0;
        st.pending_flip_h = false;
        st.pending_flip_v = false;
        drop(st);
        self.filmstrip.load();
        true
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

fn schedule_next_gif_frame(
    picture: gtk4::Picture,
    current_texture: Rc<RefCell<Option<gdk::Texture>>>,
    textures: Rc<Vec<(gdk::MemoryTexture, u32)>>,
    frame_idx: Rc<std::cell::Cell<usize>>,
    load_epoch: Rc<std::cell::Cell<u64>>,
    my_epoch: u64,
    playing: Rc<std::cell::Cell<bool>>,
) {
    let delay = textures[frame_idx.get()].1.max(20);
    glib::timeout_add_local_once(std::time::Duration::from_millis(delay as u64), move || {
        if load_epoch.get() != my_epoch {
            return;
        }
        if playing.get() {
            let next = (frame_idx.get() + 1) % textures.len();
            frame_idx.set(next);
            let tex = &textures[next].0;
            picture.set_paintable(Some(tex));
            *current_texture.borrow_mut() = Some(tex.clone().upcast());
        }
        schedule_next_gif_frame(
            picture,
            current_texture,
            textures,
            frame_idx,
            load_epoch,
            my_epoch,
            playing,
        );
    });
}

fn apply_picture_scale(picture: &gtk4::Picture, st: &AppState) {
    if !st.zoom_fit && st.zoom != 1.0 {
        return;
    }
    match st.scale_mode {
        ScaleMode::Fit => {
            picture.set_content_fit(gtk4::ContentFit::Contain);
            picture.set_can_shrink(true);
            picture.set_size_request(-1, -1);
        }
        ScaleMode::Fill => {
            picture.set_content_fit(gtk4::ContentFit::Cover);
            picture.set_can_shrink(true);
            picture.set_size_request(-1, -1);
        }
        ScaleMode::Actual => {
            picture.set_content_fit(gtk4::ContentFit::Fill);
            picture.set_can_shrink(false);
            if let Some(tex) = picture.paintable() {
                picture.set_size_request(tex.intrinsic_width(), tex.intrinsic_height());
            }
        }
    }
}
