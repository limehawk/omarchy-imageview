use gdk4 as gdk;
use std::path::Path;

/// Load an SVG via GDK (uses librsvg under the hood). Returns a Texture.
pub fn svg_to_texture(path: &Path) -> Option<gdk::Texture> {
    gdk::Texture::from_filename(path).ok()
}
