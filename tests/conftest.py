from __future__ import annotations

from pathlib import Path

import pytest

from tramuri.storage.database import Database


@pytest.fixture
def project_dir(tmp_path: Path) -> Path:
    return tmp_path


@pytest.fixture
def database(project_dir: Path) -> Database:
    return Database(project_dir)
