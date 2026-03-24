"""Image info panel — EXIF data and file metadata."""

from pathlib import Path

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gtk

from PIL import Image
from PIL.ExifTags import TAGS


class InfoPanel(Gtk.ScrolledWindow):
    def __init__(self):
        super().__init__()
        self.add_css_class("info-panel")
        self.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        self.set_size_request(280, -1)

        self._box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.set_child(self._box)

    def update(self, path: Path | None):
        while child := self._box.get_first_child():
            self._box.remove(child)

        if path is None:
            return

        path = Path(path)

        self._add_section("File")
        self._add_row("Name", path.name)
        self._add_row("Path", str(path.parent))
        try:
            size = path.stat().st_size
            self._add_row("Size", self._format_size(size))
        except OSError:
            pass

        try:
            img = Image.open(path)
            self._add_section("Image")
            self._add_row("Dimensions", f"{img.width} x {img.height}")
            self._add_row("Format", img.format or "Unknown")
            self._add_row("Mode", img.mode)

            exif_data = self._extract_exif(img)
            if exif_data:
                self._add_section("EXIF")
                for key, value in exif_data.items():
                    self._add_row(key, str(value))
            img.close()
        except Exception:
            pass

    def _add_section(self, title: str):
        label = Gtk.Label(label=title, xalign=0)
        label.set_markup(f"<b>{title}</b>")
        label.set_margin_top(8)
        self._box.append(label)
        sep = Gtk.Separator()
        self._box.append(sep)

    def _add_row(self, key: str, value: str):
        row = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        key_label = Gtk.Label(label=key, xalign=0)
        key_label.add_css_class("info-label")
        val_label = Gtk.Label(label=value, xalign=0, wrap=True)
        val_label.add_css_class("info-value")
        val_label.set_selectable(True)
        row.append(key_label)
        row.append(val_label)
        self._box.append(row)

    def _extract_exif(self, img: Image.Image) -> dict[str, str]:
        result = {}
        try:
            raw = img._getexif()
            if not raw:
                return result
        except AttributeError:
            return result

        interesting = {
            "Make": "Camera",
            "Model": "Model",
            "LensModel": "Lens",
            "ISOSpeedRatings": "ISO",
            "ExposureTime": "Shutter",
            "FNumber": "Aperture",
            "FocalLength": "Focal Length",
            "DateTimeOriginal": "Date Taken",
        }

        decoded = {}
        for tag_id, val in raw.items():
            tag_name = TAGS.get(tag_id, tag_id)
            decoded[tag_name] = val

        for exif_key, display_name in interesting.items():
            if exif_key in decoded:
                val = decoded[exif_key]
                if exif_key == "ExposureTime" and hasattr(val, "numerator"):
                    val = f"{val.numerator}/{val.denominator}s"
                elif exif_key == "FNumber" and hasattr(val, "numerator"):
                    val = f"f/{float(val):.1f}"
                elif exif_key == "FocalLength" and hasattr(val, "numerator"):
                    val = f"{float(val):.0f}mm"
                result[display_name] = str(val)

        return result

    @staticmethod
    def _format_size(size: int) -> str:
        for unit in ("B", "KB", "MB", "GB"):
            if size < 1024:
                return f"{size:.1f} {unit}" if unit != "B" else f"{size} {unit}"
            size /= 1024
        return f"{size:.1f} TB"
