use omarchy_imageview::core::formats::{detect_format, is_supported, FormatGroup};

#[test]
fn test_jpeg_supported() {
    assert!(is_supported("photo.jpg"));
    assert!(is_supported("photo.JPEG"));
    assert!(is_supported("photo.JPG"));
    assert!(is_supported("photo.jpe"));
    assert!(is_supported("photo.jfif"));
}

#[test]
fn test_png_supported() {
    assert!(is_supported("image.png"));
    assert!(is_supported("image.PNG"));
}

#[test]
fn test_webp_supported() {
    assert!(is_supported("photo.webp"));
}

#[test]
fn test_easy_raster_formats() {
    for name in [
        "icon.ico", "shot.tga", "pix.qoi", "env.hdr", "tex.dds",
        "a.pbm", "a.pgm", "a.ppm", "a.pnm", "a.ff", "scene.exr", "photo.jxl",
    ] {
        assert!(is_supported(name), "{name}");
        assert_eq!(detect_format(name), Some(FormatGroup::Image), "{name}");
    }
}

#[test]
fn test_raw_not_supported() {
    // No demosaic pipeline — don't claim camera RAW.
    for name in ["photo.cr2", "photo.NEF", "photo.arw", "photo.dng", "photo.orf", "photo.raf"] {
        assert!(!is_supported(name), "{name}");
        assert_eq!(detect_format(name), None, "{name}");
    }
}

#[test]
fn test_heic_supported() {
    assert!(is_supported("photo.heic"));
    assert!(is_supported("photo.HEIF"));
    assert_eq!(detect_format("photo.heic"), Some(FormatGroup::Image));
}

#[test]
fn test_svg_supported() {
    assert!(is_supported("icon.svg"));
}

#[test]
fn test_unsupported() {
    assert!(!is_supported("document.pdf"));
    assert!(!is_supported("video.mp4"));
    assert!(!is_supported("noext"));
}

#[test]
fn test_detect_format_image() {
    assert_eq!(detect_format("photo.jpg"), Some(FormatGroup::Image));
    assert_eq!(detect_format("photo.png"), Some(FormatGroup::Image));
    assert_eq!(detect_format("photo.webp"), Some(FormatGroup::Image));
    assert_eq!(detect_format("photo.avif"), Some(FormatGroup::Image));
    assert_eq!(detect_format("photo.heic"), Some(FormatGroup::Image));
    assert_eq!(detect_format("photo.jxl"), Some(FormatGroup::Image));
}

#[test]
fn test_detect_format_svg() {
    assert_eq!(detect_format("icon.svg"), Some(FormatGroup::Svg));
}

#[test]
fn test_detect_format_none() {
    assert_eq!(detect_format("doc.pdf"), None);
    assert_eq!(detect_format("noext"), None);
}
