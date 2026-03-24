"""Omarchy theme integration — reads colors.toml, generates GTK CSS."""

import tomllib
from pathlib import Path

_OMARCHY_COLORS = Path.home() / ".config" / "omarchy" / "current" / "theme" / "colors.toml"

_FALLBACK_COLORS = {
    "accent": "#4385BE",
    "foreground": "#CECDC3",
    "background": "#100F0F",
    "selection_bg": "#403E3C",
    "selection_fg": "#CECDC3",
    "cursor": "#CECDC3",
    "color0": "#1C1B1A",
}


def parse_colors(path: Path | None = None) -> dict[str, str]:
    """Parse Omarchy colors.toml. Returns fallback colors if file missing."""
    if path is None:
        path = _OMARCHY_COLORS

    try:
        data = tomllib.loads(Path(path).read_text())
        return {**_FALLBACK_COLORS, **data}
    except (FileNotFoundError, tomllib.TOMLDecodeError):
        return dict(_FALLBACK_COLORS)


def generate_css(colors: dict[str, str]) -> str:
    """Generate GTK CSS from Omarchy color map."""
    bg = colors.get("background", "#100F0F")
    fg = colors.get("foreground", "#CECDC3")
    accent = colors.get("accent", "#4385BE")
    surface = colors.get("color0", "#1C1B1A")
    sel_bg = colors.get("selection_bg", "#403E3C")

    return f"""
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
    """


def load_theme():
    """Parse colors and return CSS string. Call once at startup."""
    colors = parse_colors()
    return generate_css(colors)
