"""Tests for application state and persistence."""

from pathlib import Path

from src.state.app_state import AppState


def test_initial_state():
    state = AppState()
    assert state.current_folder is None
    assert state.files == []
    assert state.index == 0
    assert state.view_mode == "grid"
    assert state.zoom == 1.0


def test_navigate_next(sample_images, tmp_image_dir):
    state = AppState()
    state.load_folder(tmp_image_dir)
    assert state.index == 0
    state.navigate_next()
    assert state.index == 1
    state.navigate_next()
    assert state.index == 2


def test_navigate_prev(sample_images, tmp_image_dir):
    state = AppState()
    state.load_folder(tmp_image_dir)
    state.index = 5
    state.navigate_prev()
    assert state.index == 4


def test_navigate_next_clamps(sample_images, tmp_image_dir):
    state = AppState()
    state.load_folder(tmp_image_dir)
    state.index = len(state.files) - 1
    state.navigate_next()
    assert state.index == len(state.files) - 1


def test_navigate_prev_clamps(sample_images, tmp_image_dir):
    state = AppState()
    state.load_folder(tmp_image_dir)
    state.navigate_prev()
    assert state.index == 0


def test_current_file(sample_images, tmp_image_dir):
    state = AppState()
    state.load_folder(tmp_image_dir)
    assert state.current_file is not None
    assert state.current_file.suffix == ".jpg"


def test_load_folder_with_target(sample_images, tmp_image_dir):
    state = AppState()
    target = sample_images[4]
    state.load_folder(tmp_image_dir, target_file=target)
    assert state.index == 4


def test_persist_and_restore(tmp_path, sample_images, tmp_image_dir):
    config = tmp_path / "config"
    state = AppState(config_dir=config)
    state.load_folder(tmp_image_dir)
    state.save()

    state2 = AppState(config_dir=config)
    state2.restore()
    assert state2.last_folder == tmp_image_dir


def test_remove_file(sample_images, tmp_image_dir):
    state = AppState()
    state.load_folder(tmp_image_dir)
    count_before = len(state.files)
    removed = state.files[3]
    state.remove_file(removed)
    assert len(state.files) == count_before - 1
    assert removed not in state.files
