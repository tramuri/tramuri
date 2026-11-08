"""Provider fixtures that never inspect real user configuration."""

from __future__ import annotations

from pathlib import Path

from tramuri.core.models import (
    Capability,
    CapabilityStatus,
    InstallationComponent,
    InstallationPlan,
)
from tramuri.harness.manifest import HarnessManifest


class FixtureAdapter:
    key = "fixture"
    display_name = "Fixture Provider"

    def detect(self, project_dir: Path):
        return type(
            "FixtureDetection",
            (),
            {
                "provider": self.key,
                "installed": False,
                "executable": None,
                "version": None,
                "project_paths": (),
                "details": "fixture only",
            },
        )()

    def capabilities(self, project_dir: Path) -> list[Capability]:
        return [Capability(self.key, "skills", CapabilityStatus.AVAILABLE, "fixture/skills")]

    def installation_plan(self, project_dir: Path) -> InstallationPlan:
        return InstallationPlan(
            self.key,
            [InstallationComponent("skill", ".fixture/skill.md", "fixture")],
            ["fixture harness"],
        )

    def validate_installation(self, project_dir: Path):
        return type(
            "FixtureValidation", (), {"provider": self.key, "valid": True, "checks": ("skill",)}
        )()

    def manifest(self, project_dir: Path) -> HarnessManifest:
        plan = self.installation_plan(project_dir)
        return HarnessManifest(
            self.key,
            "fixture-1",
            tuple(plan.components),
            ("skills",),
            ("Prompt",),
            tuple(plan.notes),
        )
