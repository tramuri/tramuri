"""Anthropic Claude Code project integration."""

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
    capability_components,
    identity_instructions,
    probe_executable,
    product_skill,
    product_skill_components,
)

CLAUDE_SKILL = product_skill("Claude Code")
CLAUDE_AGENT = "---\nname: tramuri\ndescription: Coordinates the Tramuri product workflow.\ntools: Read, Glob, Grep, Bash\n---\n\nUse the tramuri skill and inspect persisted artifacts before summarizing.\n"
CLAUDE_HOOK = '#!/usr/bin/env python3\nfrom tramuri.hooks.checkpoint import main\n\nif __name__ == "__main__":\n    main("claude")\n'
CLAUDE_SETTINGS = """{
  "hooks": {
    "SessionStart": [{"hooks": [{"type": "command", "command": "tramuri hooks runtime --provider claude"}]}],
    "UserPromptSubmit": [{"hooks": [{"type": "command", "command": "tramuri hooks runtime --provider claude"}]}],
    "PreToolUse": [{"matcher": "Bash|Edit|Write|NotebookEdit|MultiEdit", "hooks": [{"type": "command", "command": "tramuri hooks runtime --provider claude"}]}],
    "PostToolUse": [{"matcher": "Bash|Edit|Write|NotebookEdit|MultiEdit", "hooks": [{"type": "command", "command": "tramuri hooks runtime --provider claude"}]}]
  }
}
"""
CLAUDE_INSTRUCTIONS = identity_instructions("Claude Code", ".claude/skills/tramuri/SKILL.md")


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
                    "project-instructions",
                    "CLAUDE.md",
                    CLAUDE_INSTRUCTIONS,
                    merge_text=True,
                    description="Persistent Tramuri identity and workflow guidance",
                ),
                InstallationComponent(
                    "product-skill",
                    ".claude/skills/tramuri/SKILL.md",
                    CLAUDE_SKILL,
                    description="Tramuri product skill",
                ),
                InstallationComponent(
                    "product-agent",
                    ".claude/agents/tramuri.md",
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
            + product_skill_components(".claude/skills")
            + capability_components(".claude/agents", "md"),
            [
                "CLAUDE.md receives a managed Tramuri block without replacing existing instructions.",
                "settings.json is merged only for Tramuri lifecycle hooks.",
            ],
        )

    def manifest(self, project_dir: Path) -> HarnessManifest:
        plan = self.installation_plan(project_dir)
        return HarnessManifest(
            self.key,
            __version__,
            tuple(plan.components),
            tuple(item.key for item in self.capabilities(project_dir)),
            tuple(["SessionStart", "UserPromptSubmit", "PreToolUse", "PostToolUse"]),
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
    "CLAUDE_AGENT",
    "CLAUDE_HOOK",
    "CLAUDE_INSTRUCTIONS",
    "CLAUDE_SETTINGS",
    "CLAUDE_SKILL",
    "ClaudeAdapter",
    "HarnessManifest",
]
