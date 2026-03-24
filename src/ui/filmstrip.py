"""Horizontal filmstrip — thumbnail strip along bottom of single view."""

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import GLib, GObject, Gio, Gtk

from concurrent.futures import ThreadPoolExecutor

from ..core.thumbnail import ThumbnailCache
from ..core.texture import image_to_texture
from ..state.app_state import AppState


class FilmstripItem(GObject.Object):
    __gtype_name__ = "FilmstripItem"

    def __init__(self, path, index):
        super().__init__()
        self._path = path
        self._index = index
        self._texture = None

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


class Filmstrip(Gtk.Box):
    """Horizontal thumbnail filmstrip."""

    __gsignals__ = {
        "image-selected": (GObject.SignalFlags.RUN_FIRST, None, (int,)),
    }

    def __init__(self, state: AppState):
        super().__init__(orientation=Gtk.Orientation.VERTICAL)
        self.add_css_class("filmstrip")
        self.state = state
        self._cache = ThumbnailCache()
        self._pool = ThreadPoolExecutor(max_workers=2)

        self._store = Gio.ListStore(item_type=FilmstripItem)
        self._selection = Gtk.SingleSelection(model=self._store)

        factory = Gtk.SignalListItemFactory()
        factory.connect("setup", self._on_setup)
        factory.connect("bind", self._on_bind)
        factory.connect("unbind", self._on_unbind)

        self._list = Gtk.ListView(model=self._selection, factory=factory)
        self._list.set_orientation(Gtk.Orientation.HORIZONTAL)
        self._list.connect("activate", self._on_activate)

        sw = Gtk.ScrolledWindow()
        sw.set_policy(Gtk.PolicyType.AUTOMATIC, Gtk.PolicyType.NEVER)
        sw.set_min_content_height(72)
        sw.set_max_content_height(72)
        sw.set_child(self._list)
        self.append(sw)

        self._sw = sw

    def load(self):
        self._store.remove_all()
        for i, path in enumerate(self.state.files):
            item = FilmstripItem(path, i)
            self._store.append(item)
            self._request_thumbnail(item)
        self.scroll_to_current()

    def scroll_to_current(self):
        if self.state.files:
            self._selection.set_selected(self.state.index)
            self._list.scroll_to(self.state.index, Gtk.ListScrollFlags.FOCUS, None)

    def update_selection(self):
        if self.state.files:
            self._selection.set_selected(self.state.index)
            self.scroll_to_current()

    def _request_thumbnail(self, item):
        path = item.path

        def generate():
            return self._cache.get_thumbnail(path)

        def on_done(future):
            thumb = future.result()
            if thumb is not None:
                GLib.idle_add(self._apply_thumbnail, item, thumb)

        future = self._pool.submit(generate)
        future.add_done_callback(on_done)

    def _apply_thumbnail(self, item, thumb):
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
        picture = Gtk.Picture()
        picture.set_size_request(56, 56)
        picture.set_content_fit(Gtk.ContentFit.CONTAIN)
        picture.set_can_shrink(True)
        picture.add_css_class("filmstrip-item")
        list_item.set_child(picture)

    def _on_bind(self, factory, list_item):
        picture = list_item.get_child()
        item = list_item.get_item()
        if item.texture:
            picture.set_paintable(item.texture)
        if item.index == self.state.index:
            picture.remove_css_class("filmstrip-item")
            picture.add_css_class("filmstrip-current")
        else:
            picture.remove_css_class("filmstrip-current")
            picture.add_css_class("filmstrip-item")

    def _on_unbind(self, factory, list_item):
        picture = list_item.get_child()
        picture.set_paintable(None)

    def _on_activate(self, list_view, position):
        self.state.index = position
        self.emit("image-selected", position)
