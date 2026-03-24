"""Application state — single source of truth."""

import tomllib
from pathlib import Path

from ..core.folder import scan_folder

_STATE_FILE = "state.toml"


class AppState:
    def __init__(self, config_dir: Path | None = None):
        if config_dir is None:
            config_dir = Path.home() / ".config" / "omarchy-imageview"
        self.config_dir = Path(config_dir)

        self.current_folder: Path | None = None
        self.files: list[Path] = []
        self.index: int = 0
        self.view_mode: str = "grid"
        self.zoom_fit: bool = True
        self.zoom: float = 1.0
        self.pan_x: float = 0.0
        self.pan_y: float = 0.0
        self.last_folder: Path | None = None

    @property
    def current_file(self) -> Path | None:
        if not self.files or self.index >= len(self.files):
            return None
        return self.files[self.index]

    def load_folder(self, folder: Path, target_file: Path | None = None):
        self.current_folder = Path(folder)
        self.files = scan_folder(folder)
        self.last_folder = self.current_folder

        if target_file and target_file in self.files:
            self.index = self.files.index(target_file)
        else:
            self.index = 0

    def navigate_next(self):
        if self.files and self.index < len(self.files) - 1:
            self.index += 1

    def navigate_prev(self):
        if self.index > 0:
            self.index -= 1

    def navigate_to(self, index: int):
        if self.files:
            self.index = max(0, min(index, len(self.files) - 1))

    def remove_file(self, path: Path):
        if path in self.files:
            was_index = self.files.index(path)
            self.files.remove(path)
            if self.index >= len(self.files):
                self.index = max(0, len(self.files) - 1)
            elif self.index > was_index:
                self.index -= 1

    def save(self):
        self.config_dir.mkdir(parents=True, exist_ok=True)
        state_path = self.config_dir / _STATE_FILE
        lines = []
        if self.last_folder:
            lines.append(f'last_folder = "{self.last_folder}"')
        state_path.write_text("\n".join(lines) + "\n")

    def restore(self):
        state_path = self.config_dir / _STATE_FILE
        if not state_path.exists():
            return
        try:
            data = tomllib.loads(state_path.read_text())
            lf = data.get("last_folder")
            if lf:
                self.last_folder = Path(lf)
        except Exception:
            pass
