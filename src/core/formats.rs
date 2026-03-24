use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatGroup {
    /// Raster formats handled by the `image` crate (JPEG, PNG, WebP, GIF, BMP, TIFF, AVIF)
    Image,
    /// SVG — loaded via gdk4::Texture::from_filename (librsvg)
    Svg,
}

/// All supported file extensions (lowercase, with leading dot).
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    ".jpg", ".jpeg", ".png", ".webp", ".gif", ".bmp", ".tif", ".tiff", ".avif",
    ".heic", ".heif",
    ".cr2", ".cr3", ".nef", ".nrf", ".arw", ".dng", ".orf", ".raf", ".rw2",
    ".svg",
];

/// Check if a filename has a supported image extension.
pub fn is_supported(filename: &str) -> bool {
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_ascii_lowercase()))
        .unwrap_or_default();
    SUPPORTED_EXTENSIONS.contains(&ext.as_str())
}

/// Detect the format group for a filename. Returns None if unsupported.
pub fn detect_format(filename: &str) -> Option<FormatGroup> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_ascii_lowercase()))
        .unwrap_or_default();

    match ext.as_str() {
        ".svg" => Some(FormatGroup::Svg),
        ".jpg" | ".jpeg" | ".png" | ".webp" | ".gif" | ".bmp" | ".tif" | ".tiff" | ".avif"
        | ".heic" | ".heif"
        | ".cr2" | ".cr3" | ".nef" | ".nrf" | ".arw" | ".dng" | ".orf" | ".raf" | ".rw2" => {
            Some(FormatGroup::Image)
        }
        _ => None,
    }
}
