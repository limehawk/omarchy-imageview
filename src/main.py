"""Omarchy Image Viewer — entry point."""

import sys
from pathlib import Path

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gio, Gtk

from .state.app_state import AppState
from .ui.window import ImageViewerWindow


class ImageViewerApp(Gtk.Application):
    def __init__(self):
        super().__init__(
            application_id="com.omarchy.imageview",
            flags=Gio.ApplicationFlags.HANDLES_OPEN,
        )
        self.state = AppState()
        self.win = None

    def do_activate(self):
        """Launched with no file argument — grid view of last-used folder."""
        if self.win is None:
            self.win = ImageViewerWindow(self, self.state)

        self.state.restore()
        folder = self.state.last_folder or Path.home()
        if folder.is_dir():
            self.state.load_folder(folder)
        self.win.show_grid()
        self.win.present()

    def do_open(self, files, n_files, hint):
        """Launched with a file argument — single view with filmstrip."""
        if self.win is None:
            self.win = ImageViewerWindow(self, self.state)

        path = Path(files[0].get_path())
        if path.is_file():
            self.state.load_folder(path.parent, target_file=path)
            self.state.view_mode = "single"
            self.win.show_single()
        else:
            self.state.load_folder(path)
            self.win.show_grid()
        self.win.present()


def main():
    app = ImageViewerApp()
    app.run(sys.argv)


if __name__ == "__main__":
    main()
