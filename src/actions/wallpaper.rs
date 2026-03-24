use std::path::Path;
use std::process::Command;

pub fn set_wallpaper(image_path: &Path) -> bool {
    let bg_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".config/omarchy/current/theme/backgrounds");
    let bg_link = dirs::home_dir()
        .unwrap_or_default()
        .join(".config/omarchy/current/background");

    if std::fs::create_dir_all(&bg_dir).is_err() {
        return false;
    }

    let dest = bg_dir.join(image_path.file_name().unwrap_or_default());
    if dest != image_path && std::fs::copy(image_path, &dest).is_err() {
        return false;
    }

    // Update symlink
    let _ = std::fs::remove_file(&bg_link);
    if std::os::unix::fs::symlink(&dest, &bg_link).is_err() {
        return false;
    }

    // Restart swaybg
    let _ = Command::new("pkill").arg("swaybg").output();
    Command::new("swaybg")
        .args(["-i", &bg_link.display().to_string(), "-m", "fill"])
        .spawn()
        .is_ok()
}
