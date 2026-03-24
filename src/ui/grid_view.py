"""Thumbnail grid view using GtkGridView."""

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import GLib, GObject, Gdk, Gio, Gtk

from concurrent.futures import ThreadPoolExecutor

from ..core.thumbnail import ThumbnailCache
from ..core.texture import image_to_texture
from ..state.app_state import AppState


class ImageItem(GObject.Object):
    """List item representing one image in the grid."""

    __gtype_name__ = "ImageItem"

    def __init__(self, path, index):
        super().__init__()
        self._path = path
        self._index = index
        self._texture = None

    @GObject.Property(type=str)
    def filename(self):
        return self._path.name

    @GObject.Property(type=int)
    def index(self):
        return self._index

    @property
    def path(self):
        return self._path

    @property
    def texture(self):
        return self._texture

    @texture.setter
    def texture(self, value):
        self._texture = value


class GridView(Gtk.Box):
    """Thumbnail grid browser."""

    __gsignals__ = {
        "image-activated": (GObject.SignalFlags.RUN_FIRST, None, (int,)),
    }

    def __init__(self, state: AppState):
        super().__init__(orientation=Gtk.Orientation.VERTICAL)
        self.state = state
        self._cache = ThumbnailCache()
        self._pool = ThreadPoolExecutor(max_workers=4)
        self._items = {}

        self._store = Gio.ListStore(item_type=ImageItem)
        self._selection = Gtk.SingleSelection(model=self._store)
        self._selection.connect("notify::selected", self._on_selection_changed)

        factory = Gtk.SignalListItemFactory()
        factory.connect("setup", self._on_setup)
        factory.connect("bind", self._on_bind)
        factory.connect("unbind", self._on_unbind)

        self._grid = Gtk.GridView(model=self._selection, factory=factory)
        self._grid.set_min_columns(3)
        self._grid.set_max_columns(12)
        self._grid.connect("activate", self._on_activate)

        sw = Gtk.ScrolledWindow(vexpand=True)
        sw.set_child(self._grid)
        self.append(sw)

    def load(self):
        self._store.remove_all()
        self._items.clear()
        for i, path in enumerate(self.state.files):
            item = ImageItem(path, i)
            self._items[path] = item
            self._store.append(item)
            self._request_thumbnail(item)

        if self.state.files:
            self._selection.set_selected(self.state.index)

    def _request_thumbnail(self, item: ImageItem):
        path = item.path

        def generate():
            return self._cache.get_thumbnail(path)

        def on_done(future):
            thumb = future.result()
            if thumb is not None:
                GLib.idle_add(self._apply_thumbnail, item, thumb)

        future = self._pool.submit(generate)
        future.add_done_callback(on_done)

    def _apply_thumbnail(self, item: ImageItem, thumb):
        try:
            item.texture = image_to_texture(thumb)
            for i in range(self._store.get_n_items()):
                if self._store.get_item(i) is item:
                    self._store.items_changed(i, 1, 1)
                    break
        except Exception:
            pass
        return False

    def _on_setup(self, factory, list_item):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        box.set_halign(Gtk.Align.CENTER)
        box.add_css_class("grid-item")

        picture = Gtk.Picture()
        picture.set_size_request(128, 128)
        picture.set_content_fit(Gtk.ContentFit.CONTAIN)
        picture.set_can_shrink(True)

        label = Gtk.Label()
        label.set_ellipsize(3)  # Pango.EllipsizeMode.END
        label.set_max_width_chars(16)
        label.add_css_class("status-text")

        box.append(picture)
        box.append(label)
        list_item.set_child(box)

    def _on_bind(self, factory, list_item):
        box = list_item.get_child()
        item = list_item.get_item()
        picture = box.get_first_child()
        label = picture.get_next_sibling()

        label.set_text(item.filename)
        if item.texture:
            picture.set_paintable(item.texture)

    def _on_unbind(self, factory, list_item):
        box = list_item.get_child()
        picture = box.get_first_child()
        picture.set_paintable(None)

    def _on_activate(self, grid_view, position):
        self.state.index = position
        self.emit("image-activated", position)

    def _on_selection_changed(self, selection, pspec):
        self.state.index = selection.get_selected()

    def remove_item(self, path):
        item = self._items.pop(path, None)
        if item is not None:
            for i in range(self._store.get_n_items()):
                if self._store.get_item(i) is item:
                    self._store.remove(i)
                    break
