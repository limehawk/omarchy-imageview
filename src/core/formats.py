"""Image format detection and supported extension registry."""

from enum import Enum, auto
from pathlib import Path


class FormatGroup(Enum):
    PILLOW = auto()  # JPEG, PNG, WebP, GIF, BMP, TIFF
    HEIF = auto()    # HEIC, HEIF, AVIF (via pillow-heif)
    RAW = auto()     # CR2, NEF, ARW, DNG, etc. (via rawpy)
    SVG = auto()     # SVG (via librsvg / Gdk.Texture)


_FORMAT_MAP: dict[str, FormatGroup] = {}

_PILLOW_EXTS = {".jpg", ".jpeg", ".png", ".webp", ".gif", ".bmp", ".tif", ".tiff"}
_HEIF_EXTS = {".heic", ".heif", ".avif"}
_RAW_EXTS = {".cr2", ".cr3", ".nef", ".nrf", ".arw", ".dng", ".orf", ".raf", ".rw2"}
_SVG_EXTS = {".svg"}

for ext in _PILLOW_EXTS:
    _FORMAT_MAP[ext] = FormatGroup.PILLOW
for ext in _HEIF_EXTS:
    _FORMAT_MAP[ext] = FormatGroup.HEIF
for ext in _RAW_EXTS:
    _FORMAT_MAP[ext] = FormatGroup.RAW
for ext in _SVG_EXTS:
    _FORMAT_MAP[ext] = FormatGroup.SVG

SUPPORTED_EXTENSIONS: frozenset[str] = frozenset(_FORMAT_MAP.keys())


def is_supported(filename: str) -> bool:
    """Check if a filename has a supported image extension."""
    return Path(filename).suffix.lower() in SUPPORTED_EXTENSIONS


def detect_format(filename: str) -> FormatGroup | None:
    """Detect the format group for a filename. Returns None if unsupported."""
    return _FORMAT_MAP.get(Path(filename).suffix.lower())
