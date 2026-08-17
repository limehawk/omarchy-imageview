use gio::prelude::*;
use std::path::{Path, PathBuf};

/// Move file to trash via gio. Returns true on success.
pub fn trash_file(path: &Path) -> bool {
    let file = gio::File::for_path(path);
    match file.trash(gio::Cancellable::NONE) {
        Ok(()) => {
            log::info!("trash_file: trashed {}", path.display());
            true
        }
        Err(e) => {
            log::error!("trash_file: failed for {}: {e}", path.display());
            false
        }
    }
}

/// Open the file in Pinta. Never `xdg-open` — this app is the default image
/// handler, so that would just relaunch us.
pub fn open_in_editor(path: &Path) -> bool {
    match std::process::Command::new("pinta").arg(path).spawn() {
        Ok(_) => {
            log::info!("open_in_editor: pinta {}", path.display());
            true
        }
        Err(e) => {
            log::error!("open_in_editor: pinta failed for {}: {e}", path.display());
            false
        }
    }
}

/// Rename a file in the same directory. Returns new path or None on failure.
pub fn rename_file(path: &Path, new_name: &str) -> Option<PathBuf> {
    let new_path = path.parent()?.join(new_name);
    if new_path.exists() {
        log::warn!("rename_file: target exists: {}", new_path.display());
        return None;
    }
    if let Err(e) = std::fs::rename(path, &new_path) {
        log::error!("rename_file: {} -> {} failed: {e}", path.display(), new_path.display());
        return None;
    }
    log::info!("rename_file: {} -> {}", path.display(), new_path.display());
    Some(new_path)
}

/// Copy file to destination directory.
pub fn copy_file(path: &Path, dest_dir: &Path) -> Option<PathBuf> {
    let dest = dest_dir.join(path.file_name()?);
    if let Err(e) = std::fs::copy(path, &dest) {
        log::error!("copy_file: {} -> {} failed: {e}", path.display(), dest.display());
        return None;
    }
    log::info!("copy_file: {} -> {}", path.display(), dest.display());
    Some(dest)
}

/// Move file to destination directory.
pub fn move_file(path: &Path, dest_dir: &Path) -> Option<PathBuf> {
    let dest = dest_dir.join(path.file_name()?);
    if let Err(e) = std::fs::rename(path, &dest) {
        log::error!("move_file: {} -> {} failed: {e}", path.display(), dest.display());
        return None;
    }
    log::info!("move_file: {} -> {}", path.display(), dest.display());
    Some(dest)
}

/// Persist a clockwise display rotation to `path`.
///
/// JPEG/TIFF: lossless EXIF orientation update via exiftool.
/// Other rasters: rotate pixels and overwrite.
/// HEIC/SVG/JXL: not written (no safe encoder path).
pub fn save_rotation(path: &Path, degrees: u32) -> bool {
    let degrees = degrees % 360;
    if degrees == 0 {
        return true;
    }
    if ![90, 180, 270].contains(&degrees) {
        log::warn!("save_rotation: unsupported degrees {degrees}");
        return false;
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let ok = match ext.as_str() {
        "jpg" | "jpeg" | "jpe" | "jfif" | "tif" | "tiff" => {
            save_via_exif(path, degrees) || save_via_pixels(path, degrees)
        }
        "heic" | "heif" | "svg" | "jxl" => {
            log::warn!(
                "save_rotation: cannot write rotation for .{ext} ({})",
                path.display()
            );
            false
        }
        _ => save_via_pixels(path, degrees),
    };
    if ok {
        log::info!("save_rotation: {} rotated {degrees}deg", path.display());
    }
    ok
}

fn save_via_exif(path: &Path, degrees: u32) -> bool {
    let current = crate::core::image_loader::read_exif_orientation(path).unwrap_or(1);
    let tag = crate::core::image_loader::compose_orientation_cw(current, degrees);
    write_exif_orientation(path, tag)
}

fn write_exif_orientation(path: &Path, tag: u32) -> bool {
    match std::process::Command::new("exiftool")
        .args([
            "-overwrite_original",
            "-n",
            &format!("-Orientation={tag}"),
            "-q",
        ])
        .arg(path)
        .status()
    {
        Ok(status) if status.success() => true,
        Ok(status) => {
            log::error!(
                "save_rotation: exiftool exited {status} for {}",
                path.display()
            );
            false
        }
        Err(e) => {
            log::error!("save_rotation: exiftool failed: {e}");
            false
        }
    }
}

fn save_via_pixels(path: &Path, degrees: u32) -> bool {
    let Some(img) = crate::core::image_loader::load_image(path) else {
        return false;
    };
    let rotated = crate::core::image_loader::rotate_degrees(img, degrees);
    match rotated.save(path) {
        Ok(()) => true,
        Err(e) => {
            log::error!("save_rotation: pixel save failed for {}: {e}", path.display());
            false
        }
    }
}
