use std::path::{Path, PathBuf};
use gtk4::gio;
use gtk4::gio::prelude::*;
use super::formats::is_supported;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Date,
    DateOldest,
    Name,
    NameZa,
    Size,
    SizeSmallest,
    Type,
}

impl SortMode {
    pub const ALL: &'static [Self] = &[
        Self::Date,
        Self::DateOldest,
        Self::Name,
        Self::NameZa,
        Self::Size,
        Self::SizeSmallest,
        Self::Type,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::DateOldest => "date-oldest",
            Self::Name => "name",
            Self::NameZa => "name-za",
            Self::Size => "size",
            Self::SizeSmallest => "size-smallest",
            Self::Type => "type",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "date-oldest" => Self::DateOldest,
            "name" | "name-az" => Self::Name,
            "name-za" => Self::NameZa,
            "size" | "size-largest" => Self::Size,
            "size-smallest" => Self::SizeSmallest,
            "type" | "kind" => Self::Type,
            _ => Self::Date,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Date => "Newest first",
            Self::DateOldest => "Oldest first",
            Self::Name => "A–Z",
            Self::NameZa => "Z–A",
            Self::Size => "Largest",
            Self::SizeSmallest => "Smallest",
            Self::Type => "Type",
        }
    }

    pub fn action(self) -> &'static str {
        match self {
            Self::Date => "sort-date",
            Self::DateOldest => "sort-date-oldest",
            Self::Name => "sort-name",
            Self::NameZa => "sort-name-za",
            Self::Size => "sort-size",
            Self::SizeSmallest => "sort-size-smallest",
            Self::Type => "sort-type",
        }
    }

    pub fn from_action(action: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|m| m.action() == action)
    }

    pub fn tooltip(self) -> &'static str {
        match self {
            Self::Date => "Sorted newest first",
            Self::DateOldest => "Sorted oldest first",
            Self::Name => "Sorted A–Z",
            Self::NameZa => "Sorted Z–A",
            Self::Size => "Sorted largest first",
            Self::SizeSmallest => "Sorted smallest first",
            Self::Type => "Sorted by type",
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

    let mut files: Vec<(PathBuf, std::time::SystemTime, u64)> = read
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .map(|n| is_supported(&n.to_string_lossy()))
                    .unwrap_or(false)
        })
        .map(|p| {
            let meta = std::fs::metadata(&p).ok();
            let mtime = meta
                .as_ref()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let size = meta.map(|m| m.len()).unwrap_or(0);
            (p, mtime, size)
        })
        .collect();

    files.sort_by(|a, b| {
        let name_a = a.0.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
        let name_b = b.0.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
        let name_cmp = || natord::compare(&name_a, &name_b);
        let ext_a = a.0.extension().unwrap_or_default().to_string_lossy().to_lowercase();
        let ext_b = b.0.extension().unwrap_or_default().to_string_lossy().to_lowercase();
        match sort {
            SortMode::Date => b.1.cmp(&a.1).then_with(name_cmp),
            SortMode::DateOldest => a.1.cmp(&b.1).then_with(name_cmp),
            SortMode::Name => name_cmp(),
            SortMode::NameZa => name_cmp().reverse(),
            SortMode::Size => b.2.cmp(&a.2).then_with(name_cmp),
            SortMode::SizeSmallest => a.2.cmp(&b.2).then_with(name_cmp),
            SortMode::Type => ext_a.cmp(&ext_b).then_with(name_cmp),
        }
    });

    log::info!("scan_folder: {} images in {}", files.len(), directory.display());
    files.into_iter().map(|(p, _, _)| p).collect()
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
