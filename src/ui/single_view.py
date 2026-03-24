"""Single image view with zoom, pan, and filmstrip."""

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import GLib, Gdk, Gtk

from concurrent.futures import ThreadPoolExecutor

from ..core.formats import FormatGroup, detect_format
from ..core.image_loader import load_image
from ..core.texture import image_to_texture, svg_to_texture
from ..state.app_state import AppState
from .filmstrip import Filmstrip


class SingleView(Gtk.Box):
    """Full-size image display with filmstrip navigation."""

    def __init__(self, state: AppState):
        super().__init__(orientation=Gtk.Orientation.VERTICAL)
        self.state = state
        self._pool = ThreadPoolExecutor(max_workers=2)
        self._current_texture = None

        # Main image area
        self._picture = Gtk.Picture()
        self._picture.set_content_fit(Gtk.ContentFit.CONTAIN)
        self._picture.set_can_shrink(True)
        self._picture.set_vexpand(True)
        self._picture.set_hexpand(True)

        scroll_ctrl = Gtk.EventControllerScroll(
            flags=Gtk.EventControllerScrollFlags.VERTICAL
        )
        scroll_ctrl.connect("scroll", self._on_scroll)
        self._picture.add_controller(scroll_ctrl)

        click_ctrl = Gtk.GestureClick()
        click_ctrl.set_button(1)
        click_ctrl.connect("released", self._on_click)
        self._picture.add_controller(click_ctrl)

        self.append(self._picture)

        # Filmstrip
        self._filmstrip = Filmstrip(state)
        self._filmstrip.connect("image-selected", self._on_filmstrip_select)
        self.append(self._filmstrip)

    @property
    def filmstrip(self):
        return self._filmstrip

    @property
    def current_texture(self):
        return self._current_texture

    def load(self):
        self._filmstrip.load()
        self._load_current_image()

    def refresh_image(self):
        self._load_current_image()
        self._filmstrip.update_selection()

    def _load_current_image(self):
        path = self.state.current_file
        if path is None:
            self._picture.set_paintable(None)
            return

        fmt = detect_format(path.name)
        if fmt == FormatGroup.SVG:
            texture = svg_to_texture(path)
            if texture:
                self._current_texture = texture
                self._picture.set_paintable(texture)
            return

        def decode():
            return load_image(path)

        def on_done(future):
            img = future.result()
            if img is not None:
                GLib.idle_add(self._apply_image, img)

        future = self._pool.submit(decode)
        future.add_done_callback(on_done)

    def _apply_image(self, img):
        texture = image_to_texture(img)
        self._current_texture = texture
        self._picture.set_paintable(texture)
        if self.state.zoom_fit:
            self._picture.set_content_fit(Gtk.ContentFit.CONTAIN)
            self._picture.set_can_shrink(True)
        return False

    def _on_scroll(self, controller, dx, dy):
        mods = controller.get_current_event_state()
        ctrl = mods & Gdk.ModifierType.CONTROL_MASK

        if ctrl:
            if dy > 0:
                self._zoom_out()
            elif dy < 0:
                self._zoom_in()
            return True

        if dy > 0:
            self.state.navigate_next()
        elif dy < 0:
            self.state.navigate_prev()
        self.refresh_image()
        return True

    def _on_click(self, gesture, n_press, x, y):
        if n_press == 2:
            if self.state.zoom_fit:
                self.state.zoom_fit = False
                self.state.zoom = 1.0
            else:
                self.state.zoom_fit = True
            self._apply_zoom()

    def _zoom_in(self):
        self.state.zoom_fit = False
        self.state.zoom = min(self.state.zoom * 1.25, 10.0)
        self._apply_zoom()

    def _zoom_out(self):
        self.state.zoom_fit = False
        self.state.zoom = max(self.state.zoom / 1.25, 0.1)
        self._apply_zoom()

    def zoom_to_fit(self):
        self.state.zoom_fit = True
        self._apply_zoom()

    def zoom_to_actual(self):
        self.state.zoom_fit = False
        self.state.zoom = 1.0
        self._apply_zoom()

    def _apply_zoom(self):
        if self.state.zoom_fit:
            self._picture.set_content_fit(Gtk.ContentFit.CONTAIN)
            self._picture.set_can_shrink(True)
            self._picture.set_size_request(-1, -1)
        else:
            texture = self._current_texture
            if texture:
                w = int(texture.get_width() * self.state.zoom)
                h = int(texture.get_height() * self.state.zoom)
                self._picture.set_content_fit(Gtk.ContentFit.FILL)
                self._picture.set_size_request(w, h)

    def _on_filmstrip_select(self, filmstrip, index):
        self.state.index = index
        self.refresh_image()

    def set_filmstrip_visible(self, visible):
        self._filmstrip.set_visible(visible)
