"""Tests for Omarchy theme integration."""

from src.ui.theme import parse_colors, generate_css


def test_parse_colors(tmp_path):
    toml_file = tmp_path / "colors.toml"
    toml_file.write_text("""
accent = "#4385BE"
foreground = "#CECDC3"
background = "#100F0F"
selection_bg = "#403E3C"
selection_fg = "#CECDC3"
cursor = "#CECDC3"
color0 = "#1C1B1A"
color1 = "#AF3029"
""")
    colors = parse_colors(toml_file)
    assert colors["accent"] == "#4385BE"
    assert colors["background"] == "#100F0F"
    assert colors["color0"] == "#1C1B1A"


def test_parse_missing_file(tmp_path):
    colors = parse_colors(tmp_path / "nope.toml")
    assert "background" in colors
    assert "foreground" in colors
    assert "accent" in colors


def test_generate_css():
    colors = {
        "accent": "#4385BE",
        "foreground": "#CECDC3",
        "background": "#100F0F",
        "color0": "#1C1B1A",
        "selection_bg": "#403E3C",
    }
    css = generate_css(colors)
    assert "background-color" in css
    assert "#100F0F" in css
    assert "#4385BE" in css
    assert "#1C1B1A" in css
