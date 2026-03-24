"""Shared test fixtures."""

from pathlib import Path

import pytest
from PIL import Image


@pytest.fixture
def tmp_image_dir(tmp_path):
    """Create a temp directory with test images."""
    img_dir = tmp_path / "images"
    img_dir.mkdir()
    return img_dir


@pytest.fixture
def sample_jpeg(tmp_image_dir):
    """Create a small test JPEG."""
    path = tmp_image_dir / "test.jpg"
    img = Image.new("RGB", (100, 80), color=(255, 0, 0))
    img.save(path, "JPEG")
    return path


@pytest.fixture
def sample_png(tmp_image_dir):
    """Create a small test PNG with alpha."""
    path = tmp_image_dir / "test.png"
    img = Image.new("RGBA", (100, 80), color=(0, 255, 0, 128))
    img.save(path, "PNG")
    return path


@pytest.fixture
def sample_images(tmp_image_dir):
    """Create a folder with several numbered test images."""
    paths = []
    for i in range(1, 11):
        path = tmp_image_dir / f"photo{i}.jpg"
        img = Image.new("RGB", (100, 80), color=(i * 25, 0, 0))
        img.save(path, "JPEG")
        paths.append(path)
    return paths
