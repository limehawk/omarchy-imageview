"""Clipboard operations — copy image and copy file path."""

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gdk


def copy_texture_to_clipboard(texture: Gdk.Texture):
    """Copy a GdkTexture (image) to the system clipboard."""
    clipboard = Gdk.Display.get_default().get_clipboard()
    clipboard.set_texture(texture)


def copy_text_to_clipboard(text: str):
    """Copy text to the system clipboard."""
    clipboard = Gdk.Display.get_default().get_clipboard()
    clipboard.set(text)
