use gdk4 as gdk;
use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::{self, glib, gio};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;

use crate::core::thumbnail::ThumbnailCache;
use crate::state::app_state::AppState;

// ---------------------------------------------------------------------------
// ImageItem — GObject stored in the ListStore
// ---------------------------------------------------------------------------

mod imp {
    use std::cell::{Cell, RefCell};
    use gdk4 as gdk;
    use gtk4::glib;
    use gtk4::subclass::prelude::*;

    #[derive(Default)]
    pub struct ImageItem {
        pub path: RefCell<String>,
        pub index: Cell<u32>,
        pub texture: RefCell<Option<gdk::Texture>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImageItem {
        const NAME: &'static str = "ImageItem";
        type Type = super::ImageItem;
    }

    impl ObjectImpl for ImageItem {}
}

glib::wrapper! {
    pub struct ImageItem(ObjectSubclass<imp::ImageItem>);
}

impl ImageItem {
    pub fn new(path: &str, index: u32) -> Self {
        let obj: Self = glib::Object::new();
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
// Thumbnail result sent from background threads
// ---------------------------------------------------------------------------

struct ThumbResult {
    index: u32,
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

// ---------------------------------------------------------------------------
// GridView — manages the gtk4::GridView and async thumbnail loading
// ---------------------------------------------------------------------------

pub struct GridView {
    pub container: gtk4::ScrolledWindow,
    store: gio::ListStore,
    selection: gtk4::SingleSelection,
    #[allow(dead_code)]
    grid: gtk4::GridView,
    state: Rc<RefCell<AppState>>,
    bound_widgets: Rc<RefCell<HashMap<u32, gtk4::Picture>>>,
    on_activate: Rc<RefCell<Option<Box<dyn Fn(u32)>>>>,
}

impl GridView {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let store = gio::ListStore::new::<ImageItem>();
        let selection = gtk4::SingleSelection::new(Some(store.clone()));
        selection.set_autoselect(true);

        let factory = gtk4::SignalListItemFactory::new();
        let bound_widgets: Rc<RefCell<HashMap<u32, gtk4::Picture>>> =
            Rc::new(RefCell::new(HashMap::new()));

        // --- factory setup ---
        factory.connect_setup(|_factory, obj| {
            let list_item = obj.downcast_ref::<gtk4::ListItem>().unwrap();

            let bx = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            bx.set_halign(gtk4::Align::Center);
            bx.add_css_class("grid-item");

            let picture = gtk4::Picture::new();
            picture.set_size_request(128, 128);
            picture.set_content_fit(gtk4::ContentFit::Contain);
            picture.set_can_shrink(true);

            let label = gtk4::Label::new(None);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            label.set_max_width_chars(16);
            label.add_css_class("status-text");

            bx.append(&picture);
            bx.append(&label);
            list_item.set_child(Some(&bx));
        });

        // --- factory bind ---
        let bw_bind = bound_widgets.clone();
        factory.connect_bind(move |_factory, obj| {
            let list_item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
            let item = list_item
                .item()
                .and_downcast::<ImageItem>()
                .expect("item should be ImageItem");

            let child = list_item
                .child()
                .and_downcast::<gtk4::Box>()
                .expect("child should be Box");

            let picture = child
                .first_child()
                .and_downcast::<gtk4::Picture>()
                .expect("first child should be Picture");
            let label = child
                .last_child()
                .and_downcast::<gtk4::Label>()
                .expect("last child should be Label");

            // Set filename label
            let path = PathBuf::from(item.path());
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            label.set_text(&filename);

            // If thumbnail already loaded, display it
            if let Some(tex) = item.texture() {
                picture.set_paintable(Some(&tex));
            } else {
                picture.set_paintable(gdk::Paintable::NONE);
            }

            // Track widget for async thumbnail updates
            bw_bind.borrow_mut().insert(item.index(), picture);
        });

        // --- factory unbind ---
        let bw_unbind = bound_widgets.clone();
        factory.connect_unbind(move |_factory, obj| {
            let list_item = obj.downcast_ref::<gtk4::ListItem>().unwrap();
            if let Some(item) = list_item.item().and_downcast::<ImageItem>() {
                bw_unbind.borrow_mut().remove(&item.index());
            }
        });

        let grid = gtk4::GridView::new(Some(selection.clone()), Some(factory));
        grid.set_min_columns(2);
        grid.set_max_columns(20);
        grid.add_css_class("grid-view");

        // Activate (double-click / Enter)
        let on_activate: Rc<RefCell<Option<Box<dyn Fn(u32)>>>> =
            Rc::new(RefCell::new(None));
        let on_activate_ref = on_activate.clone();
        let store_ref = store.clone();
        grid.connect_activate(move |_grid, position| {
            if let Some(item) = store_ref.item(position).and_downcast::<ImageItem>() {
                let idx = item.index();
                if let Some(ref cb) = *on_activate_ref.borrow() {
                    cb(idx);
                }
            }
        });

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&grid));

        GridView {
            container: scrolled,
            store,
            selection,
            grid,
            state,
            bound_widgets,
            on_activate,
        }
    }

    /// Register a callback for when the user activates (double-clicks) an item.
    pub fn set_on_activate<F: Fn(u32) + 'static>(&self, cb: F) {
        self.on_activate.borrow_mut().replace(Box::new(cb));
    }

    /// Populate the grid from current AppState files and kick off thumbnail loading.
    pub fn load(&self) {
        self.store.remove_all();
        self.bound_widgets.borrow_mut().clear();

        let files = self.state.borrow().files.clone();
        let current_index = self.state.borrow().index;

        // Populate store with items
        for (i, path) in files.iter().enumerate() {
            let item = ImageItem::new(&path.to_string_lossy(), i as u32);
            self.store.append(&item);
        }

        // Select current index
        self.selection.set_selected(current_index as u32);

        // Spawn async thumbnail loading
        if !files.is_empty() {
            self.load_thumbnails(files);
        }
    }

    fn load_thumbnails(&self, files: Vec<PathBuf>) {
        let (tx, rx) = mpsc::channel::<ThumbResult>();

        // Spawn a single background thread that processes all thumbnails
        std::thread::spawn(move || {
            let cache = ThumbnailCache::new(None);
            for (i, path) in files.iter().enumerate() {
                if let Some(thumb) = cache.get_thumbnail(path) {
                    let rgba = thumb.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let result = ThumbResult {
                        index: i as u32,
                        bytes: rgba.into_raw(),
                        width: w,
                        height: h,
                    };
                    if tx.send(result).is_err() {
                        break; // Receiver dropped, stop
                    }
                }
            }
        });

        // Poll the receiver from the main thread
        let store = self.store.clone();
        let bound_widgets = self.bound_widgets.clone();
        let total = self.store.n_items();

        let received = Rc::new(RefCell::new(0u32));
        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            // Drain all available results
            while let Ok(result) = rx.try_recv() {
                let texture = {
                    let gb = glib::Bytes::from_owned(result.bytes);
                    let stride = result.width as usize * 4;
                    gdk::MemoryTexture::new(
                        result.width as i32,
                        result.height as i32,
                        gdk::MemoryFormat::R8g8b8a8,
                        &gb,
                        stride,
                    )
                };

                // Store on item
                if let Some(item) = store.item(result.index).and_downcast::<ImageItem>() {
                    item.set_texture(texture.clone().upcast());
                }

                // Update visible widget if bound
                if let Some(picture) = bound_widgets.borrow().get(&result.index) {
                    picture.set_paintable(Some(&texture));
                }

                *received.borrow_mut() += 1;
            }

            if *received.borrow() >= total {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }
}
