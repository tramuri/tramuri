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
CLAUDE_AGENT = "---\nname: tramuri\ndescription: Routes the user's product goal through relevant Tramuri capabilities and verifies the resulting artifacts.\ntools: Read, Glob, Grep, Bash, Agent\n---\n\nIdentify Tramuri and inspect persisted status and the latest checkpoint first. For recall questions, run `tramuri memory show`, search relevant topics with `tramuri memory search`, and verify active work with `tramuri status` before answering. Save confirmed durable preferences, constraints, facts and decisions with `tramuri memory save`; never save raw transcripts or credentials. This is a product-capability workflow, not a professional-role persona. Ask one grouped question only when product intent is genuinely missing; never invent a product.\n\nRead only the product skills relevant to the current request. Use the matching Tramuri custom subagent through Agent with one bounded task and a named expected artifact. For a complete definition, route framing, decision-relevant research, scope, product journey/experience, one independent review, and traceability in dependency order. Focused requests use only their matching capability. Never invoke all agents by default. If Agent or the matching specialist is unavailable, perform the bounded capability inline and do not claim delegation.\n\nReuse SQLite-backed state and prior artifacts. Keep user facts, external evidence, inference, hypothesis, assumption, decision and pending validation distinct. Research must use available tools and produce source-linked tramuri.research.v1; otherwise preserve not_run. Never invent interviews, statistics, willingness to pay, validation or product features. Consolidate results, persist through Tramuri CLI, and report actual validation receipts and limitations.\n"
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
