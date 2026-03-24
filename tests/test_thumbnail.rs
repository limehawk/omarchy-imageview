use std::path::PathBuf;
use image::{DynamicImage, RgbImage};
use omarchy_imageview::core::thumbnail::{ThumbnailCache, cache_key};

fn create_test_jpeg(dir: &std::path::Path, name: &str, w: u32, h: u32) -> PathBuf {
    let path = dir.join(name);
    let img = DynamicImage::ImageRgb8(RgbImage::new(w, h));
    img.save(&path).unwrap();
    path
}

#[test]
fn test_cache_key_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    let path = create_test_jpeg(dir.path(), "test.jpg", 100, 80);
    let k1 = cache_key(&path).unwrap();
    let k2 = cache_key(&path).unwrap();
    assert_eq!(k1, k2);
    assert_eq!(k1.len(), 64);
}

#[test]
fn test_generate_thumbnail() {
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = tempfile::tempdir().unwrap();
    let path = create_test_jpeg(dir.path(), "test.jpg", 100, 80);
    let cache = ThumbnailCache::new(Some(cache_dir.path().to_path_buf()));
    let thumb = cache.get_thumbnail(&path).unwrap();
    assert!(thumb.width() <= 256 && thumb.height() <= 256);
}

#[test]
fn test_cache_hit() {
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = tempfile::tempdir().unwrap();
    let path = create_test_jpeg(dir.path(), "test.jpg", 100, 80);
    let cache = ThumbnailCache::new(Some(cache_dir.path().to_path_buf()));
    let _ = cache.get_thumbnail(&path);
    let key = cache_key(&path).unwrap();
    assert!(cache_dir.path().join(format!("{}.png", key)).exists());
}

#[test]
fn test_nonexistent() {
    let cache_dir = tempfile::tempdir().unwrap();
    let cache = ThumbnailCache::new(Some(cache_dir.path().to_path_buf()));
    assert!(cache.get_thumbnail(std::path::Path::new("/nonexistent.jpg")).is_none());
}

#[test]
fn test_large_image_scaled() {
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = tempfile::tempdir().unwrap();
    let path = create_test_jpeg(dir.path(), "big.jpg", 4000, 3000);
    let cache = ThumbnailCache::new(Some(cache_dir.path().to_path_buf()));
    let thumb = cache.get_thumbnail(&path).unwrap();
    assert!(thumb.width() <= 256);
    assert!(thumb.height() <= 256);
}
