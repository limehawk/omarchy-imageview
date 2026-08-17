use image::DynamicImage;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};

const THUMBNAIL_SIZE: u32 = 256;

pub fn cache_key(path: &Path) -> Option<String> {
    let abs = path.canonicalize().ok()?;
    let meta = fs::metadata(&abs).ok()?;
    let mtime = meta.modified().ok()?;
    let raw = format!("v2:{}:{:?}", abs.display(), mtime);
    let hash = Sha256::digest(raw.as_bytes());
    Some(format!("{:x}", hash))
}

pub struct ThumbnailCache {
    cache_dir: PathBuf,
}

impl ThumbnailCache {
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        let dir = cache_dir.unwrap_or_else(|| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("omarchy-imageview")
        });
        fs::create_dir_all(&dir).ok();
        Self { cache_dir: dir }
    }

    pub fn get_thumbnail(&self, path: &Path) -> Option<DynamicImage> {
        if !path.is_file() {
            return None;
        }
        let key = cache_key(path)?;
        let cached = self.cache_dir.join(format!("{}.png", key));

        if cached.exists() {
            if let Ok(img) = image::open(&cached) {
                return Some(img);
            }
            fs::remove_file(&cached).ok();
        }

        self.generate(path, &cached)
    }

    fn generate(&self, path: &Path, cache_path: &Path) -> Option<DynamicImage> {
        let img = super::image_loader::load_image(path)?;
        let thumb = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
        thumb.save(cache_path).ok()?;
        Some(thumb)
    }
}
