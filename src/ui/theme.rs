use std::collections::HashMap;
use std::path::PathBuf;

fn omarchy_colors_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".config/omarchy/current/theme/colors.toml")
}

const FALLBACK_ACCENT: &str = "#4385BE";
const FALLBACK_FG: &str = "#CECDC3";
const FALLBACK_BG: &str = "#100F0F";
const FALLBACK_SEL_BG: &str = "#403E3C";
const FALLBACK_SURFACE: &str = "#1C1B1A";

pub fn parse_colors(path: Option<&std::path::Path>) -> HashMap<String, String> {
    let path = path.map(|p| p.to_path_buf()).unwrap_or_else(omarchy_colors_path);
    let mut colors = HashMap::new();

    // Set fallbacks
    colors.insert("accent".into(), FALLBACK_ACCENT.into());
    colors.insert("foreground".into(), FALLBACK_FG.into());
    colors.insert("background".into(), FALLBACK_BG.into());
    colors.insert("selection_bg".into(), FALLBACK_SEL_BG.into());
    colors.insert("color0".into(), FALLBACK_SURFACE.into());

    // Try to parse the file
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(table) = content.parse::<toml::Table>() {
            for (key, value) in table {
                if let toml::Value::String(s) = value {
                    colors.insert(key, s);
                }
            }
        }
    }

    colors
}

pub fn generate_css(colors: &HashMap<String, String>) -> String {
    let bg = colors.get("background").map(|s| s.as_str()).unwrap_or(FALLBACK_BG);
    let fg = colors.get("foreground").map(|s| s.as_str()).unwrap_or(FALLBACK_FG);
    let accent = colors.get("accent").map(|s| s.as_str()).unwrap_or(FALLBACK_ACCENT);
    let surface = colors.get("color0").map(|s| s.as_str()).unwrap_or(FALLBACK_SURFACE);
    let sel_bg = colors.get("selection_bg").map(|s| s.as_str()).unwrap_or(FALLBACK_SEL_BG);

    format!(r#"
    window {{
        background-color: {bg};
        color: {fg};
    }}

    .toolbar {{
        background-color: {surface};
        border-bottom: 1px solid {sel_bg};
        padding: 4px 12px;
    }}

    .grid-item {{
        border: 2px solid transparent;
        border-radius: 4px;
        padding: 4px;
    }}

    .grid-item:selected,
    .grid-item.selected {{
        border-color: {accent};
        background-color: alpha({accent}, 0.1);
    }}

    .filmstrip {{
        background-color: {surface};
        border-top: 1px solid {sel_bg};
        padding: 6px 8px;
    }}

    .filmstrip-item {{
        opacity: 0.5;
        border: 2px solid transparent;
        border-radius: 3px;
    }}

    .filmstrip-current {{
        opacity: 1.0;
        border-color: {accent};
    }}

    .info-panel {{
        background-color: {surface};
        border-left: 1px solid {sel_bg};
        padding: 12px;
    }}

    .info-label {{
        color: alpha({fg}, 0.6);
        font-size: 0.85em;
    }}

    .info-value {{
        color: {fg};
    }}

    .status-text {{
        color: alpha({fg}, 0.6);
        font-size: 0.9em;
    }}

    .accent-text {{
        color: {accent};
    }}

    .empty-state {{
        color: alpha({fg}, 0.4);
        font-size: 1.2em;
    }}
    "#)
}

/// Parse colors and return CSS string. Call once at startup.
pub fn load_theme() -> String {
    let colors = parse_colors(None);
    generate_css(&colors)
}
