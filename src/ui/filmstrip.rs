use gdk4 as gdk;
use glib::subclass::prelude::*;
use gtk4::prelude::*;
use gtk4::{self, gio, glib};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::core::thumbnail::ThumbnailCache;
use crate::state::app_state::AppState;

// ---------------------------------------------------------------------------
// FilmstripItem GObject
// ---------------------------------------------------------------------------

mod imp_item {
    use super::*;
    use std::cell::Cell;

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
}

impl Filmstrip {
    pub fn new(state: Rc<RefCell<AppState>>) -> Self {
        let store = gio::ListStore::new::<FilmstripItem>();
        let selection = gtk4::SingleSelection::new(Some(store.clone()));
        selection.set_autoselect(false);
        selection.set_can_unselect(true);

        let factory = gtk4::SignalListItemFactory::new();

        factory.connect_setup(|_factory, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            let picture = gtk4::Picture::new();
            picture.set_content_fit(gtk4::ContentFit::Cover);
            picture.set_can_shrink(true);
            picture.set_size_request(64, 64);
            picture.add_css_class("filmstrip-thumb");
            list_item.set_child(Some(&picture));
        });

        factory.connect_bind(|_factory, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            let item = list_item.item().and_downcast::<FilmstripItem>().unwrap();
            let picture = list_item.child().and_downcast::<gtk4::Picture>().unwrap();
            if let Some(tex) = item.texture() {
                picture.set_paintable(Some(&tex));
            } else {
                picture.set_paintable(gdk::Paintable::NONE);
            }
        });

        let list_view = gtk4::ListView::new(Some(selection.clone()), Some(factory));
        list_view.set_orientation(gtk4::Orientation::Horizontal);
        list_view.add_css_class("filmstrip");

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_child(Some(&list_view));
        scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Never);
        scroll.set_min_content_height(72);
        scroll.set_max_content_height(72);
        scroll.set_vexpand(false);

        let on_select: Rc<RefCell<Option<Box<dyn Fn(u32)>>>> = Rc::new(RefCell::new(None));

        // Connect selection changed
        let on_select_ref = on_select.clone();
        selection.connect_selection_changed(move |sel, _, _| {
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
        }
    }

    pub fn set_on_select<F: Fn(u32) + 'static>(&self, f: F) {
        *self.on_select.borrow_mut() = Some(Box::new(f));
    }

    pub fn load(&self) {
        self.store.remove_all();

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

        // Async thumbnail loading
        let store = self.store.clone();

        for (i, (path_str, _idx)) in paths.iter().enumerate() {
            let path = PathBuf::from(path_str);
            let store_clone = store.clone();
            let pos = i as u32;

            let (tx, rx) = std::sync::mpsc::channel::<(Vec<u8>, u32, u32)>();

            std::thread::spawn(move || {
                let cache = ThumbnailCache::new(None);
                if let Some(thumb) = cache.get_thumbnail(&path) {
                    let rgba = thumb.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let raw = rgba.into_raw();
                    let _ = tx.send((raw, w, h));
                }
            });

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
                        if let Some(obj) = store_clone.item(pos) {
                            if let Some(item) = obj.downcast_ref::<FilmstripItem>() {
                                item.set_texture(texture.upcast());
                                store_clone.items_changed(pos, 1, 1);
                            }
                        }
                        glib::ControlFlow::Break
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
                }
            });
        }
    }

    pub fn update_selection(&self, index: usize) {
        let pos = index as u32;
        if pos < self.store.n_items() {
            self.selection.set_selected(pos);
            // Scroll to the selected item
            self.list_view
                .scroll_to(pos, gtk4::ListScrollFlags::NONE, None);
        }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.container
    }
}
