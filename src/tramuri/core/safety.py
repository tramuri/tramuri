"""Filesystem safety primitives used by installers and reports."""

from __future__ import annotations

import hashlib
import os
import re
import tempfile
from pathlib import Path

from tramuri.core.errors import UnsafePathError

SECRET_PATTERNS = (
    re.compile(r"(?i)(api[_-]?key|token|secret|password)\s*[:=]\s*[^\s,;]+"),
    re.compile(r"\bsk-[A-Za-z0-9_-]{12,}\b"),
)


def project_root(path: Path) -> Path:
    resolved = path.expanduser().resolve(strict=False)
    if not resolved.exists():
        raise UnsafePathError(f"Project directory does not exist: {resolved}")
    if not resolved.is_dir():
        raise UnsafePathError(f"Project path is not a directory: {resolved}")
    return resolved


def safe_child(root: Path, relative: str) -> Path:
    base = root.resolve()
    candidate = (base / relative).resolve(strict=False)
    try:
        candidate.relative_to(base)
    except ValueError as exc:
        raise UnsafePathError(f"Path escapes project root: {relative}") from exc
    return candidate


def sha256_bytes(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def atomic_write(path: Path, content: bytes | str) -> None:
    data = content.encode("utf-8") if isinstance(content, str) else content
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temp_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temp_name, path)
    finally:
        if os.path.exists(temp_name):
            os.unlink(temp_name)


def backup_file(path: Path, backup_root: Path) -> Path | None:
    if not path.exists():
        return None
    import shutil

    destination = backup_root / path.name
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(path, destination)
    return destination


def redact(value: str) -> str:
    result = value
    for pattern in SECRET_PATTERNS:
        result = pattern.sub(
            lambda match: (
                "<redacted>" if match.lastindex is None else f"{match.group(1)}=<redacted>"
            ),
            result,
        )
    return result
