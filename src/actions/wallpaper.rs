use std::path::Path;
use std::process::Command;

pub fn set_wallpaper(image_path: &Path) -> bool {
    match Command::new("omarchy")
        .args(["theme", "bg", "set"])
        .arg(image_path)
        .status()
    {
        Ok(status) if status.success() => {
            log::info!("set_wallpaper: {}", image_path.display());
            true
        }
        Ok(status) => {
            log::error!(
                "set_wallpaper: omarchy theme bg set failed ({status}) for {}",
                image_path.display()
            );
            false
        }
        Err(e) => {
            log::error!("set_wallpaper: failed to run omarchy: {e}");
            false
        }
    }
}
