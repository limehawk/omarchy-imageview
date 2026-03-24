"""File operations — trash, rename, move, copy, rotate."""

import shutil
import subprocess
import sys
from pathlib import Path

from PIL import Image


def trash_file(path: Path) -> bool:
    try:
        result = subprocess.run(["gio", "trash", str(path)], capture_output=True, text=True)
        return result.returncode == 0
    except FileNotFoundError:
        print("gio not found, cannot trash file", file=sys.stderr)
        return False


def rotate_image(path: Path, degrees: int) -> bool:
    try:
        img = Image.open(path)
        rotated = img.rotate(-degrees, expand=True)
        rotated.save(path, quality=95)
        return True
    except Exception as e:
        print(f"Rotate failed for {path}: {e}", file=sys.stderr)
        return False


def rename_file(path: Path, new_name: str) -> Path | None:
    path = Path(path)
    new_path = path.parent / new_name
    if new_path.exists():
        return None
    try:
        path.rename(new_path)
        return new_path
    except OSError as e:
        print(f"Rename failed: {e}", file=sys.stderr)
        return None


def copy_file(path: Path, dest_dir: Path) -> Path | None:
    try:
        dest = Path(dest_dir) / path.name
        shutil.copy2(str(path), str(dest))
        return dest
    except OSError as e:
        print(f"Copy failed: {e}", file=sys.stderr)
        return None


def move_file(path: Path, dest_dir: Path) -> Path | None:
    try:
        dest = Path(dest_dir) / path.name
        shutil.move(str(path), str(dest))
        return dest
    except OSError as e:
        print(f"Move failed: {e}", file=sys.stderr)
        return None
