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


class FolderMonitor:
    """Watch a directory for changes."""

    def __init__(self, directory: Path, on_changed: callable):
        import gi
        gi.require_version("Gtk", "4.0")
        from gi.repository import Gio as _Gio
        self._Gio = _Gio
        self._monitor = None
        self._callback = on_changed
        self.watch(directory)

    def watch(self, directory: Path):
        self.stop()
        gfile = self._Gio.File.new_for_path(str(directory))
        self._monitor = gfile.monitor_directory(self._Gio.FileMonitorFlags.NONE, None)
        self._monitor.connect("changed", self._on_changed)

    def stop(self):
        if self._monitor:
            self._monitor.cancel()
            self._monitor = None

    def _on_changed(self, monitor, file, other_file, event_type):
        path = Path(file.get_path())
        if not is_supported(path.name):
            return

        if event_type in (
            self._Gio.FileMonitorEvent.CREATED,
            self._Gio.FileMonitorEvent.DELETED,
            self._Gio.FileMonitorEvent.MOVED_IN,
            self._Gio.FileMonitorEvent.MOVED_OUT,
        ):
            self._callback()
