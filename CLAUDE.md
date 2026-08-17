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

## Release
Source of truth is Forgejo (`limehawk/omarchy-imageview`). GitHub is a push mirror.

```bash
# 1. Tag + push (FJ origin mirrors to GitHub)
git tag -a v0.X.X -m "omarchy-imageview 0.X.X"
git push && git push --tags

# 2. Attach the amd64 binary
cargo build --release --locked
cp target/release/omarchy-imageview /tmp/omarchy-imageview-linux-amd64
gh release create v0.X.X /tmp/omarchy-imageview-linux-amd64
fj release create v0.X.X -t v0.X.X -a /tmp/omarchy-imageview-linux-amd64

# 3. AUR (tarball PKGBUILD lives only on the AUR repo)
curl -sL "https://github.com/limehawk/omarchy-imageview/archive/v0.X.X.tar.gz" | sha256sum
git clone ssh://aur@aur.archlinux.org/omarchy-imageview.git /tmp/omarchy-imageview-aur
# bump pkgver + sha256sums, then:
cd /tmp/omarchy-imageview-aur
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update to 0.X.X"
git push
```
