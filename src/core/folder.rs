use std::path::{Path, PathBuf};
use gtk4::gio;
use gtk4::gio::prelude::*;
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

pub struct FolderMonitor {
    _monitor: gio::FileMonitor,
}

impl FolderMonitor {
    pub fn new<F: Fn() + 'static>(directory: &Path, on_changed: F) -> Option<Self> {
        let gfile = gio::File::for_path(directory);
        let monitor = gfile
            .monitor_directory(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
            .ok()?;
        monitor.connect_changed(move |_monitor, file, _other, event| {
            let path = file.path().unwrap_or_default();
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            if !super::formats::is_supported(&filename) {
                return;
            }
            match event {
                gio::FileMonitorEvent::Created
                | gio::FileMonitorEvent::Deleted
                | gio::FileMonitorEvent::MovedIn
                | gio::FileMonitorEvent::MovedOut => {
                    on_changed();
                }
                _ => {}
            }
        });
        Some(Self { _monitor: monitor })
    }
}
