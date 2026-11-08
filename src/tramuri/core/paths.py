"""Stable project-local paths owned by Tramuri."""

from __future__ import annotations

from pathlib import Path

OUTPUT_DIR_NAME = "tramuri_output"


def output_root(project_dir: Path) -> Path:
    """Return the single directory containing Tramuri state and generated output."""
    return project_dir.resolve() / OUTPUT_DIR_NAME
