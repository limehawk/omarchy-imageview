"""Set image as Omarchy wallpaper via swaybg."""

import shutil
import subprocess
import sys
from pathlib import Path

_BG_DIR = Path.home() / ".config" / "omarchy" / "current" / "theme" / "backgrounds"
_BG_LINK = Path.home() / ".config" / "omarchy" / "current" / "background"


def set_wallpaper(image_path: Path) -> bool:
    image_path = Path(image_path)

    try:
        _BG_DIR.mkdir(parents=True, exist_ok=True)
        dest = _BG_DIR / image_path.name
        if dest != image_path:
            shutil.copy2(str(image_path), str(dest))

        if _BG_LINK.is_symlink() or _BG_LINK.exists():
            _BG_LINK.unlink()
        _BG_LINK.symlink_to(dest)

        subprocess.run(["pkill", "swaybg"], capture_output=True)
        subprocess.Popen(
            ["swaybg", "-i", str(_BG_LINK), "-m", "fill"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        return True
    except Exception as e:
        print(f"Failed to set wallpaper: {e}", file=sys.stderr)
        return False
