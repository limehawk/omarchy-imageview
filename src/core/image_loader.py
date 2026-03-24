"""Image decoding — routes files to the correct decoder, returns Pillow Image."""

import sys
from pathlib import Path

from PIL import Image

from .formats import FormatGroup, detect_format

try:
    from pillow_heif import register_heif_opener
    register_heif_opener()
except ImportError:
    print("Warning: pillow-heif not installed, HEIC/AVIF support disabled", file=sys.stderr)

try:
    import rawpy
    _HAS_RAWPY = True
except ImportError:
    _HAS_RAWPY = False
    print("Warning: rawpy not installed, RAW format support disabled", file=sys.stderr)


def load_image(path: Path) -> Image.Image | None:
    """Load an image file and return a Pillow Image, or None on failure."""
    path = Path(path)
    if not path.is_file():
        return None

    fmt = detect_format(path.name)
    if fmt is None:
        return None

    try:
        if fmt == FormatGroup.RAW:
            return _load_raw(path)
        elif fmt == FormatGroup.SVG:
            return None  # SVGs handled by Gdk.Texture, not Pillow
        else:
            return _load_pillow(path)
    except Exception as e:
        print(f"Error loading {path}: {e}", file=sys.stderr)
        return None


def _load_pillow(path: Path) -> Image.Image:
    """Load via Pillow (JPEG, PNG, WebP, GIF, BMP, TIFF, HEIC, AVIF)."""
    img = Image.open(path)
    img.load()
    return img


def _load_raw(path: Path) -> Image.Image | None:
    """Load camera RAW via rawpy → numpy → Pillow."""
    if not _HAS_RAWPY:
        return None
    raw = rawpy.imread(str(path))
    rgb = raw.postprocess()
    return Image.fromarray(rgb)
