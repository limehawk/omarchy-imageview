use gdk4 as gdk;
use glib::subclass::prelude::*;
use gtk4::prelude::*;
use gtk4::{self, gio, glib};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;

use crate::core::thumbnail::ThumbnailCache;
use crate::state::app_state::AppState;

// ---------------------------------------------------------------------------
// FilmstripItem GObject
// ---------------------------------------------------------------------------

mod imp_item {
    use super::*;

    #[derive(Default)]
    pub struct FilmstripItemInner {
        pub path: RefCell<String>,
        pub index: Cell<u32>,
        pub texture: RefCell<Option<gdk::Texture>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilmstripItemInner {
        const NAME: &'static str = "FilmstripItem";
        type Type = super::FilmstripItem;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for FilmstripItemInner {}
}

glib::wrapper! {
    pub struct FilmstripItem(ObjectSubclass<imp_item::FilmstripItemInner>);
}

impl FilmstripItem {
    pub fn new(path: &str, index: u32) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.imp().path.replace(path.to_string());
        obj.imp().index.set(index);
        obj
    }

    pub fn path(&self) -> String {
        self.imp().path.borrow().clone()
    }

    pub fn index(&self) -> u32 {
        self.imp().index.get()
    }

    pub fn texture(&self) -> Option<gdk::Texture> {
        self.imp().texture.borrow().clone()
    }

    pub fn set_texture(&self, tex: gdk::Texture) {
        self.imp().texture.replace(Some(tex));
    }
}

// ---------------------------------------------------------------------------
// Filmstrip
// ---------------------------------------------------------------------------

pub struct Filmstrip {
    pub container: gtk4::ScrolledWindow,
    store: gio::ListStore,
    selection: gtk4::SingleSelection,
    list_view: gtk4::ListView,
    state: Rc<RefCell<AppState>>,
    on_select: Rc<RefCell<Option<Box<dyn Fn(u32)>>>>,
    bound_widgets: Rc<RefCell<HashMap<u32, gtk4::Picture>>>,
    loading: Rc<Cell<bool>>, // suppress selection callback during load
}

impl Filmstrip {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let store = gio::ListStore::new::<FilmstripItem>();
        let selection = gtk4::SingleSelection::new(Some(store.clone()));
        selection.set_autoselect(false);
        selection.set_can_unselect(true);

        let factory = gtk4::SignalListItemFactory::new();

        // Setup: create a Picture widget for each cell
        factory.connect_setup(|_factory, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            let picture = gtk4::Picture::new();
            picture.set_content_fit(gtk4::ContentFit::Cover);
            picture.set_can_shrink(true);
            picture.set_size_request(64, 64);
            picture.add_css_class("filmstrip-item");
            list_item.set_child(Some(&picture));
        });

        // Bind: populate from item data + track widget
        let bound_widgets: Rc<RefCell<HashMap<u32, gtk4::Picture>>> =
            Rc::new(RefCell::new(HashMap::new()));

        let bound_bind = bound_widgets.clone();
        let state_bind = state.clone();
        factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            let item = list_item.item().and_downcast::<FilmstripItem>().unwrap();
            let picture = list_item.child().and_downcast::<gtk4::Picture>().unwrap();

            if let Some(tex) = item.texture() {
                picture.set_paintable(Some(&tex));
            } else {
                picture.set_paintable(gdk::Paintable::NONE);
            }

            // Highlight current image
            let current_idx = state_bind.borrow().index as u32;
            picture.remove_css_class("filmstrip-item");
            picture.remove_css_class("filmstrip-current");
            if item.index() == current_idx {
                picture.add_css_class("filmstrip-current");
            } else {
                picture.add_css_class("filmstrip-item");
            }

            bound_bind.borrow_mut().insert(item.index(), picture);
        });

        // Unbind: stop tracking widget
        let bound_unbind = bound_widgets.clone();
        factory.connect_unbind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            if let Some(item) = list_item.item().and_downcast::<FilmstripItem>() {
                bound_unbind.borrow_mut().remove(&item.index());
            }
        });

        let list_view = gtk4::ListView::new(Some(selection.clone()), Some(factory));
        list_view.set_orientation(gtk4::Orientation::Horizontal);
        list_view.add_css_class("filmstrip");

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_child(Some(&list_view));
        scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Never);
        scroll.set_min_content_height(80);
        scroll.set_max_content_height(80);
        scroll.set_vexpand(false);
        scroll.set_size_request(-1, 80);

        let on_select: Rc<RefCell<Option<Box<dyn Fn(u32)>>>> =
            Rc::new(RefCell::new(None));
        let loading = Rc::new(Cell::new(false));

        // Selection changed → navigate to clicked thumbnail
        let on_select_ref = on_select.clone();
        let loading_ref = loading.clone();
        selection.connect_selection_changed(move |sel, _, _| {
            // Suppress during load to prevent cascading navigations
            if loading_ref.get() {
                return;
            }
            let idx = sel.selected();
            if idx != gtk4::INVALID_LIST_POSITION {
                if let Some(ref cb) = *on_select_ref.borrow() {
                    cb(idx);
                }
            }
        });

        Self {
            container: scroll,
            store,
            selection,
            list_view,
            state,
            on_select,
            bound_widgets,
            loading,
        }
    }

    pub fn set_on_select<F: Fn(u32) + 'static>(&self, f: F) {
        *self.on_select.borrow_mut() = Some(Box::new(f));
    }

    pub fn load(&self) {
        // Suppress selection callbacks while populating
        self.loading.set(true);
        self.store.remove_all();
        self.bound_widgets.borrow_mut().clear();

        let paths: Vec<(String, u32)> = {
            let st = self.state.borrow();
            st.files
                .iter()
                .enumerate()
                .map(|(i, p)| (p.to_string_lossy().to_string(), i as u32))
                .collect()
        };

        for (path, idx) in &paths {
            self.store.append(&FilmstripItem::new(path, *idx));
        }

        self.loading.set(false);

        // Start async thumbnail loading with a bounded worker pool
        self.load_thumbnails(&paths);
    }

    fn load_thumbnails(&self, paths: &[(String, u32)]) {
        if paths.is_empty() {
            return;
        }

        // Send all paths to a single background thread that processes them sequentially
        // This avoids spawning hundreds of threads
        let (work_tx, work_rx) = mpsc::channel::<(u32, PathBuf)>();
        let (result_tx, result_rx) = mpsc::channel::<(u32, Vec<u8>, u32, u32)>();

        // Queue work starting from current index, spiraling outward
        // so visible filmstrip thumbnails load first
        let current = self.state.borrow().index;
        let len = paths.len();
        let mut order: Vec<usize> = Vec::with_capacity(len);
        order.push(current.min(len.saturating_sub(1)));
        for offset in 1..len {
            if current + offset < len {
                order.push(current + offset);
            }
            if offset <= current {
                order.push(current - offset);
            }
        }
        for i in order {
            let (ref path_str, idx) = paths[i];
            let _ = work_tx.send((idx, PathBuf::from(path_str)));
        }
        drop(work_tx);

        // Single background thread processes thumbnails
        std::thread::spawn(move || {
            let cache = ThumbnailCache::new(None);
            while let Ok((idx, path)) = work_rx.recv() {
                if let Some(thumb) = cache.get_thumbnail(&path) {
                    let rgba = thumb.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let raw = rgba.into_raw();
                    let _ = result_tx.send((idx, raw, w, h));
                }
            }
        });

        // Poll results on main thread every 32ms (not idle — idle can starve on initial render)
        let bound = self.bound_widgets.clone();
        let store = self.store.clone();
        let list_view = self.list_view.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(32), move || {
            let mut updated = false;
            // Drain all available results
            loop {
                match result_rx.try_recv() {
                    Ok((idx, raw, w, h)) => {
                        updated = true;
                        let bytes = glib::Bytes::from_owned(raw);
                        let texture = gdk::MemoryTexture::new(
                            w as i32,
                            h as i32,
                            gdk::MemoryFormat::R8g8b8a8,
                            &bytes,
                            (w * 4) as usize,
                        );
                        let tex_ref: gdk::Texture = texture.upcast();

                        // Store texture on the item for future bind calls
                        if let Some(obj) = store.item(idx) {
                            if let Some(item) = obj.downcast_ref::<FilmstripItem>() {
                                item.set_texture(tex_ref.clone());
                            }
                        }

                        // Update currently-bound widget directly
                        if let Some(picture) = bound.borrow().get(&idx) {
                            picture.set_paintable(Some(&tex_ref));
                            picture.queue_draw();
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        // Force final repaint
                        list_view.queue_draw();
                        return glib::ControlFlow::Break;
                    }
                }
            }
            if updated {
                list_view.queue_draw();
            }
            glib::ControlFlow::Continue
        });
    }

    pub fn update_selection(&self, index: usize) {
        self.loading.set(true);
        let pos = index as u32;
        if pos < self.store.n_items() {
            self.selection.set_selected(pos);
            self.list_view
                .scroll_to(pos, gtk4::ListScrollFlags::NONE, None);

            // Update CSS classes on visible items
            for (&idx, picture) in self.bound_widgets.borrow().iter() {
                picture.remove_css_class("filmstrip-item");
                picture.remove_css_class("filmstrip-current");
                if idx == pos {
                    picture.add_css_class("filmstrip-current");
                } else {
                    picture.add_css_class("filmstrip-item");
                }
            }
        }
        self.loading.set(false);
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.container
    }
}
