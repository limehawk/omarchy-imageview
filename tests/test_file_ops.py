"""Tests for file operations."""

from PIL import Image

from src.actions.file_ops import rotate_image, rename_file, copy_file, move_file


def test_rotate_image(sample_jpeg):
    original = Image.open(sample_jpeg)
    orig_size = original.size
    original.close()
    rotate_image(sample_jpeg, 90)
    rotated = Image.open(sample_jpeg)
    assert rotated.size == (orig_size[1], orig_size[0])


def test_rotate_image_270(sample_jpeg):
    original = Image.open(sample_jpeg)
    orig_size = original.size
    original.close()
    rotate_image(sample_jpeg, 270)
    rotated = Image.open(sample_jpeg)
    assert rotated.size == (orig_size[1], orig_size[0])


def test_rename_file(sample_jpeg, tmp_image_dir):
    new_path = rename_file(sample_jpeg, "renamed.jpg")
    assert new_path == tmp_image_dir / "renamed.jpg"
    assert new_path.exists()
    assert not sample_jpeg.exists()


def test_rename_collision(sample_jpeg, tmp_image_dir):
    existing = tmp_image_dir / "existing.jpg"
    Image.new("RGB", (10, 10)).save(existing)
    result = rename_file(sample_jpeg, "existing.jpg")
    assert result is None


def test_copy_file(sample_jpeg, tmp_path):
    dest = tmp_path / "copies"
    dest.mkdir()
    result = copy_file(sample_jpeg, dest)
    assert result == dest / "test.jpg"
    assert result.exists()
    assert sample_jpeg.exists()


def test_move_file(sample_jpeg, tmp_path):
    dest = tmp_path / "moved"
    dest.mkdir()
    result = move_file(sample_jpeg, dest)
    assert result == dest / "test.jpg"
    assert result.exists()
    assert not sample_jpeg.exists()
