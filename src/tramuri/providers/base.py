"""Provider adapter contracts and safe executable probes."""

from __future__ import annotations

import os
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from tramuri.core.models import Capability, InstallationComponent, InstallationPlan


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
    """Probe only a version command; never execute project files or hooks."""
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
        return False, executable, f"version probe failed: {type(exc).__name__}"
    output = (completed.stdout or completed.stderr).strip().splitlines()
    version = output[0][:160] if output else None
    return completed.returncode == 0, executable, version or "version command returned no output"


def pm_skill_components(root: str) -> list[InstallationComponent]:
    """Load the small reusable PM skills shipped in the wheel."""
    skill_root = Path(__file__).parent.parent / "templates" / "pm_skills"
    components: list[InstallationComponent] = []
    for directory in sorted(skill_root.iterdir()):
        source = directory / "SKILL.md"
        if source.is_file():
            components.append(
                InstallationComponent(
                    f"skill-{directory.name}",
                    f"{root}/{directory.name}/SKILL.md",
                    source.read_text(encoding="utf-8"),
                    description=f"Reusable PM skill: {directory.name}",
                )
            )
    return components


def specialist_components(root: str, extension: str) -> list[InstallationComponent]:
    roles = {
        "product-strategist": "problem, audience, value proposition",
        "discovery-specialist": "gaps, assumptions, context and evidence",
        "product-analyst": "outcomes, metrics and success criteria",
        "scope-specialist": "features, prioritization, scope and exclusions",
        "critical-reviewer": "contradictions, risks and result quality",
    }
    components: list[InstallationComponent] = []
    for role, responsibility in roles.items():
        if extension == "toml":
            content = (
                f'name = "tramuri-{role}"\ndescription = "PM specialist for {responsibility}."\n'
            )
            path = f"{root}/tramuri-{role}.toml"
        else:
            content = f"---\nname: tramuri-{role}\ndescription: PM specialist for {responsibility}.\n---\n\nPreserve evidence classification and report disagreements to the coordinator.\n"
            path = f"{root}/tramuri-{role}.md"
        components.append(
            InstallationComponent(
                f"agent-{role}", path, content, description=f"PM specialist: {role}"
            )
        )
    return components
