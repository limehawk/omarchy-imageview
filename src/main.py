"""Omarchy Image Viewer — entry point."""

import sys

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gio, Gtk


class ImageViewerApp(Gtk.Application):
    def __init__(self):
        super().__init__(
            application_id="com.omarchy.imageview",
            flags=Gio.ApplicationFlags.HANDLES_OPEN,
        )

    def do_activate(self):
        """Launched with no file argument — grid view."""
        win = Gtk.ApplicationWindow(application=self, title="Image Viewer")
        win.set_default_size(1200, 800)
        label = Gtk.Label(label="Grid view placeholder")
        win.set_child(label)
        win.present()

    def do_open(self, files, n_files, hint):
        """Launched with a file argument — single view."""
        self.do_activate()


def main():
    app = ImageViewerApp()
    app.run(sys.argv)


if __name__ == "__main__":
    main()
