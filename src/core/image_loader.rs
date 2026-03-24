use std::path::Path;
use image::DynamicImage;
use super::formats::{detect_format, FormatGroup};

/// Load an image file, returning None on failure or unsupported format.
/// SVGs are NOT handled here — they go through gdk4::Texture::from_filename.
pub fn load_image(path: &Path) -> Option<DynamicImage> {
    if !path.is_file() {
        return None;
    }

    let filename = path.file_name()?.to_string_lossy();
    let fmt = detect_format(&filename)?;

    match fmt {
        FormatGroup::Svg => None,
        FormatGroup::Image => image::open(path).ok(),
    }
}
