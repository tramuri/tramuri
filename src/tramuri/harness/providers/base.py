"""Provider harness contracts and safe executable probes."""

from __future__ import annotations

import os
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from tramuri.core.models import Capability, InstallationPlan
from tramuri.modules.pm.assets import (
    TRAMURI_BLOCK_END,
    TRAMURI_BLOCK_START,
    capability_components,
    identity_instructions,
    product_skill,
    product_skill_components,
)


@dataclass(frozen=True, slots=True)
class ProviderDetection:
    provider: str
    installed: bool
    executable: str | None
    version: str | None
    project_paths: tuple[str, ...]
    details: str


@dataclass(frozen=True, slots=True)
class ProviderValidation:
    provider: str
    valid: bool
    checks: tuple[str, ...]
    warnings: tuple[str, ...] = ()


class ProviderAdapter(Protocol):
    key: str
    display_name: str

    def detect(self, project_dir: Path) -> ProviderDetection: ...
    def capabilities(self, project_dir: Path) -> list[Capability]: ...
    def installation_plan(self, project_dir: Path) -> InstallationPlan: ...
    def validate_installation(self, project_dir: Path) -> ProviderValidation: ...


def probe_executable(command: str, timeout: float = 2.0) -> tuple[bool, str | None, str]:
    """Find a provider executable and optionally read its version.

    Finding the executable is enough to identify an installed provider. A
    version probe can fail because a CLI needs authentication, is a wrapper
    script, or does not implement ``--version``; that must not turn a real
    installation into a false negative.
    """
    executable = shutil.which(command)
    if not executable:
        return False, None, f"{command} was not found on PATH"
    try:
        completed = subprocess.run(
            [executable, "--version"],
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
            env={"PATH": os.environ.get("PATH", "")},
        )
    except (OSError, subprocess.SubprocessError) as exc:
        return True, executable, f"version probe failed: {type(exc).__name__}"
    output = (completed.stdout or completed.stderr).strip().splitlines()
    if completed.returncode != 0:
        return True, executable, f"version probe exited with code {completed.returncode}"
    return True, executable, output[0][:160] if output else "version unavailable"


__all__ = [
    "ProviderAdapter",
    "ProviderDetection",
    "ProviderValidation",
    "TRAMURI_BLOCK_END",
    "TRAMURI_BLOCK_START",
    "identity_instructions",
    "product_skill",
    "product_skill_components",
    "probe_executable",
    "capability_components",
]
