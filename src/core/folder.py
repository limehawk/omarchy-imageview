"""Folder scanning with natural sort order."""

from pathlib import Path

from natsort import natsorted

from .formats import is_supported


def scan_folder(directory: Path) -> list[Path]:
    """Scan a directory for supported image files, naturally sorted."""
    directory = Path(directory)
    if not directory.is_dir():
        return []

    files = [f for f in directory.iterdir() if f.is_file() and is_supported(f.name)]
    return natsorted(files, key=lambda f: f.name.lower())
