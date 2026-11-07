"""Deterministic PM workflow runner.

The runner is intentionally useful without an LLM. A provider can later supply
deeper analysis, but state, evidence labels, retries, and artifacts stay local
and deterministic.
"""

from __future__ import annotations

import re
from collections.abc import Callable
from typing import Any

from tramuri.core.models import Mode, RunStatus, StageStatus
from tramuri.core.workflow import stage_specs
from tramuri.pm.stages import MODE_PROFILES, specialist_name
from tramuri.storage.database import Database

ProgressCallback = Callable[[str, str, str], None]


def _title(idea: str) -> str:
    cleaned = re.sub(r"\s+", " ", idea.strip()).strip(" .")
    return cleaned[:80] or "Unnamed product idea"


def _payloads(idea: str, mode: Mode) -> dict[str, dict[str, Any]]:
    title = _title(idea)
    profile = MODE_PROFILES[mode]
    base = {
        "product_name": title,
        "mode": mode.value,
        "mode_label": profile["label"],
        "focus": profile["focus"],
        "idea": idea.strip(),
        "classification": "inference",
    }
    return {
        "intent": base
        | {
            "desired_outcome": "Turn the stated intent into a product decision set.",
            "classification": "confirmed",
        },
        "framing": base
        | {
            "problem": f"People who need {title.lower()} may struggle to achieve the intended outcome consistently.",
            "target_users": ["Primary users inferred from the idea; validate with real users."],
            "value_proposition": f"A focused way to help users achieve the outcome implied by: {idea.strip()}",
            "classification": "hypothesis",
        },
        "discovery": base
        | {
            "gaps": [
                "User context",
                "Current alternatives",
                "Frequency and urgency of the problem",
            ],
            "questions": [
                "Who experiences this problem most often?",
                "What would make a first solution useful?",
            ],
            "classification": "pending_validation",
        },
        "hypothesis": base
        | {
            "hypotheses": [
                {
                    "statement": "A focused first workflow will be more useful than a broad feature set.",
                    "risk": "high",
                    "test": "Observe completion of the smallest end-to-end task.",
                },
                {
                    "statement": "The inferred primary user has a recurring need for the outcome.",
                    "risk": "high",
                    "test": "Interview or observe representative users.",
                },
            ],
            "classification": "hypothesis",
        },
        "experiment": base
        | {
            "main_hypothesis": "A small end-to-end experiment can establish whether the core outcome is feasible.",
            "question": "Can the user reach the intended outcome with the smallest useful flow?",
            "experiment": "Prototype the critical path and test it with representative scenarios.",
            "classification": "hypothesis",
        },
        "shaping": base
        | {
            "in_scope": ["Intent capture", "Core outcome flow", "Confirmation or result state"],
            "out_of_scope": [
                "Advanced administration",
                "Unvalidated integrations",
                "Operational scale work",
            ],
            "prioritization": "Value and risk first; defer breadth until the core outcome is validated.",
            "classification": "decision",
        },
        "success": base
        | {
            "outcomes": [
                "A representative user completes the core outcome",
                "The main uncertainty is reduced",
            ],
            "metrics": [
                {
                    "name": "Core-flow completion",
                    "target": "Define after baseline observation",
                    "state": "pending_validation",
                }
            ],
            "classification": "decision",
        },
        "governance": base
        | {
            "controls": [
                "Decision record",
                "Evidence reference",
                "Acceptance criteria",
                "Owner for each validation",
            ],
            "classification": "decision",
        },
        "traceability": base
        | {
            "trace": [
                "Intent -> framing -> hypothesis -> scope -> success criteria",
                "Every claim remains classified",
            ],
            "classification": "decision",
        },
        "delivery": base
        | {
            "increments": [
                {"name": "Critical path", "goal": "Prove the core user outcome"},
                {"name": "Learning loop", "goal": "Capture evidence and revise assumptions"},
                {"name": "Hardening", "goal": "Improve reliability after validation"},
            ],
            "classification": "decision",
        },
        "critical_review": base
        | {
            "findings": [
                "The user and problem framing are hypotheses until validated.",
                "Success metrics require a baseline and an owner.",
                "The scope intentionally excludes breadth before evidence.",
            ],
            "risks": ["Unvalidated demand", "Ambiguous success threshold", "Scope expansion"],
            "classification": "review",
        },
        "risk_review": base
        | {
            "risks": [
                "Governance overhead before evidence",
                "Unclear ownership",
                "Dependency drift",
            ],
            "mitigations": [
                "Review only relevant controls",
                "Assign an owner",
                "Revalidate dependencies",
            ],
            "classification": "review",
        },
        "report": base
        | {
            "report_sections": [
                "Overview",
                "Problem",
                "Users",
                "Hypotheses",
                "Scope",
                "Success",
                "Decisions",
                "Risks",
            ],
            "classification": "generated",
        },
        "navigation": base
        | {
            "prototype": "Offline HTML prototype scoped to the defined core journey.",
            "classification": "generated",
        },
    }


class PMRunner:
    def __init__(self, database: Database, progress: ProgressCallback | None = None):
        self.database = database
        self.progress = progress

    def _notify(self, stage: str, status: str, owner: str) -> None:
        if self.progress:
            self.progress(stage, status, owner)

    def run(
        self, idea: str, mode: Mode, provider: str = "local-demo", run_id: str | None = None
    ) -> str:
        run_id = run_id or self.database.create_run(idea, mode, provider)
        details = self.database.run_details(run_id)
        payloads = _payloads(str(details["run"]["idea"]), Mode(str(details["run"]["mode"])))
        for spec in stage_specs(mode):
            stage = self.database.stage(run_id, spec.key)
            if StageStatus(stage["status"]) == StageStatus.COMPLETED:
                continue
            if not self.database.completed_stages(run_id).issuperset(spec.dependencies):
                self.database.complete_run(run_id, RunStatus.BLOCKED)
                return run_id
            for attempt in range(1, 4):
                try:
                    self.database.event(
                        run_id, "stage.planned", {"stage": spec.key, "owner": spec.owner}
                    )
                    self.database.event(
                        run_id, "stage.delegated", {"stage": spec.key, "owner": spec.owner}
                    )
                    self.database.transition_stage(run_id, spec.key, StageStatus.RUNNING)
                    self._notify(spec.key, "running", specialist_name(spec.owner))
                    payload = payloads.get(
                        spec.key, {"stage": spec.key, "classification": "generated"}
                    )
                    self.database.event(run_id, "stage.executed", {"stage": spec.key})
                    artifact_id = self.database.add_artifact(
                        run_id, spec.key, f"pm.{spec.key}", payload, list(spec.dependencies)
                    )
                    self._persist_records(run_id, spec.key, payload)
                    self.database.validation(
                        run_id,
                        str(stage["id"]),
                        artifact_id,
                        f"pm.{spec.key}.integrity",
                        "passed",
                        "Structured payload persisted with classification",
                    )
                    self.database.event(
                        run_id, "stage.evaluated", {"stage": spec.key, "result": "passed"}
                    )
                    self.database.transition_stage(run_id, spec.key, StageStatus.COMPLETED)
                    self.database.event(
                        run_id, "stage.verified", {"stage": spec.key, "artifact_id": artifact_id}
                    )
                    self._notify(spec.key, "completed", specialist_name(spec.owner))
                    break
                except Exception as exc:
                    self.database.transition_stage(run_id, spec.key, StageStatus.FAILED, str(exc))
                    if attempt == 3:
                        self.database.complete_run(run_id, RunStatus.FAILED)
                        self._notify(spec.key, "failed", specialist_name(spec.owner))
                        raise
                    self.database.event(
                        run_id,
                        "stage.retry",
                        {"stage": spec.key, "attempt": attempt + 1, "error": str(exc)},
                    )
        self.database.complete_run(run_id, RunStatus.COMPLETED)
        return run_id

    def resume(self, run_id: str) -> str:
        details = self.database.run_details(run_id)
        self.database.resume_run(run_id)
        return self.run(
            str(details["run"]["idea"]),
            Mode(str(details["run"]["mode"])),
            str(details["run"]["provider"]),
            run_id,
        )

    def _persist_records(self, run_id: str, stage_key: str, payload: dict[str, Any]) -> None:
        classification = str(payload.get("classification", "inference"))
        if stage_key in {"framing", "intent"}:
            self.database.add_record(
                "decisions",
                {
                    "run_id": run_id,
                    "title": stage_key.title(),
                    "decision_text": str(
                        payload.get(
                            "value", payload.get("problem", payload.get("desired_outcome", ""))
                        )
                    ),
                    "classification": classification,
                    "rationale": "Generated from the supplied intent; validate with evidence.",
                    "source_kind": "user_intent",
                    "source_ref": None,
                },
            )
        if stage_key == "hypothesis":
            for item in payload.get("hypotheses", []):
                self.database.add_record(
                    "hypotheses",
                    {
                        "run_id": run_id,
                        "statement": item["statement"],
                        "risk_level": item["risk"],
                        "evidence_state": "unvalidated",
                        "test_method": item["test"],
                        "success_signal": "Observed improvement in the core outcome",
                    },
                )
        if stage_key == "discovery":
            for question in payload.get("questions", []):
                self.database.add_record(
                    "evidence",
                    {
                        "run_id": run_id,
                        "claim": question,
                        "source_kind": "gap",
                        "source_ref": None,
                        "classification": "pending_validation",
                        "freshness": "not_established",
                        "verified_by": None,
                    },
                )
