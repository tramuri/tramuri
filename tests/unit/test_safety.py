from pathlib import Path

import pytest

from tramuri.core.errors import UnsafePathError
from tramuri.core.safety import atomic_write, redact, safe_child


def test_safe_child_rejects_escape(tmp_path: Path) -> None:
    with pytest.raises(UnsafePathError):
        safe_child(tmp_path, "../outside.txt")


def test_atomic_write_and_redaction(tmp_path: Path) -> None:
    target = tmp_path / "nested" / "value.txt"
    atomic_write(target, "token=secret-value")
    assert target.read_text(encoding="utf-8") == "token=secret-value"
    assert "secret-value" not in redact("token=secret-value")
    assert "sk-" not in redact("sk-abcdefghijklmnop")
