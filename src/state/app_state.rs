use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    Single,
}

pub struct AppState {
    pub current_folder: Option<PathBuf>,
    pub files: Vec<PathBuf>,
    pub index: usize,
    pub view_mode: ViewMode,
    pub zoom_fit: bool,
    pub zoom: f64,
    pub last_folder: Option<PathBuf>,
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
            zoom_fit: true,
            zoom: 1.0,
            last_folder: None,
            config_dir: dir,
        }
    }

    pub fn current_file(&self) -> Option<&Path> {
        self.files.get(self.index).map(|p| p.as_path())
    }

    pub fn load_folder(&mut self, folder: &Path, target_file: Option<&Path>) {
        self.current_folder = Some(folder.to_path_buf());
        self.files = crate::core::folder::scan_folder(folder);
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
        if !self.files.is_empty() && self.index < self.files.len() - 1 {
            self.index += 1;
        }
    }

    pub fn navigate_prev(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    pub fn navigate_to(&mut self, index: usize) {
        if !self.files.is_empty() {
            self.index = index.min(self.files.len() - 1);
        }
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
        std::fs::write(state_path, content).ok();
    }

    pub fn restore(&mut self) {
        let state_path = self.config_dir.join("state.toml");
        if let Ok(content) = std::fs::read_to_string(&state_path) {
            if let Ok(table) = content.parse::<toml::Table>() {
                if let Some(toml::Value::String(s)) = table.get("last_folder") {
                    self.last_folder = Some(PathBuf::from(s));
                }
            }
        }
    }
}
