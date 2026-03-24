use std::path::{Path, PathBuf};
use super::formats::is_supported;

/// Scan a directory for supported image files, naturally sorted.
pub fn scan_folder(directory: &Path) -> Vec<PathBuf> {
    if !directory.is_dir() {
        return Vec::new();
    }

    let mut files: Vec<PathBuf> = std::fs::read_dir(directory)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .map(|n| is_supported(&n.to_string_lossy()))
                    .unwrap_or(false)
        })
        .collect();

    files.sort_by(|a, b| {
        natord::compare(
            &a.file_name().unwrap_or_default().to_string_lossy().to_lowercase(),
            &b.file_name().unwrap_or_default().to_string_lossy().to_lowercase(),
        )
    });

    files
}
