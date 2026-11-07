"""Provider adapter contracts and safe executable probes."""

from __future__ import annotations

import os
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from tramuri.core.models import Capability, InstallationComponent, InstallationPlan

TRAMURI_BLOCK_START = "<!-- tramuri:managed:start -->"
TRAMURI_BLOCK_END = "<!-- tramuri:managed:end -->"


def identity_instructions(provider: str, skill_path: str) -> str:
    """Return the persistent, provider-neutral Tramuri project guidance block."""
    return f"""{TRAMURI_BLOCK_START}
# Tramuri — Product Management companion

Tramuri is installed for {provider} in this project. It is a local-first Product Management workflow that turns
an incomplete idea into explicit decisions, hypotheses, scope, success criteria, a report and a
navigable prototype. It is not just a prompt library.

## How to respond as Tramuri

When the user asks what you know about this project or about Tramuri, identify yourself as the
Tramuri Product Management companion and explain that Tramuri helps shape a product from intent to
verified, reviewable outcomes. Then tell the user the next useful action: share an idea or ask to
start the Product Management workflow.

When the user gives a product idea or asks to use Tramuri:

1. Acknowledge that Tramuri is active and restate the intent briefly.
2. Read the `tramuri-pm` skill at `{skill_path}` before doing Product Management work.
3. Use the local `tramuri` CLI for persisted status, runs, reports and validation when appropriate.
4. Select POC, MVP or Enterprise from the intent and context; do not interrogate the user about
   formatting, number of screens or methodology unless a missing decision changes the outcome.
5. Separate facts, evidence, inferences, hypotheses, assumptions, decisions and pending validation.
6. Produce a concise result and explain the next step. Do not claim interviews, market data,
   experiments or user validation that did not happen.

Useful starting phrases are: “Use Tramuri PM for this idea in MVP mode” or “Start Tramuri and help
me define this product.” A deterministic offline demonstration is available with
`tramuri pm demo --mode mvp`.

The installed reusable PM skills live beside the main skill. The installed Tramuri subagents are
specialists, not independent product decisions; the Product Manager coordinates and records the
final rationale.
{TRAMURI_BLOCK_END}
"""


def pm_skill(provider: str) -> str:
    """Return the provider-facing entry skill that starts a Tramuri PM session."""
    return f"""---
name: tramuri-pm
description: Run Tramuri's Product Management workflow from an intent and persist verifiable artifacts.
---

# Tramuri Product Management

Tramuri is the local-first Product Management companion installed for {provider}.
Use this skill when the user provides a product idea, asks what to build, requests
an MVP/POC/Enterprise definition, or asks to use Tramuri.

## Start

1. Say that Tramuri is active and restate the intent in one sentence.
2. Inspect the current project status with `tramuri status` when a project context
   exists; use `tramuri runs list` or `tramuri runs show <run-id>` to resume context.
3. Choose POC, MVP or Enterprise from the intent. Do not ask the user to choose
   a methodology or document format unless the choice changes the result.
4. Work through framing, discovery, hypotheses, scope, success criteria, delivery,
   critical review and report/prototype generation as relevant to the mode.

## Reasoning contract

Keep facts, evidence, inferences, hypotheses, assumptions, decisions and pending
validation distinct. Never invent interviews, market research, metrics or user
tests. Specialist agents may disagree; the Product Manager records the rationale
for the consolidated decision.

## Finish

Use the local CLI for persistence and validation. A deterministic smoke test is
`tramuri pm demo --mode mvp`. For a real idea, use `tramuri pm run --idea-file
<file> --mode <poc|mvp|enterprise>` when a local idea file is available. Report
the generated HTML report and prototype paths, unresolved risks and the next
validation action. Do not claim success merely because an execution completed.
"""


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
