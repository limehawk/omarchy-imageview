# omarchy-imageview

A GTK4 image viewer for [Omarchy](https://omarchy.org) (Hyprland / Wayland).

Open a folder, browse the grid, click into a picture. It follows the current Omarchy theme, talks to the trash instead of deleting files, and can set the wallpaper through `omarchy`.

## Install

### Arch

```bash
yay -S omarchy-imageview
```

### From source

Needs `gtk4`, `librsvg`, `libheif`, and a Rust toolchain.

```bash
git clone https://github.com/limehawk/omarchy-imageview.git
cd omarchy-imageview
make PREFIX=$HOME/.local install-as-default
```

That puts the binary in `~/.local/bin`, writes a desktop file with an absolute `Exec` (file managers launched by systemd do not see `~/.local/bin`), and sets it as the default handler for the formats it can open.

```bash
make PREFIX=$HOME/.local uninstall
```

`perl-image-exiftool` is optional. Without it, JPEG/TIFF rotation is written as pixels instead of a lossless orientation tag.

## Use

```bash
omarchy-imageview
omarchy-imageview ~/pictures/shot.jpg
omarchy-imageview ~/pictures
```

Grid for the folder. Single view for one image, with a filmstrip along the bottom.

| Key | Action |
|-----|--------|
| click | Open from grid |
| Esc / g | Back to grid |
| ← → · Space · Backspace | Previous / next |
| Home / End | First / last |
| + − · 0 · 1 | Zoom in / out / fit / actual size |
| middle-click · double-click | Toggle fit / actual size |
| drag | Pan when zoomed |
| Ctrl+R / Ctrl+Shift+R | Rotate 90° / 270° (display only) |
| Ctrl+S | Write the rotation to the file |
| Ctrl+C / Ctrl+Shift+C | Copy image / copy path |
| Ctrl+W | Set as wallpaper |
| Ctrl+E | Open in Pinta |
| Ctrl+I | EXIF / info panel |
| F2 | Rename |
| Ctrl+M / Ctrl+Shift+M | Move / copy to a folder |
| Delete | Trash |
| F11 / f | Fullscreen |
| q | Quit |

Sort lives in the toolbar: a **Sort** menu with Date or Name.

## Formats

JPEG, PNG, WebP, GIF, BMP, TIFF, AVIF, HEIC/HEIF, JPEG XL, SVG, ICO, TGA, QOI, HDR, DDS, PNM, Farbfeld, EXR.

RAW is not supported.

## Build

```bash
cargo test
cargo build --release
```

The binary is `target/release/omarchy-imageview`.
