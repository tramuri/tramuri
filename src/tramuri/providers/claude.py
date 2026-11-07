"""Anthropic Claude Code project integration."""

from __future__ import annotations

from pathlib import Path

from tramuri.core.models import (
    Capability,
    CapabilityStatus,
    InstallationComponent,
    InstallationPlan,
)
from tramuri.providers.base import (
    ProviderAdapter,
    ProviderDetection,
    ProviderValidation,
    pm_skill_components,
    probe_executable,
    specialist_components,
)

CLAUDE_SKILL = "---\nname: tramuri-pm\ndescription: Coordinate a Tramuri Product Management workflow and preserve verifiable artifacts.\n---\n\nUse Tramuri's local CLI for workflow state and reports. Never fabricate evidence.\n"
CLAUDE_AGENT = "---\nname: tramuri-product-manager\ndescription: Coordinates the Tramuri Product Management workflow.\ntools: Read, Glob, Grep, Bash\n---\n\nUse the tramuri-pm skill and inspect persisted artifacts before summarizing.\n"
CLAUDE_HOOK = '#!/usr/bin/env python3\nfrom tramuri.hooks.checkpoint import main\n\nif __name__ == "__main__":\n    main()\n'
CLAUDE_SETTINGS = '{\n  "hooks": {\n    "PostToolUse": [{"matcher": "tramuri", "hooks": [{"type": "command", "command": "python .claude/hooks/tramuri-checkpoint.py"}]}]\n  }\n}\n'


class ClaudeAdapter(ProviderAdapter):
    key = "claude"
    display_name = "Anthropic Claude Code"

    def detect(self, project_dir: Path) -> ProviderDetection:
        installed, executable, version = probe_executable("claude")
        candidates = (project_dir / "CLAUDE.md", project_dir / ".claude")
        paths = tuple(str(path) for path in candidates if path.exists())
        return ProviderDetection(
            self.key,
            installed or bool(paths),
            executable,
            version,
            paths,
            "Claude settings are merged",
        )

    def capabilities(self, project_dir: Path) -> list[Capability]:
        status = (
            CapabilityStatus.AVAILABLE
            if self.detect(project_dir).installed
            else CapabilityStatus.MISSING
        )
        return [
            Capability(self.key, key, status, path)
            for key, path in (
                ("project-instructions", "CLAUDE.md"),
                ("skills", ".claude/skills"),
                ("subagents", ".claude/agents"),
                ("hooks", ".claude/settings.json"),
            )
        ]

    def installation_plan(self, project_dir: Path) -> InstallationPlan:
        return InstallationPlan(
            self.key,
            [
                InstallationComponent(
                    "pm-skill",
                    ".claude/skills/tramuri-pm/SKILL.md",
                    CLAUDE_SKILL,
                    description="Tramuri PM skill",
                ),
                InstallationComponent(
                    "pm-agent",
                    ".claude/agents/tramuri-product-manager.md",
                    CLAUDE_AGENT,
                    description="Claude subagent",
                ),
                InstallationComponent(
                    "checkpoint-hook",
                    ".claude/hooks/tramuri-checkpoint.py",
                    CLAUDE_HOOK,
                    description="Safe checkpoint hook",
                ),
                InstallationComponent(
                    "settings",
                    ".claude/settings.json",
                    CLAUDE_SETTINGS,
                    optional=True,
                    merge_json=True,
                    description="Hook registration",
                ),
            ]
            + pm_skill_components(".claude/skills")
            + specialist_components(".claude/agents", "md"),
            ["CLAUDE.md is not overwritten; settings.json is merged only for the Tramuri hook."],
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
