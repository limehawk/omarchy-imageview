"""Main application window with view switching."""

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gtk, Gdk

from ..state.app_state import AppState
from .theme import load_theme
from .grid_view import GridView
from .single_view import SingleView
from .toolbar import Toolbar
from .info_panel import InfoPanel
from ..actions.file_ops import trash_file, rotate_image


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

        self._info_panel = InfoPanel()
        self._info_panel.set_visible(False)

        # Content area: stack + info panel side by side
        self._content_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        self._content_box.append(self._stack)
        self._content_box.append(self._info_panel)

        self._toolbar = Toolbar(state)
        self._toolbar.connect("action", self._on_toolbar_action)
        self._main_box.append(self._toolbar)
        self._main_box.append(self._content_box)
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
        self._toolbar.update_grid_mode()

    def show_single(self):
        self.state.view_mode = "single"
        self._stack.set_visible_child_name("single")
        self._single_view.load()
        self._toolbar.update_single_mode()

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
                self._toolbar.set_visible(True)
            else:
                self.fullscreen()
                self._single_view.set_filmstrip_visible(False)
                self._toolbar.set_visible(False)
            return True

        # File operations (single view)
        if self.state.view_mode == "single":
            if key == "Delete":
                self._do_trash()
                return True
            if ctrl and key == "r":
                if shift:
                    self._do_rotate(270)
                else:
                    self._do_rotate(90)
                return True
            if ctrl and shift and key == "X":
                self._do_trash()
                return True

        if ctrl and key == "i":
            visible = not self._info_panel.get_visible()
            self._info_panel.set_visible(visible)
            if visible:
                self._info_panel.update(self.state.current_file)
            return True

        if self.state.view_mode == "single":
            if key in ("Left", "Up"):
                self.state.navigate_prev()
                self._single_view.refresh_image()
                self._toolbar.update_single_mode()
                if self._info_panel.get_visible():
                    self._info_panel.update(self.state.current_file)
                return True
            if key in ("Right", "Down"):
                self.state.navigate_next()
                self._single_view.refresh_image()
                self._toolbar.update_single_mode()
                if self._info_panel.get_visible():
                    self._info_panel.update(self.state.current_file)
                return True
            if key == "Home":
                self.state.navigate_to(0)
                self._single_view.refresh_image()
                self._toolbar.update_single_mode()
                if self._info_panel.get_visible():
                    self._info_panel.update(self.state.current_file)
                return True
            if key == "End":
                self.state.navigate_to(len(self.state.files) - 1)
                self._single_view.refresh_image()
                self._toolbar.update_single_mode()
                if self._info_panel.get_visible():
                    self._info_panel.update(self.state.current_file)
                return True

        return False

    def _do_trash(self):
        path = self.state.current_file
        if path and trash_file(path):
            self.state.remove_file(path)
            self._single_view.refresh_image()
            self._toolbar.update_single_mode()

    def _do_rotate(self, degrees):
        path = self.state.current_file
        if path:
            rotate_image(path, degrees)
            self._single_view.refresh_image()

    def _on_toolbar_action(self, toolbar, action):
        if action == "trash":
            self._do_trash()
        elif action == "rotate":
            self._do_rotate(90)
        elif action == "info":
            visible = not self._info_panel.get_visible()
            self._info_panel.set_visible(visible)
            if visible:
                self._info_panel.update(self.state.current_file)
        elif action == "copy":
            pass  # Clipboard wired in Batch 7
