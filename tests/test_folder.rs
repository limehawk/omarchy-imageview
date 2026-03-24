use std::path::PathBuf;
use image::{DynamicImage, RgbImage};
use omarchy_imageview::core::folder::scan_folder;

fn save_img(dir: &std::path::Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    DynamicImage::ImageRgb8(RgbImage::new(10, 10)).save(&p).unwrap();
    p
}

#[test]
fn test_scan_finds_images() {
    let dir = tempfile::tempdir().unwrap();
    for i in 1..=5 { save_img(dir.path(), &format!("photo{i}.jpg")); }
    let files = scan_folder(dir.path());
    assert_eq!(files.len(), 5);
}

#[test]
fn test_natural_sort() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["photo10.jpg", "photo1.jpg", "photo2.jpg", "photo20.jpg"] {
        save_img(dir.path(), name);
    }
    let files = scan_folder(dir.path());
    let names: Vec<_> = files.iter().map(|f| f.file_name().unwrap().to_string_lossy().to_string()).collect();
    assert_eq!(names, vec!["photo1.jpg", "photo2.jpg", "photo10.jpg", "photo20.jpg"]);
}

#[test]
fn test_ignores_unsupported() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("readme.txt"), "hello").unwrap();
    save_img(dir.path(), "photo.jpg");
    let files = scan_folder(dir.path());
    assert_eq!(files.len(), 1);
}

#[test]
fn test_empty_folder() {
    let dir = tempfile::tempdir().unwrap();
    assert!(scan_folder(dir.path()).is_empty());
}
