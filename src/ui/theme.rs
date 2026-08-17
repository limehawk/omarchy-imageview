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

pub fn hex_rgba(hex: &str) -> [u8; 4] {
    let h = hex.trim().trim_start_matches('#');
    let parse = |i| u8::from_str_radix(h.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    if h.len() >= 6 {
        [parse(0), parse(2), parse(4), 255]
    } else {
        [0xCE, 0xCD, 0xC3, 255]
    }
}

pub fn foreground_rgba() -> [u8; 4] {
    let colors = parse_colors(None);
    hex_rgba(colors.get("foreground").map(|s| s.as_str()).unwrap_or(FALLBACK_FG))
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
        background-color: {bg};
        border-bottom: 1px solid alpha({fg}, 0.08);
        padding: 8px 14px;
        min-height: 0;
    }}

    .toolbar-copy {{
        margin-right: 16px;
    }}

    .toolbar-identity {{
        color: {fg};
        font-weight: 500;
        font-size: 0.95em;
        letter-spacing: -0.01em;
    }}

    .toolbar-meta {{
        color: alpha({fg}, 0.45);
        font-size: 0.75em;
        letter-spacing: 0.02em;
    }}

    .toolbar-tools {{
        margin-left: 4px;
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
        min-height: 80px;
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

    .sort-menu checkbutton {{
        padding: 4px 10px;
    }}

    .empty-state {{
        color: alpha({fg}, 0.4);
        font-size: 1.2em;
    }}

    .single-image.nearest {{
        image-rendering: pixelated;
        image-rendering: crisp-edges;
    }}

    button.pixel-btn,
    menubutton.pixel-btn > button {{
        min-width: 32px;
        min-height: 32px;
        padding: 4px;
        border: none;
        border-radius: 4px;
        background: transparent;
        background-image: none;
        box-shadow: none;
        outline: none;
        color: {fg};
    }}

    menubutton.pixel-btn {{
        min-width: 32px;
        min-height: 32px;
        padding: 0;
        border: none;
        background: transparent;
        box-shadow: none;
    }}

    button.pixel-btn:hover,
    menubutton.pixel-btn > button:hover {{
        background: alpha({fg}, 0.08);
        background-image: none;
        box-shadow: none;
    }}

    button.pixel-btn:active,
    menubutton.pixel-btn > button:active,
    menubutton.pixel-btn > button:checked {{
        background: alpha({fg}, 0.12);
        background-image: none;
        box-shadow: none;
    }}

    button.pixel-btn:focus,
    button.pixel-btn:focus-visible,
    menubutton.pixel-btn > button:focus,
    menubutton.pixel-btn > button:focus-visible {{
        outline: 1px solid {accent};
        outline-offset: 0;
        box-shadow: none;
    }}
    "#)
}

/// Parse colors and return CSS string. Call once at startup.
pub fn load_theme() -> String {
    let colors = parse_colors(None);
    generate_css(&colors)
}
