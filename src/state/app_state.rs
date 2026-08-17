use std::path::{Path, PathBuf};

use crate::core::folder::SortMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    Single,
}

/// Named view of the current image. `s` cycles these; +/- leave them for a custom zoom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleMode {
    /// Scale up or down to fit (imv `full`).
    Fit,
    /// Scale and crop to fill (imv `crop`).
    Fill,
    /// One image pixel per screen pixel (imv `none` / actual).
    Actual,
}

impl ScaleMode {
    pub fn next(self) -> Self {
        match self {
            Self::Fit => Self::Fill,
            Self::Fill => Self::Actual,
            Self::Actual => Self::Fit,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Fit => "Fit",
            Self::Fill => "Fill",
            Self::Actual => "1:1",
        }
    }
}

pub struct AppState {
    pub current_folder: Option<PathBuf>,
    pub files: Vec<PathBuf>,
    pub index: usize,
    pub view_mode: ViewMode,
    pub zoom_fit: bool,
    pub zoom: f64,
    pub last_folder: Option<PathBuf>,
    /// Unsaved clockwise rotation applied on screen (0, 90, 180, 270).
    pub pending_rotation: u32,
    pub pending_flip_h: bool,
    pub pending_flip_v: bool,
    pub sort: SortMode,
    pub scale_mode: ScaleMode,
    /// Slideshow step in seconds. 0 = off.
    pub slideshow_secs: u32,
    pub nearest_neighbor: bool,
    /// Filmstrip under the picture. User-toggled; remembered.
    pub filmstrip_visible: bool,
    config_dir: PathBuf,
}

impl AppState {
    pub fn new(config_dir: Option<PathBuf>) -> Self {
        let dir = config_dir.unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("omarchy-imageview")
        });
        Self {
            current_folder: None,
            files: Vec::new(),
            index: 0,
            view_mode: ViewMode::Grid,
            zoom_fit: false,
            zoom: 1.0,
            last_folder: None,
            pending_rotation: 0,
            pending_flip_h: false,
            pending_flip_v: false,
            sort: SortMode::Date,
            scale_mode: ScaleMode::Fit,
            slideshow_secs: 0,
            nearest_neighbor: false,
            filmstrip_visible: true,
            config_dir: dir,
        }
    }

    pub fn current_file(&self) -> Option<&Path> {
        self.files.get(self.index).map(|p| p.as_path())
    }

    pub fn load_folder(&mut self, folder: &Path, target_file: Option<&Path>) {
        self.current_folder = Some(folder.to_path_buf());
        self.files = crate::core::folder::scan_folder(folder, self.sort);
        self.last_folder = self.current_folder.clone();

        if let Some(target) = target_file {
            if let Some(pos) = self.files.iter().position(|f| f == target) {
                self.index = pos;
            } else {
                self.index = 0;
            }
        } else {
            self.index = 0;
        }
    }

    pub fn navigate_next(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.index = (self.index + 1) % self.files.len();
        self.reset_zoom();
    }

    pub fn navigate_prev(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.index = if self.index == 0 {
            self.files.len() - 1
        } else {
            self.index - 1
        };
        self.reset_zoom();
    }

    pub fn navigate_to(&mut self, index: usize) {
        if !self.files.is_empty() {
            self.index = index.min(self.files.len() - 1);
            self.reset_zoom();
        }
    }

    /// Each navigation starts the new image at fit-to-window, so a stuck zoom
    /// or unsaved rotate from a previous image can't bleed across.
    fn reset_zoom(&mut self) {
        self.zoom_fit = true;
        self.zoom = 1.0;
        self.pending_rotation = 0;
        self.pending_flip_h = false;
        self.pending_flip_v = false;
        self.scale_mode = ScaleMode::Fit;
    }

    pub fn add_rotation(&mut self, degrees: u32) {
        self.pending_rotation = (self.pending_rotation + degrees) % 360;
    }

    pub fn toggle_flip_h(&mut self) {
        self.pending_flip_h = !self.pending_flip_h;
    }

    pub fn toggle_flip_v(&mut self) {
        self.pending_flip_v = !self.pending_flip_v;
    }

    pub fn is_dirty(&self) -> bool {
        self.pending_rotation != 0 || self.pending_flip_h || self.pending_flip_v
    }

    pub fn cycle_scale_mode(&mut self) -> ScaleMode {
        self.scale_mode = self.scale_mode.next();
        self.zoom_fit = self.scale_mode != ScaleMode::Actual;
        self.zoom = 1.0;
        self.scale_mode
    }

    /// `t` / `T` in imv: 0 means off, otherwise seconds per image.
    pub fn bump_slideshow(&mut self, delta: i32) -> u32 {
        let next = self.slideshow_secs as i32 + delta;
        self.slideshow_secs = next.clamp(0, 30) as u32;
        self.slideshow_secs
    }

    /// Returns true if the mode changed.
    pub fn set_sort(&mut self, sort: SortMode) -> bool {
        if self.sort == sort {
            return false;
        }
        self.sort = sort;
        true
    }

    pub fn remove_file(&mut self, path: &Path) {
        if let Some(pos) = self.files.iter().position(|f| f == path) {
            self.files.remove(pos);
            if self.index >= self.files.len() {
                self.index = self.files.len().saturating_sub(1);
            } else if self.index > pos {
                self.index -= 1;
            }
        }
    }

    pub fn save(&self) {
        std::fs::create_dir_all(&self.config_dir).ok();
        let state_path = self.config_dir.join("state.toml");
        let mut content = String::new();
        if let Some(ref folder) = self.last_folder {
            content.push_str(&format!("last_folder = \"{}\"\n", folder.display()));
        }
        content.push_str(&format!("sort = \"{}\"\n", self.sort.as_str()));
        content.push_str(&format!("filmstrip = {}\n", self.filmstrip_visible));
        std::fs::write(state_path, content).ok();
    }

    pub fn restore(&mut self) {
        let state_path = self.config_dir.join("state.toml");
        if let Ok(content) = std::fs::read_to_string(&state_path) {
            if let Ok(table) = content.parse::<toml::Table>() {
                if let Some(toml::Value::String(s)) = table.get("last_folder") {
                    self.last_folder = Some(PathBuf::from(s));
                }
                if let Some(toml::Value::String(s)) = table.get("sort") {
                    self.sort = SortMode::parse(s);
                }
                if let Some(toml::Value::Boolean(b)) = table.get("filmstrip") {
                    self.filmstrip_visible = *b;
                }
            }
        }
    }

    pub fn toggle_filmstrip(&mut self) -> bool {
        self.filmstrip_visible = !self.filmstrip_visible;
        self.filmstrip_visible
    }
}
