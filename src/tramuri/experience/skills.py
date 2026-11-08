"""Contextual discovery of harness skills without copying them into Tramuri."""

from __future__ import annotations

import hashlib
import importlib.util
import re
from pathlib import Path

from tramuri.experience.models import SkillRecord, SkillSelection

CONTEXT_RULES: dict[str, tuple[tuple[str, str], ...]] = {
    "frontend": (
        ("frontend-design", "define visual direction and responsive interface tokens"),
        ("web-design-guidelines", "audit semantic, accessible and usable interface behavior"),
        ("verification-before-completion", "require rendered evidence before delivery"),
    ),
    "browser": (
        (
            "webapp-testing",
            "exercise the real experience in a browser when Playwright is available",
        ),
        ("verification-before-completion", "require browser evidence before delivery"),
    ),
    "python": (
        ("python-testing", "design pytest coverage for Python contracts and workflows"),
        ("test-driven-development", "write a failing contract test before implementation"),
        ("verification-before-completion", "require fresh command evidence before completion"),
    ),
    "debugging": (
        ("systematic-debugging", "find the root cause before changing implementation"),
        ("verification-before-completion", "re-run the relevant checks after repair"),
    ),
    "skill-authoring": (
        ("skill-creator", "define a focused skill contract and activation evaluation"),
        ("verification-before-completion", "validate the skill behavior before delivery"),
    ),
    "product-experience": (
        ("frontend-design", "shape the portal or product demo visual system"),
        ("web-design-guidelines", "review accessibility and interaction quality"),
        ("web-animation-design", "design restrained motion when the skill is installed"),
        ("webapp-testing", "validate the generated journey in a browser when available"),
        ("python-testing", "cover the generator, sensors and lifecycle contracts"),
        ("test-driven-development", "add a failing contract before changing the runtime"),
        ("verification-before-completion", "require evidence for every acceptance criterion"),
    ),
}


class SkillCatalog:
    """Inspect installed harness skills and select only task-relevant ones."""

    def __init__(self, harness_root: Path):
        self.harness_root = harness_root.resolve()
        self.skills_root = self.harness_root / ".agents" / "skills"
        self.records = self._discover()

    def _discover(self) -> dict[str, SkillRecord]:
        records: dict[str, SkillRecord] = {}
        if not self.skills_root.is_dir():
            return records
        for path in sorted(self.skills_root.glob("*/SKILL.md")):
            text = path.read_text(encoding="utf-8")
            frontmatter = re.match(r"^---\s*\n(.*?)\n---\s*\n", text, re.DOTALL)
            values = (
                dict(re.findall(r"^([a-z_-]+):\s*(.+?)\s*$", frontmatter.group(1), re.MULTILINE))
                if frontmatter
                else {}
            )
            name = values.get("name", path.parent.name).strip(" '\"")
            dependencies: tuple[str, ...] = ("playwright",) if name == "webapp-testing" else ()
            notes: list[str] = []
            status = "available"
            if not frontmatter or not values.get("name") or not values.get("description"):
                status = "invalid"
                notes.append("SKILL.md frontmatter requires name and description")
            if (
                status != "invalid"
                and "playwright" in dependencies
                and importlib.util.find_spec("playwright") is None
            ):
                status = "degraded"
                notes.append(
                    "playwright runtime is not installed; browser execution is unavailable"
                )
            records[name] = SkillRecord(
                name=name,
                source="installed-harness-skill",
                skill_path=str(path),
                content_hash=hashlib.sha256(text.encode("utf-8")).hexdigest(),
                capabilities=(values.get("description", "").strip(),),
                dependencies=dependencies,
                status=status,
                notes=tuple(notes),
            )
        return records

    def resolve(self, task_kind: str) -> SkillSelection:
        rules = CONTEXT_RULES.get(task_kind)
        if rules is None:
            return SkillSelection(
                task_kind,
                (),
                (),
                {"task": "No contextual skill mapping; no skills were loaded."},
                (),
            )
        selected: list[SkillRecord] = []
        unavailable: list[str] = []
        reasons: dict[str, str] = {}
        checks: list[str] = []
        for name, reason in rules:
            record = self.records.get(name)
            if record is None:
                unavailable.append(name)
                reasons[name] = "Skill is not installed in this harness."
                continue
            if record.status == "invalid":
                unavailable.append(name)
                reasons[name] = "; ".join(record.notes)
                continue
            selected.append(record)
            reasons[name] = reason
            checks.extend(record.notes)
            if record.status == "degraded":
                unavailable.extend(record.notes)
        return SkillSelection(
            task_kind, tuple(selected), tuple(unavailable), reasons, tuple(checks)
        )
