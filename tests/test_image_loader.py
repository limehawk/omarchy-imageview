"""Tests for image loading/decoding."""

from PIL import Image

from src.core.image_loader import load_image


def test_load_jpeg(sample_jpeg):
    img = load_image(sample_jpeg)
    assert isinstance(img, Image.Image)
    assert img.size == (100, 80)


def test_load_png(sample_png):
    img = load_image(sample_png)
    assert isinstance(img, Image.Image)
    assert img.size == (100, 80)


def test_load_nonexistent(tmp_path):
    result = load_image(tmp_path / "nope.jpg")
    assert result is None


def test_load_corrupt_file(tmp_image_dir):
    bad = tmp_image_dir / "corrupt.jpg"
    bad.write_bytes(b"not an image")
    result = load_image(bad)
    assert result is None


def test_load_webp(tmp_image_dir):
    path = tmp_image_dir / "test.webp"
    img = Image.new("RGB", (50, 50), color=(0, 0, 255))
    img.save(path, "WebP")
    result = load_image(path)
    assert isinstance(result, Image.Image)
    assert result.size == (50, 50)
