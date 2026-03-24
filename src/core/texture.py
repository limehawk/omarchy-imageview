"""Convert Pillow Images to Gdk.MemoryTexture for GTK4 display."""

from pathlib import Path

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gdk, GLib

from PIL import Image


def image_to_texture(img: Image.Image) -> Gdk.Texture:
    """Convert a Pillow Image to a Gdk.MemoryTexture."""
    rgba = img.convert("RGBA")
    data = rgba.tobytes()
    gbytes = GLib.Bytes.new(data)
    width, height = rgba.size
    stride = width * 4  # 4 bytes per pixel (RGBA)
    return Gdk.MemoryTexture.new(width, height, Gdk.MemoryFormat.R8G8B8A8, gbytes, stride)


def svg_to_texture(path: Path) -> Gdk.Texture | None:
    """Load an SVG via librsvg (through GdkTexture) — Wayland-native path."""
    try:
        return Gdk.Texture.new_from_filename(str(path))
    except Exception:
        return None
