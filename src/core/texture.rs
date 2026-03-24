use gdk4 as gdk;
use glib;
use image::DynamicImage;
use std::path::Path;

/// Convert a DynamicImage to a gdk::MemoryTexture for GTK4 display.
pub fn image_to_texture(img: &DynamicImage) -> gdk::MemoryTexture {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let bytes = glib::Bytes::from_owned(rgba.into_raw());
    let stride = width as usize * 4;
    gdk::MemoryTexture::new(
        width as i32,
        height as i32,
        gdk::MemoryFormat::R8g8b8a8,
        &bytes,
        stride,
    )
}

/// Load an SVG via GDK (uses librsvg under the hood). Returns a Texture.
pub fn svg_to_texture(path: &Path) -> Option<gdk::Texture> {
    gdk::Texture::from_filename(path).ok()
}
