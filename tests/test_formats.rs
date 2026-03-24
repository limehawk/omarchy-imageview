use omarchy_imageview::core::formats::{detect_format, is_supported, FormatGroup};

#[test]
fn test_jpeg_supported() {
    assert!(is_supported("photo.jpg"));
    assert!(is_supported("photo.JPEG"));
    assert!(is_supported("photo.JPG"));
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
fn test_raw_supported() {
    assert!(is_supported("photo.cr2"));
    assert!(is_supported("photo.NEF"));
    assert!(is_supported("photo.arw"));
    assert!(is_supported("photo.dng"));
}

#[test]
fn test_heic_supported() {
    assert!(is_supported("photo.heic"));
    assert!(is_supported("photo.HEIF"));
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
    assert_eq!(detect_format("photo.cr2"), Some(FormatGroup::Image));
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
