"""Context-aware toolbar — adapts content for grid vs single view."""

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import GObject, Gtk

from ..state.app_state import AppState


class Toolbar(Gtk.Box):
    __gsignals__ = {
        "action": (GObject.SignalFlags.RUN_FIRST, None, (str,)),
    }

    def __init__(self, state: AppState):
        super().__init__(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self.add_css_class("toolbar")
        self.state = state

        self._path_label = Gtk.Label(xalign=0)
        self._path_label.add_css_class("accent-text")
        self._path_label.set_ellipsize(3)
        self.append(self._path_label)

        spacer = Gtk.Box(hexpand=True)
        self.append(spacer)

        self._info_label = Gtk.Label()
        self._info_label.add_css_class("status-text")
        self.append(self._info_label)

        self._actions_box = Gtk.Box(spacing=4)
        for action_name, icon, tooltip in [
            ("rotate", "object-rotate-right-symbolic", "Rotate (Ctrl+R)"),
            ("copy", "edit-copy-symbolic", "Copy to clipboard (Ctrl+C)"),
            ("trash", "user-trash-symbolic", "Move to trash (Delete)"),
            ("info", "dialog-information-symbolic", "Image info (Ctrl+I)"),
        ]:
            btn = Gtk.Button(icon_name=icon, tooltip_text=tooltip)
            btn.add_css_class("flat")
            btn.connect("clicked", lambda b, a=action_name: self.emit("action", a))
            self._actions_box.append(btn)
        self.append(self._actions_box)

    def update_grid_mode(self):
        folder = self.state.current_folder
        count = len(self.state.files)
        self._path_label.set_text(str(folder) if folder else "")
        self._info_label.set_text(f"{count} images")
        self._actions_box.set_visible(False)

    def update_single_mode(self):
        path = self.state.current_file
        if path is None:
            return
        self._path_label.set_text(path.name)
        pos = self.state.index + 1
        total = len(self.state.files)
        self._info_label.set_text(f"{pos} / {total}")
        self._actions_box.set_visible(True)
