use std::path::{Path, PathBuf};
use gtk4::gio;
use gtk4::gio::prelude::*;
use super::formats::is_supported;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Date,
    Name,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            Self::Date => Self::Name,
            Self::Name => Self::Date,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::Name => "name",
        }
    }

    pub fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("name") {
            Self::Name
        } else {
            Self::Date
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Date => "Date",
            Self::Name => "Name",
        }
    }

    pub fn tooltip(self) -> &'static str {
        match self {
            Self::Date => "Sorted by date — click for name",
            Self::Name => "Sorted by name — click for date",
        }
    }
}

/// Scan a directory for supported image files.
pub fn scan_folder(directory: &Path, sort: SortMode) -> Vec<PathBuf> {
    if !directory.is_dir() {
        log::warn!("scan_folder: not a directory: {}", directory.display());
        return Vec::new();
    }

    let read = match std::fs::read_dir(directory) {
        Ok(r) => r,
        Err(e) => {
            log::error!("scan_folder: read_dir failed for {}: {e}", directory.display());
            return Vec::new();
        }
    };

    let mut files: Vec<(PathBuf, std::time::SystemTime)> = read
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .map(|n| is_supported(&n.to_string_lossy()))
                    .unwrap_or(false)
        })
        .map(|p| {
            let mtime = std::fs::metadata(&p)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            (p, mtime)
        })
        .collect();

    files.sort_by(|a, b| {
        let name_cmp = || {
            natord::compare(
                &a.0.file_name().unwrap_or_default().to_string_lossy().to_lowercase(),
                &b.0.file_name().unwrap_or_default().to_string_lossy().to_lowercase(),
            )
        };
        match sort {
            SortMode::Date => b.1.cmp(&a.1).then_with(name_cmp),
            SortMode::Name => name_cmp(),
        }
    });

    log::info!("scan_folder: {} images in {}", files.len(), directory.display());
    files.into_iter().map(|(p, _)| p).collect()
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
