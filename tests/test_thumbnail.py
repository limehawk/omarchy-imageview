"""Tests for thumbnail generation and caching."""

from pathlib import Path

from PIL import Image

from src.core.thumbnail import ThumbnailCache, _cache_key


def test_cache_key_deterministic(sample_jpeg):
    key1 = _cache_key(sample_jpeg)
    key2 = _cache_key(sample_jpeg)
    assert key1 == key2
    assert len(key1) == 64


def test_cache_key_changes_with_mtime(sample_jpeg):
    key1 = _cache_key(sample_jpeg)
    sample_jpeg.touch()
    key2 = _cache_key(sample_jpeg)
    assert key1 != key2


def test_generate_thumbnail(sample_jpeg, tmp_path):
    cache = ThumbnailCache(cache_dir=tmp_path / "cache")
    thumb = cache.get_thumbnail(sample_jpeg)
    assert isinstance(thumb, Image.Image)
    assert max(thumb.size) <= 256


def test_cache_hit(sample_jpeg, tmp_path):
    cache = ThumbnailCache(cache_dir=tmp_path / "cache")
    thumb1 = cache.get_thumbnail(sample_jpeg)
    thumb2 = cache.get_thumbnail(sample_jpeg)
    assert thumb1.size == thumb2.size
    key = _cache_key(sample_jpeg)
    assert (tmp_path / "cache" / f"{key}.png").exists()


def test_nonexistent_file(tmp_path):
    cache = ThumbnailCache(cache_dir=tmp_path / "cache")
    result = cache.get_thumbnail(tmp_path / "nope.jpg")
    assert result is None


def test_large_image_thumbnail(tmp_image_dir, tmp_path):
    path = tmp_image_dir / "big.jpg"
    img = Image.new("RGB", (4000, 3000), color=(100, 100, 100))
    img.save(path, "JPEG")
    cache = ThumbnailCache(cache_dir=tmp_path / "cache")
    thumb = cache.get_thumbnail(path)
    assert max(thumb.size) <= 256
    assert thumb.size[0] > 0 and thumb.size[1] > 0
