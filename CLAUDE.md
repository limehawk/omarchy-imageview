# omarchy-imageview

Python + GTK4 image viewer/browser for the Omarchy desktop (Hyprland/Wayland).

## Stack
- Python 3.11+
- GTK4 via PyGObject (`gi.repository`)
- Pillow, pillow-heif, rawpy for image decoding

## Architecture
Three layers: core (no UI), state (data model), ui (GTK4 widgets).
Core decodes images and generates thumbnails. State is the single source of truth.
UI reads from state and dispatches actions.

## Running
```bash
python -m src.main [path-to-image]
```

## Testing
```bash
pytest tests/ -v
```

## Conventions
- Use `gio trash` for all file deletion (never `rm`)
- Background image decoding via `concurrent.futures.ThreadPoolExecutor`
- Bridge to GTK main thread via `GLib.idle_add()`
- Use `bun` over npm if JS tooling is ever needed
- No secrets in git
