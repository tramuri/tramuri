"""Google Gemini CLI project integration."""

from __future__ import annotations

from pathlib import Path

from tramuri import __version__
from tramuri.core.models import (
    Capability,
    CapabilityStatus,
    InstallationComponent,
    InstallationPlan,
)
from tramuri.harness.manifest import HarnessManifest
from tramuri.harness.providers.base import (
    ProviderAdapter,
    ProviderDetection,
    ProviderValidation,
    identity_instructions,
    probe_executable,
    product_skill,
    product_skill_components,
)
from tramuri.modules.pm.assets import gemini_command

GEMINI_SKILL = product_skill("Gemini CLI")
GEMINI_HOOK = '#!/usr/bin/env python3\nfrom tramuri.hooks.checkpoint import main\n\nif __name__ == "__main__":\n    main("gemini")\n'
GEMINI_SETTINGS = """{
  "hooks": {
    "SessionStart": [{"matcher": "startup|resume|clear", "hooks": [{"name": "tramuri-session", "type": "command", "command": "tramuri hooks runtime --provider gemini"}]}],
    "BeforeAgent": [{"hooks": [{"name": "tramuri-prompt", "type": "command", "command": "tramuri hooks runtime --provider gemini"}]}],
    "BeforeTool": [{"matcher": "run_shell_command|write_file|replace|edit", "hooks": [{"name": "tramuri-gate", "type": "command", "command": "tramuri hooks runtime --provider gemini"}]}],
    "AfterTool": [{"matcher": "run_shell_command|write_file|replace|edit", "hooks": [{"name": "tramuri-checkpoint", "type": "command", "command": "tramuri hooks runtime --provider gemini"}]}]
  }
}
"""
GEMINI_INSTRUCTIONS = identity_instructions("Gemini CLI", ".gemini/skills/tramuri/SKILL.md")


class GeminiAdapter(ProviderAdapter):
    key = "gemini"
    display_name = "Google Gemini CLI"

    def detect(self, project_dir: Path) -> ProviderDetection:
        installed, executable, version = probe_executable("gemini")
        candidates = (project_dir / "GEMINI.md", project_dir / ".gemini")
        paths = tuple(str(path) for path in candidates if path.exists())
        return ProviderDetection(
            self.key,
            installed or bool(paths),
            executable,
            version,
            paths,
            "Gemini hooks are version-dependent",
        )

    def capabilities(self, project_dir: Path) -> list[Capability]:
        status = (
            CapabilityStatus.AVAILABLE
            if self.detect(project_dir).installed
            else CapabilityStatus.MISSING
        )
        return [
            Capability(self.key, "project-instructions", status, "GEMINI.md"),
            Capability(self.key, "skills", status, ".gemini/skills"),
            Capability(self.key, "hooks", status, ".gemini/settings.json"),
            Capability(
                self.key,
                "subagents",
                CapabilityStatus.UNKNOWN,
                details="Version-dependent preview surface",
            ),
        ]

    def installation_plan(self, project_dir: Path) -> InstallationPlan:
        return InstallationPlan(
            self.key,
            [
                InstallationComponent(
                    "project-instructions",
                    "GEMINI.md",
                    GEMINI_INSTRUCTIONS,
                    merge_text=True,
                    description="Persistent Tramuri identity and workflow guidance",
                ),
                InstallationComponent(
                    "product-skill",
                    ".gemini/skills/tramuri/SKILL.md",
                    GEMINI_SKILL,
                    description="Tramuri product skill",
                ),
                InstallationComponent(
                    "checkpoint-hook",
                    ".gemini/hooks/tramuri-checkpoint.py",
                    GEMINI_HOOK,
                    description="Safe checkpoint hook",
                ),
                InstallationComponent(
                    "settings",
                    ".gemini/settings.json",
                    GEMINI_SETTINGS,
                    optional=True,
                    merge_json=True,
                    description="Hook registration",
                ),
                InstallationComponent(
                    "tramuri-command",
                    ".gemini/commands/tramuri.toml",
                    gemini_command(),
                    description="Gemini native /tramuri command",
                ),
            ]
            + product_skill_components(".gemini/skills"),
            [
                "GEMINI.md receives a managed Tramuri block without replacing existing instructions.",
                "Gemini hook support is version-dependent; the installer never claims unsupported subagents.",
            ],
        )

    def manifest(self, project_dir: Path) -> HarnessManifest:
        plan = self.installation_plan(project_dir)
        return HarnessManifest(
            self.key,
            __version__,
            tuple(plan.components),
            tuple(item.key for item in self.capabilities(project_dir)),
            tuple(["SessionStart", "BeforeAgent", "BeforeTool", "AfterTool"]),
            tuple(plan.notes),
        )

    def validate_installation(self, project_dir: Path) -> ProviderValidation:
        plan = self.installation_plan(project_dir)
        checks = tuple(
            item.relative_path
            for item in plan.components
            if (project_dir / item.relative_path).is_file()
        )
        return ProviderValidation(
            self.key, len(checks) == len(plan.components), checks, tuple(plan.notes)
        )


__all__ = [
    "GEMINI_HOOK",
    "GEMINI_INSTRUCTIONS",
    "GEMINI_SETTINGS",
    "GEMINI_SKILL",
    "GeminiAdapter",
    "HarnessManifest",
]
