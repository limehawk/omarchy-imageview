"""Tests for folder scanning and natural sort."""

from pathlib import Path

from PIL import Image

from src.core.folder import scan_folder


def test_scan_finds_images(sample_images, tmp_image_dir):
    files = scan_folder(tmp_image_dir)
    assert len(files) == 10
    assert all(isinstance(f, Path) for f in files)


def test_scan_natural_sort(tmp_image_dir):
    for name in ["photo10.jpg", "photo1.jpg", "photo2.jpg", "photo20.jpg"]:
        img = Image.new("RGB", (10, 10))
        img.save(tmp_image_dir / name)
    files = scan_folder(tmp_image_dir)
    names = [f.name for f in files]
    assert names == ["photo1.jpg", "photo2.jpg", "photo10.jpg", "photo20.jpg"]


def test_scan_ignores_unsupported(tmp_image_dir):
    (tmp_image_dir / "readme.txt").write_text("hello")
    (tmp_image_dir / "data.csv").write_text("a,b")
    img = Image.new("RGB", (10, 10))
    img.save(tmp_image_dir / "photo.jpg")
    files = scan_folder(tmp_image_dir)
    assert len(files) == 1
    assert files[0].name == "photo.jpg"


def test_scan_empty_folder(tmp_path):
    empty = tmp_path / "empty"
    empty.mkdir()
    files = scan_folder(empty)
    assert files == []


def test_scan_mixed_extensions(tmp_image_dir):
    Image.new("RGB", (10, 10)).save(tmp_image_dir / "a.jpg")
    Image.new("RGB", (10, 10)).save(tmp_image_dir / "b.png")
    Image.new("RGB", (10, 10)).save(tmp_image_dir / "c.webp")
    Image.new("RGB", (10, 10)).save(tmp_image_dir / "d.bmp")
    files = scan_folder(tmp_image_dir)
    assert len(files) == 4


def test_scan_case_insensitive(tmp_image_dir):
    Image.new("RGB", (10, 10)).save(tmp_image_dir / "photo.JPG")
    files = scan_folder(tmp_image_dir)
    assert len(files) == 1
