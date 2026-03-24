"""Thumbnail generation with disk cache."""

import hashlib
import sys
from pathlib import Path

from PIL import Image

from .image_loader import load_image

THUMBNAIL_SIZE = 256


def _cache_key(path: Path) -> str:
    """Generate a cache key from absolute path + mtime."""
    path = Path(path).resolve()
    mtime = path.stat().st_mtime
    raw = f"{path}:{mtime}"
    return hashlib.sha256(raw.encode()).hexdigest()


class ThumbnailCache:
    def __init__(self, cache_dir: Path | None = None):
        if cache_dir is None:
            cache_dir = Path.home() / ".cache" / "omarchy-imageview"
        self.cache_dir = Path(cache_dir)
        self.cache_dir.mkdir(parents=True, exist_ok=True)

    def get_thumbnail(self, path: Path) -> Image.Image | None:
        """Get a thumbnail for the given image path. Uses cache if available."""
        path = Path(path)
        if not path.is_file():
            return None

        try:
            key = _cache_key(path)
        except OSError:
            return None

        cached = self.cache_dir / f"{key}.png"
        if cached.exists():
            try:
                return Image.open(cached)
            except Exception:
                cached.unlink(missing_ok=True)

        return self._generate(path, cached)

    def _generate(self, path: Path, cache_path: Path) -> Image.Image | None:
        """Generate thumbnail, save to cache, return it."""
        img = load_image(path)
        if img is None:
            return None

        try:
            img.thumbnail((THUMBNAIL_SIZE, THUMBNAIL_SIZE), Image.LANCZOS)
            img.save(cache_path, "PNG")
            return img
        except Exception as e:
            print(f"Thumbnail generation failed for {path}: {e}", file=sys.stderr)
            return None
