use image::{DynamicImage, Rgba, RgbaImage};
use omarchy_imageview::core::image_loader::apply_orientation;

fn sample() -> DynamicImage {
    // 2x1: left red, right blue
    let mut img = RgbaImage::new(2, 1);
    img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
    img.put_pixel(1, 0, Rgba([0, 0, 255, 255]));
    DynamicImage::ImageRgba8(img)
}

fn px(img: &DynamicImage, x: u32, y: u32) -> [u8; 4] {
    img.to_rgba8().get_pixel(x, y).0
}

#[test]
fn orientation_1_unchanged() {
    let img = apply_orientation(sample(), 1);
    assert_eq!(img.width(), 2);
    assert_eq!(img.height(), 1);
    assert_eq!(px(&img, 0, 0), [255, 0, 0, 255]);
}

#[test]
fn orientation_3_rotate_180() {
    let img = apply_orientation(sample(), 3);
    assert_eq!(px(&img, 0, 0), [0, 0, 255, 255]);
    assert_eq!(px(&img, 1, 0), [255, 0, 0, 255]);
}

#[test]
fn orientation_6_rotate_90_cw() {
    let img = apply_orientation(sample(), 6);
    assert_eq!(img.width(), 1);
    assert_eq!(img.height(), 2);
    assert_eq!(px(&img, 0, 0), [255, 0, 0, 255]);
    assert_eq!(px(&img, 0, 1), [0, 0, 255, 255]);
}

#[test]
fn orientation_8_rotate_270_cw() {
    let img = apply_orientation(sample(), 8);
    assert_eq!(img.width(), 1);
    assert_eq!(img.height(), 2);
    assert_eq!(px(&img, 0, 0), [0, 0, 255, 255]);
    assert_eq!(px(&img, 0, 1), [255, 0, 0, 255]);
}

#[test]
fn orientation_unknown_passthrough() {
    let img = apply_orientation(sample(), 99);
    assert_eq!(img.width(), 2);
    assert_eq!(px(&img, 0, 0), [255, 0, 0, 255]);
}
