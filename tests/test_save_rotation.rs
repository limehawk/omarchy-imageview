use image::{DynamicImage, Rgba, RgbaImage};
use omarchy_imageview::actions::file_ops::{editor_commands, save_rotation};
use omarchy_imageview::core::image_loader::{
    compose_orientation_cw, load_image, read_exif_orientation,
};

#[test]
fn compose_90_steps_around_the_identity() {
    assert_eq!(compose_orientation_cw(1, 90), 6);
    assert_eq!(compose_orientation_cw(6, 90), 3);
    assert_eq!(compose_orientation_cw(3, 90), 8);
    assert_eq!(compose_orientation_cw(8, 90), 1);
}

#[test]
fn compose_180_and_270() {
    assert_eq!(compose_orientation_cw(1, 180), 3);
    assert_eq!(compose_orientation_cw(1, 270), 8);
    assert_eq!(compose_orientation_cw(6, 270), 1);
}

#[test]
fn compose_missing_tag_treated_as_normal() {
    assert_eq!(compose_orientation_cw(0, 90), 6);
    assert_eq!(compose_orientation_cw(99, 90), 6);
}

fn sample() -> DynamicImage {
    let mut img = RgbaImage::new(2, 1);
    img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
    img.put_pixel(1, 0, Rgba([0, 0, 255, 255]));
    DynamicImage::ImageRgba8(img)
}

#[test]
fn save_png_rewrites_pixels() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shot.png");
    sample().save(&path).unwrap();

    assert!(save_rotation(&path, 90));
    let loaded = load_image(&path).unwrap();
    assert_eq!(loaded.width(), 1);
    assert_eq!(loaded.height(), 2);
    assert_eq!(loaded.to_rgba8().get_pixel(0, 0).0, [255, 0, 0, 255]);
    assert_eq!(loaded.to_rgba8().get_pixel(0, 1).0, [0, 0, 255, 255]);
}

#[test]
fn save_jpeg_sets_exif_orientation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("photo.jpg");
    sample().save(&path).unwrap();

    assert!(save_rotation(&path, 90));
    assert_eq!(read_exif_orientation(&path), Some(6));

    // Loader applies the tag, so the image comes back already rotated.
    let loaded = load_image(&path).unwrap();
    assert_eq!(loaded.width(), 1);
    assert_eq!(loaded.height(), 2);
}

#[test]
fn save_zero_degrees_is_noop() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shot.png");
    sample().save(&path).unwrap();
    assert!(save_rotation(&path, 0));
    let loaded = load_image(&path).unwrap();
    assert_eq!(loaded.width(), 2);
}

#[test]
fn editor_prefers_tensaku() {
    assert_eq!(editor_commands()[0], "tensaku-edit");
    assert_eq!(*editor_commands().last().unwrap(), "pinta");
}
