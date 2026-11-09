"""OpenAI Codex project integration."""

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

CODEX_SKILL = product_skill("Codex")
CODEX_AGENT = '''name = "tramuri"
description = "Routes the user's product goal through relevant Tramuri capabilities and verifies the resulting artifacts."
developer_instructions = """Identify Tramuri and inspect persisted status and the latest checkpoint first. For recall questions, run tramuri memory show, search relevant topics with tramuri memory search, and verify active work with tramuri status before answering. Save confirmed durable preferences, constraints, facts and decisions with tramuri memory save; never save raw transcripts or credentials. This is a product-capability workflow, not a professional-role persona. Ask one grouped question only when product intent is missing; never invent a product. Read only the product skills relevant to the current request. Invoke the matching Tramuri custom agent with one bounded task and a named expected artifact. For full product definition, route framing, decision-relevant research, scope, product journey/experience, one independent review and traceability in dependency order. Focused requests use only their matching capability. Never invoke all agents by default. If a specialist tool is missing, do the bounded work inline and do not claim delegation. Reuse SQLite state and artifacts. Separate user facts, external evidence, inference, hypothesis, assumption, decision and pending validation. Research must produce source-linked tramuri.research.v1 or remain not_run. Never invent interviews, statistics, willingness to pay, validation or product features. Consolidate results, persist through Tramuri CLI and report actual validation receipts and limitations."""
'''
CODEX_INSTRUCTIONS = identity_instructions("Codex", ".agents/skills/tramuri/SKILL.md")
CODEX_HOOKS = """{
  "description": "Deterministic Tramuri workflow gate and SQLite checkpoint recorder.",
  "hooks": {
    "SessionStart": [{"matcher": "startup|resume|clear", "hooks": [{"type": "command", "command": "tramuri hooks runtime --provider codex", "timeout": 10}]}],
    "UserPromptSubmit": [{"hooks": [{"type": "command", "command": "tramuri hooks runtime --provider codex", "timeout": 10}]}],
    "PreToolUse": [{"matcher": "Bash|apply_patch|Edit|Write", "hooks": [{"type": "command", "command": "tramuri hooks runtime --provider codex", "timeout": 10, "statusMessage": "Checking Tramuri workflow state"}]}],
    "PostToolUse": [{"matcher": "Bash|apply_patch|Edit|Write", "hooks": [{"type": "command", "command": "tramuri hooks runtime --provider codex", "timeout": 10}]}]
  }
}
"""


class CodexAdapter(ProviderAdapter):
    key = "codex"
    display_name = "OpenAI Codex"

    def detect(self, project_dir: Path) -> ProviderDetection:
        installed, executable, version = probe_executable("codex")
        candidates = (project_dir / "AGENTS.md", project_dir / ".agents")
        paths = tuple(str(path) for path in candidates if path.exists())
        return ProviderDetection(
            self.key, installed or bool(paths), executable, version, paths, "AGENTS.md is preserved"
        )

    def capabilities(self, project_dir: Path) -> list[Capability]:
        detection = self.detect(project_dir)
        status = CapabilityStatus.AVAILABLE if detection.installed else CapabilityStatus.MISSING
        return [
            Capability(self.key, "project-instructions", status, "AGENTS.md"),
            Capability(self.key, "skills", status, ".agents/skills"),
            Capability(self.key, "subagents", status, ".codex/agents"),
            Capability(
                self.key, "hooks", CapabilityStatus.UNKNOWN, details="Not assumed by this adapter"
            ),
            Capability(
                self.key,
                "mcp",
                CapabilityStatus.UNKNOWN,
                details="Optional; never installed automatically",
            ),
        ]

    def installation_plan(self, project_dir: Path) -> InstallationPlan:
        return InstallationPlan(
            self.key,
            [
                InstallationComponent(
                    "project-instructions",
                    "AGENTS.md",
                    CODEX_INSTRUCTIONS,
                    merge_text=True,
                    description="Persistent Tramuri identity and workflow guidance",
                ),
                InstallationComponent(
                    "product-skill",
                    ".agents/skills/tramuri/SKILL.md",
                    CODEX_SKILL,
                    description="Tramuri product skill",
                ),
                InstallationComponent(
                    "hook-config",
                    ".codex/hooks.json",
                    CODEX_HOOKS,
                    merge_json=True,
                    description="Codex deterministic workflow hooks",
                ),
                InstallationComponent(
                    "product-agent",
                    ".codex/agents/tramuri.toml",
                    CODEX_AGENT,
                    optional=True,
                    description="Codex subagent definition",
                ),
            ]
            + product_skill_components(".agents/skills")
            + capability_components(".codex/agents", "toml"),
            [
                "AGENTS.md receives a managed Tramuri block without replacing existing instructions.",
                "Codex project hooks require review/trust with `/hooks` before they run.",
                "MCP is not claimed because availability is not verified.",
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
    "CODEX_AGENT",
    "CODEX_HOOKS",
    "CODEX_INSTRUCTIONS",
    "CODEX_SKILL",
    "CodexAdapter",
    "HarnessManifest",
]
