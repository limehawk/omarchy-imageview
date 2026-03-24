"""Main application window with view switching."""

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gtk, Gdk

from ..state.app_state import AppState
from .theme import load_theme
from .grid_view import GridView
from .single_view import SingleView


class ImageViewerWindow(Gtk.ApplicationWindow):
    def __init__(self, app, state: AppState):
        super().__init__(application=app, title="Image Viewer")
        self.state = state
        self.set_default_size(1200, 800)

        self._apply_theme()

        # Main layout: will hold toolbar + content later
        self._main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)

        # View stack
        self._stack = Gtk.Stack()
        self._stack.set_transition_type(Gtk.StackTransitionType.CROSSFADE)
        self._stack.set_transition_duration(150)

        # Grid view
        self._grid_view = GridView(state)
        self._grid_view.connect("image-activated", self._on_image_activated)

        self._single_view = SingleView(state)

        self._stack.add_named(self._grid_view, "grid")
        self._stack.add_named(self._single_view, "single")

        self._main_box.append(self._stack)
        self.set_child(self._main_box)

        self._setup_keybindings()

    def _apply_theme(self):
        css = load_theme()
        provider = Gtk.CssProvider()
        provider.load_from_string(css)
        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(),
            provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION,
        )

    def show_grid(self):
        self.state.view_mode = "grid"
        self._stack.set_visible_child_name("grid")
        self._grid_view.load()

    def show_single(self):
        self.state.view_mode = "single"
        self._stack.set_visible_child_name("single")
        self._single_view.load()

    def _on_image_activated(self, grid_view, index):
        self.state.index = index
        self.show_single()

    def _setup_keybindings(self):
        controller = Gtk.EventControllerKey()
        controller.connect("key-pressed", self._on_key_pressed)
        self.add_controller(controller)

    def _on_key_pressed(self, controller, keyval, keycode, modifiers):
        key = Gdk.keyval_name(keyval)
        ctrl = modifiers & Gdk.ModifierType.CONTROL_MASK
        shift = modifiers & Gdk.ModifierType.SHIFT_MASK

        if key == "Escape":
            if self.state.view_mode == "single":
                self.show_grid()
                return True
            else:
                self.close()
                return True

        if key == "BackSpace" and self.state.view_mode == "single":
            self.show_grid()
            return True

        if key == "g" and not ctrl:
            self.show_grid()
            return True

        if key in ("F11", "f") and not ctrl:
            if self.is_fullscreen():
                self.unfullscreen()
                self._single_view.set_filmstrip_visible(True)
            else:
                self.fullscreen()
                self._single_view.set_filmstrip_visible(False)
            return True

        if self.state.view_mode == "single":
            if key in ("Left", "Up"):
                self.state.navigate_prev()
                self._single_view.refresh_image()
                return True
            if key in ("Right", "Down"):
                self.state.navigate_next()
                self._single_view.refresh_image()
                return True
            if key == "Home":
                self.state.navigate_to(0)
                self._single_view.refresh_image()
                return True
            if key == "End":
                self.state.navigate_to(len(self.state.files) - 1)
                self._single_view.refresh_image()
                return True

        return False
