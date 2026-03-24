use std::path::PathBuf;
use image::{DynamicImage, RgbImage};
use omarchy_imageview::state::app_state::{AppState, ViewMode};

fn save_img(dir: &std::path::Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    DynamicImage::ImageRgb8(RgbImage::new(10, 10)).save(&p).unwrap();
    p
}

fn make_images(dir: &std::path::Path, count: usize) -> Vec<PathBuf> {
    (1..=count).map(|i| save_img(dir, &format!("photo{i}.jpg"))).collect()
}

#[test]
fn test_initial_state() {
    let state = AppState::new(Some(PathBuf::from("/tmp/test-state")));
    assert!(state.current_folder.is_none());
    assert!(state.files.is_empty());
    assert_eq!(state.index, 0);
    assert_eq!(state.view_mode, ViewMode::Grid);
    assert!(state.zoom_fit);
    assert_eq!(state.zoom, 1.0);
}

#[test]
fn test_navigate_next() {
    let dir = tempfile::tempdir().unwrap();
    make_images(dir.path(), 10);
    let mut state = AppState::new(Some(PathBuf::from("/tmp/test-nav")));
    state.load_folder(dir.path(), None);
    assert_eq!(state.index, 0);
    state.navigate_next();
    assert_eq!(state.index, 1);
    state.navigate_next();
    assert_eq!(state.index, 2);
}

#[test]
fn test_navigate_prev() {
    let dir = tempfile::tempdir().unwrap();
    make_images(dir.path(), 10);
    let mut state = AppState::new(Some(PathBuf::from("/tmp/test-prev")));
    state.load_folder(dir.path(), None);
    state.index = 5;
    state.navigate_prev();
    assert_eq!(state.index, 4);
}

#[test]
fn test_navigate_clamps() {
    let dir = tempfile::tempdir().unwrap();
    make_images(dir.path(), 5);
    let mut state = AppState::new(Some(PathBuf::from("/tmp/test-clamp")));
    state.load_folder(dir.path(), None);
    state.index = 4;
    state.navigate_next();
    assert_eq!(state.index, 4); // stays at end
    state.index = 0;
    state.navigate_prev();
    assert_eq!(state.index, 0); // stays at start
}

#[test]
fn test_current_file() {
    let dir = tempfile::tempdir().unwrap();
    make_images(dir.path(), 5);
    let mut state = AppState::new(Some(PathBuf::from("/tmp/test-cf")));
    state.load_folder(dir.path(), None);
    assert!(state.current_file().is_some());
}

#[test]
fn test_load_folder_with_target() {
    let dir = tempfile::tempdir().unwrap();
    let images = make_images(dir.path(), 10);
    let mut state = AppState::new(Some(PathBuf::from("/tmp/test-target")));
    state.load_folder(dir.path(), Some(&images[4]));
    assert_eq!(state.index, 4);
}

#[test]
fn test_persist_and_restore() {
    let dir = tempfile::tempdir().unwrap();
    make_images(dir.path(), 5);
    let config = tempfile::tempdir().unwrap();
    let mut state = AppState::new(Some(config.path().to_path_buf()));
    state.load_folder(dir.path(), None);
    state.save();

    let mut state2 = AppState::new(Some(config.path().to_path_buf()));
    state2.restore();
    assert_eq!(state2.last_folder, Some(dir.path().to_path_buf()));
}

#[test]
fn test_remove_file() {
    let dir = tempfile::tempdir().unwrap();
    make_images(dir.path(), 10);
    let mut state = AppState::new(Some(PathBuf::from("/tmp/test-rm")));
    state.load_folder(dir.path(), None);
    let count = state.files.len();
    let removed = state.files[3].clone();
    state.remove_file(&removed);
    assert_eq!(state.files.len(), count - 1);
    assert!(!state.files.contains(&removed));
}
