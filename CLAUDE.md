# omarchy-imageview

Rust + GTK4 image viewer/browser for the Omarchy desktop (Hyprland/Wayland).

## Stack
- Rust (edition 2021)
- GTK4 via gtk4-rs
- image crate for decoding
- kamadak-exif for EXIF metadata

## Architecture
Three layers: core (no UI), state (data model), ui (GTK4 widgets).
Core decodes images and generates thumbnails. State uses Rc<RefCell<AppState>>.
UI uses gtk4-rs with GObject subclasses for list items.

## Running
```bash
cargo run -- [path-to-image]
```

## Testing
```bash
cargo test
```

## Building release
```bash
cargo build --release
# Binary at target/release/omarchy-imageview
```

## Conventions
- Use gio::File::trash() for all file deletion (never rm)
- Async image decoding via std::thread::spawn + glib::MainContext::default().invoke()
- No secrets in git
