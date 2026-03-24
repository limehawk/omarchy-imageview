"""Tests for format detection."""

from src.core.formats import is_supported, detect_format, FormatGroup


def test_jpeg_extensions():
    assert is_supported("photo.jpg")
    assert is_supported("photo.JPEG")
    assert is_supported("photo.JPG")


def test_raw_extensions():
    assert is_supported("photo.cr2")
    assert is_supported("photo.NEF")
    assert is_supported("photo.arw")
    assert is_supported("photo.dng")


def test_heic_extensions():
    assert is_supported("photo.heic")
    assert is_supported("photo.HEIF")


def test_svg_extension():
    assert is_supported("icon.svg")


def test_unsupported():
    assert not is_supported("document.pdf")
    assert not is_supported("video.mp4")
    assert not is_supported("noext")


def test_detect_format_groups():
    assert detect_format("photo.jpg") == FormatGroup.PILLOW
    assert detect_format("photo.png") == FormatGroup.PILLOW
    assert detect_format("photo.webp") == FormatGroup.PILLOW
    assert detect_format("photo.bmp") == FormatGroup.PILLOW
    assert detect_format("photo.tif") == FormatGroup.PILLOW
    assert detect_format("photo.gif") == FormatGroup.PILLOW
    assert detect_format("photo.avif") == FormatGroup.HEIF
    assert detect_format("photo.heic") == FormatGroup.HEIF
    assert detect_format("photo.cr2") == FormatGroup.RAW
    assert detect_format("icon.svg") == FormatGroup.SVG
    assert detect_format("unknown.xyz") is None
