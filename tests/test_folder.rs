use std::path::PathBuf;
use image::{DynamicImage, RgbImage};
use omarchy_imageview::core::folder::{scan_folder, SortMode};

fn save_img(dir: &std::path::Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    DynamicImage::ImageRgb8(RgbImage::new(10, 10)).save(&p).unwrap();
    p
}

#[test]
fn test_scan_finds_images() {
    let dir = tempfile::tempdir().unwrap();
    for i in 1..=5 { save_img(dir.path(), &format!("photo{i}.jpg")); }
    let files = scan_folder(dir.path(), SortMode::Date);
    assert_eq!(files.len(), 5);
}

#[test]
fn test_sorts_newest_first() {
    use std::time::{Duration, SystemTime};

    let dir = tempfile::tempdir().unwrap();
    let older = save_img(dir.path(), "older.jpg");
    let newer = save_img(dir.path(), "newer.jpg");
    let oldest = save_img(dir.path(), "oldest.jpg");

    let epoch = SystemTime::UNIX_EPOCH;
    std::fs::File::open(&oldest).unwrap().set_modified(epoch + Duration::from_secs(100)).unwrap();
    std::fs::File::open(&older).unwrap().set_modified(epoch + Duration::from_secs(200)).unwrap();
    std::fs::File::open(&newer).unwrap().set_modified(epoch + Duration::from_secs(300)).unwrap();

    let files = scan_folder(dir.path(), SortMode::Date);
    let names: Vec<_> = files.iter().map(|f| f.file_name().unwrap().to_string_lossy().to_string()).collect();
    assert_eq!(names, vec!["newer.jpg", "older.jpg", "oldest.jpg"]);
}

#[test]
fn test_sorts_by_name() {
    use std::time::{Duration, SystemTime};

    let dir = tempfile::tempdir().unwrap();
    let z = save_img(dir.path(), "z-last.jpg");
    let a = save_img(dir.path(), "a-first.jpg");
    let epoch = SystemTime::UNIX_EPOCH;
    // Newest file is z-last, but name sort should ignore that.
    std::fs::File::open(&a).unwrap().set_modified(epoch + Duration::from_secs(100)).unwrap();
    std::fs::File::open(&z).unwrap().set_modified(epoch + Duration::from_secs(300)).unwrap();

    let files = scan_folder(dir.path(), SortMode::Name);
    let names: Vec<_> = files.iter().map(|f| f.file_name().unwrap().to_string_lossy().to_string()).collect();
    assert_eq!(names, vec!["a-first.jpg", "z-last.jpg"]);
}

#[test]
fn test_same_date_falls_back_to_name() {
    use std::time::{Duration, SystemTime};

    let dir = tempfile::tempdir().unwrap();
    let a = save_img(dir.path(), "photo10.jpg");
    let b = save_img(dir.path(), "photo2.jpg");
    let t = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    std::fs::File::open(&a).unwrap().set_modified(t).unwrap();
    std::fs::File::open(&b).unwrap().set_modified(t).unwrap();

    let files = scan_folder(dir.path(), SortMode::Date);
    let names: Vec<_> = files.iter().map(|f| f.file_name().unwrap().to_string_lossy().to_string()).collect();
    assert_eq!(names, vec!["photo2.jpg", "photo10.jpg"]);
}

#[test]
fn test_sorts_oldest_first() {
    use std::time::{Duration, SystemTime};

    let dir = tempfile::tempdir().unwrap();
    let older = save_img(dir.path(), "older.jpg");
    let newer = save_img(dir.path(), "newer.jpg");
    let epoch = SystemTime::UNIX_EPOCH;
    std::fs::File::open(&older).unwrap().set_modified(epoch + Duration::from_secs(100)).unwrap();
    std::fs::File::open(&newer).unwrap().set_modified(epoch + Duration::from_secs(300)).unwrap();

    let files = scan_folder(dir.path(), SortMode::DateOldest);
    let names: Vec<_> = files.iter().map(|f| f.file_name().unwrap().to_string_lossy().to_string()).collect();
    assert_eq!(names, vec!["older.jpg", "newer.jpg"]);
}

#[test]
fn test_sorts_name_za() {
    let dir = tempfile::tempdir().unwrap();
    save_img(dir.path(), "a-first.jpg");
    save_img(dir.path(), "z-last.jpg");
    let files = scan_folder(dir.path(), SortMode::NameZa);
    let names: Vec<_> = files.iter().map(|f| f.file_name().unwrap().to_string_lossy().to_string()).collect();
    assert_eq!(names, vec!["z-last.jpg", "a-first.jpg"]);
}

#[test]
fn test_sorts_by_size() {
    let dir = tempfile::tempdir().unwrap();
    DynamicImage::ImageRgb8(RgbImage::new(8, 8))
        .save(dir.path().join("small.jpg"))
        .unwrap();
    DynamicImage::ImageRgb8(RgbImage::new(80, 80))
        .save(dir.path().join("big.jpg"))
        .unwrap();

    let large_first = scan_folder(dir.path(), SortMode::Size);
    let large_names: Vec<_> = large_first
        .iter()
        .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(large_names, vec!["big.jpg", "small.jpg"]);

    let small_first = scan_folder(dir.path(), SortMode::SizeSmallest);
    let small_names: Vec<_> = small_first
        .iter()
        .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(small_names, vec!["small.jpg", "big.jpg"]);
}

#[test]
fn test_sorts_by_type() {
    let dir = tempfile::tempdir().unwrap();
    save_img(dir.path(), "zeta.jpg");
    DynamicImage::ImageRgb8(RgbImage::new(10, 10))
        .save(dir.path().join("alpha.png"))
        .unwrap();
    let files = scan_folder(dir.path(), SortMode::Type);
    let names: Vec<_> = files.iter().map(|f| f.file_name().unwrap().to_string_lossy().to_string()).collect();
    assert_eq!(names, vec!["zeta.jpg", "alpha.png"]);
}

#[test]
fn test_ignores_unsupported() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("readme.txt"), "hello").unwrap();
    save_img(dir.path(), "photo.jpg");
    let files = scan_folder(dir.path(), SortMode::Date);
    assert_eq!(files.len(), 1);
}

#[test]
fn test_empty_folder() {
    let dir = tempfile::tempdir().unwrap();
    assert!(scan_folder(dir.path(), SortMode::Date).is_empty());
}
