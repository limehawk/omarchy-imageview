use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatGroup {
    /// Raster formats we can decode to a bitmap
    Image,
    /// SVG — loaded via gdk4::Texture::from_filename (librsvg)
    Svg,
}

const RASTER_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "jpe", "jfif",
    "png", "webp", "gif", "bmp",
    "tif", "tiff",
    "avif", "heic", "heif",
    "jxl",
    "ico", "tga", "qoi", "hdr", "dds",
    "pbm", "pgm", "ppm", "pnm",
    "ff", "exr",
];

fn extension(filename: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default()
}

/// Check if a filename has a supported image extension.
pub fn is_supported(filename: &str) -> bool {
    detect_format(filename).is_some()
}

/// Detect the format group for a filename. Returns None if unsupported.
pub fn detect_format(filename: &str) -> Option<FormatGroup> {
    let ext = extension(filename);
    if ext == "svg" {
        return Some(FormatGroup::Svg);
    }
    if RASTER_EXTENSIONS.contains(&ext.as_str()) {
        return Some(FormatGroup::Image);
    }
    None
}
