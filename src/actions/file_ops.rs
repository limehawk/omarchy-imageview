use gio::prelude::*;
use std::path::{Path, PathBuf};

/// Move file to trash via gio. Returns true on success.
pub fn trash_file(path: &Path) -> bool {
    let file = gio::File::for_path(path);
    file.trash(gio::Cancellable::NONE).is_ok()
}

/// Rotate image by degrees (90, 180, 270) and save back.
pub fn rotate_image(path: &Path, degrees: u32) -> bool {
    match image::open(path) {
        Ok(img) => {
            let rotated = match degrees {
                90 => img.rotate90(),
                180 => img.rotate180(),
                270 => img.rotate270(),
                _ => return false,
            };
            rotated.save(path).is_ok()
        }
        Err(_) => false,
    }
}

/// Rename a file in the same directory. Returns new path or None on failure.
pub fn rename_file(path: &Path, new_name: &str) -> Option<PathBuf> {
    let new_path = path.parent()?.join(new_name);
    if new_path.exists() {
        return None;
    }
    std::fs::rename(path, &new_path).ok()?;
    Some(new_path)
}

/// Copy file to destination directory.
pub fn copy_file(path: &Path, dest_dir: &Path) -> Option<PathBuf> {
    let dest = dest_dir.join(path.file_name()?);
    std::fs::copy(path, &dest).ok()?;
    Some(dest)
}

/// Move file to destination directory.
pub fn move_file(path: &Path, dest_dir: &Path) -> Option<PathBuf> {
    let dest = dest_dir.join(path.file_name()?);
    std::fs::rename(path, &dest).ok()?;
    Some(dest)
}
