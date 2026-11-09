"""Deterministic product workflow runner.

The runner is intentionally useful without an LLM. A provider can later supply
deeper analysis, but state, evidence labels, retries, and artifacts stay local
and deterministic.
"""

from __future__ import annotations

import re
from collections.abc import Callable
from pathlib import Path
from typing import Any

from tramuri.core.context import build_context_pack
from tramuri.core.errors import ValidationError
from tramuri.core.models import Mode, RunStatus, StageStatus
from tramuri.core.workflow import stage_specs
from tramuri.knowledge.memory import MemoryService
from tramuri.knowledge.service import KnowledgeService
from tramuri.modules.pm.intent import IntentProfile, parse_intent
from tramuri.modules.pm.research import load_research_pack, not_run_payload
from tramuri.modules.pm.stages import MODE_PROFILES, capability_name, specialist_name
from tramuri.sensors.registry import SensorRegistry
from tramuri.storage.database import Database

ProgressCallback = Callable[[str, str, str], None]


def _title(idea: str) -> str:
    profile = parse_intent(idea)
    cleaned = re.sub(r"\s+", " ", profile.title.strip()).strip(" .")
    return cleaned[:100] or "Unnamed product idea"


def _payloads(
    idea: str, mode: Mode, profile: IntentProfile | None = None
) -> dict[str, dict[str, Any]]:
    profile = profile or parse_intent(idea)
    title = profile.title
    mode_profile = MODE_PROFILES[mode]
    base: dict[str, Any] = {
        "product_name": title,
        "mode": mode.value,
        "mode_label": mode_profile["label"],
        "focus": mode_profile["focus"],
        "idea": idea.strip(),
        "classification": "inference",
        "archetype": profile.archetype,
        "archetype_confidence": profile.archetype_confidence,
        "execution": "deterministic-local",
    }
    intent_profile = parse_intent(idea)
    if intent_profile.archetype == "learning":
        target_users = list(intent_profile.audience)
        problem = "Learners and independent creators need a clear way to sell and complete outcome-led courses."
        value = "Connect course creators with learners through purchase, learning progress and proof of completion."
        in_scope = [
            "Course catalog",
            "Course purchase",
            "Learner progress",
            "Creator administration",
        ]
        out_of_scope = ["Accreditation", "Live teaching infrastructure", "Enterprise billing"]
    elif intent_profile.archetype == "appointment":
        target_users = list(intent_profile.audience)
        problem = "People who need a service need a reliable way to find availability and complete a booking."
        value = "Help a customer move from service selection to a confirmed appointment with clear availability."
        in_scope = ["Service selection", "Availability", "Booking confirmation"]
        out_of_scope = [
            "Advanced administration",
            "Unvalidated integrations",
            "Operational scale work",
        ]
    else:
        target_users = list(intent_profile.audience)
        problem = "The supplied intent needs a validated problem frame before implementation breadth is chosen."
        value = f"A focused first workflow for the outcome expressed in: {intent_profile.intent}"
        in_scope = ["Intent capture", "Core outcome flow", "Confirmation or result state"]
        out_of_scope = ["Unvalidated integrations", "Operational scale work"]
    base.update(
        {
            "intent_text": intent_profile.intent,
            "desired_outcome": intent_profile.desired_outcome,
            "constraints": list(intent_profile.constraints),
        }
    )
    return {
        "intent": base
        | {
            "desired_outcome": intent_profile.desired_outcome,
            "intent_profile": intent_profile.to_dict(),
            "classification": "confirmed",
        },
        "framing": base
        | {
            "problem": problem,
            "target_users": target_users,
            "value_proposition": value,
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
            "hypotheses": list(
                [
                    {
                        "statement": f"The primary audience will use {title} to reach the stated outcome.",
                        "risk": "high",
                        "test": "Observe completion of the smallest end-to-end task.",
                    },
                    {
                        "statement": "A focused first workflow will be more useful than a broad feature set.",
                        "risk": "high",
                        "test": "Interview or observe representative users.",
                    },
                ]
                + [
                    {
                        "statement": item,
                        "risk": "medium",
                        "test": "Design a falsifiable experiment with representative users.",
                    }
                    for item in intent_profile.hypotheses
                ]
            ),
            "classification": "hypothesis",
        },
        "research": not_run_payload("local-demo"),
        "experiment": base
        | {
            "main_hypothesis": "A small end-to-end experiment can establish whether the core outcome is feasible.",
            "question": "Can the user reach the intended outcome with the smallest useful flow?",
            "experiment": "Prototype the critical path and test it with representative scenarios.",
            "classification": "hypothesis",
        },
        "shaping": base
        | {
            "in_scope": in_scope,
            "out_of_scope": out_of_scope,
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
                "Market Research",
                "Buying Motivations",
                "Positioning",
                "Hypotheses",
                "Scope",
                "Product UX",
                "Success",
                "Decisions",
                "Risks",
            ],
            "classification": "generated",
        },
        "navigation": base
        | {
            "prototype": "Offline multi-surface Product Experience scoped to the defined journey.",
            "surfaces": ["marketing", "customer", "admin"],
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
        self,
        idea: str,
        mode: Mode,
        provider: str = "local-demo",
        run_id: str | None = None,
        research_file: Path | None = None,
    ) -> str:
        if not idea.strip():
            raise ValidationError(
                "A product intent is required before Tramuri can create a product workflow."
            )
        run_id = run_id or self.database.create_run(idea, mode, provider)
        session_id = self.database.record_session_event(
            provider,
            "SessionStart",
            f"tramuri-run-{run_id}",
            run_id=run_id,
            metadata={"source": "tramuri", "phase": "product"},
        )
        if not self.database.list_knowledge(run_id=run_id):
            KnowledgeService(self.database).ingest_project_documents(
                self.database.project_dir, run_id=run_id
            )
            self.database.add_knowledge(
                run_id=run_id,
                tier="project",
                title="Product intent source",
                content=idea.strip(),
                classification="fact",
                source_kind="user_intent",
                source_ref=f"run:{run_id}",
            )
            self.database.add_knowledge(
                run_id=run_id,
                tier="methodology",
                title="Autonomous progression policy",
                content=(
                    "Use reversible low-impact operational defaults and record them. "
                    "Ask only when authorization, security, privacy, legal or high-impact "
                    "decisions cannot be safely defaulted."
                ),
                classification="decision",
                source_kind="tramuri_policy",
                source_ref="constitution:VII",
            )
        details = self.database.run_details(run_id)
        relevant_knowledge = KnowledgeService(self.database).retrieve(idea, run_id=run_id)
        if relevant_knowledge:
            self.database.event(
                run_id,
                "knowledge.retrieved",
                {"count": len(relevant_knowledge), "source": "local"},
                idempotency_key=f"knowledge-retrieved:{run_id}",
            )
        intent_profile = parse_intent(str(details["run"]["idea"]))
        payloads = _payloads(
            str(details["run"]["idea"]), Mode(str(details["run"]["mode"])), intent_profile
        )
        if research_file is not None:
            payloads["research"] = load_research_pack(research_file).payload
        sensor_registry = SensorRegistry()
        for spec in stage_specs(mode):
            stage = self.database.stage(run_id, spec.key)
            if StageStatus(stage["status"]) == StageStatus.COMPLETED:
                continue
            if spec.key == "research" and research_file is None and provider == "local-demo":
                self.database.transition_stage(
                    run_id,
                    spec.key,
                    StageStatus.SKIPPED,
                    idempotency_key=f"{run_id}:research:not-run:skipped",
                )
                self.database.event(
                    run_id,
                    "research.not_run",
                    {"reason": "local-demo", "status": "not_run"},
                    idempotency_key=f"{run_id}:research:not-run:event",
                )
                self._notify(spec.key, "skipped", capability_name(spec.owner))
                self._save_checkpoint(run_id, spec.key, session_id)
                continue
            if spec.key == "research" and research_file is None:
                self.database.transition_stage(run_id, spec.key, StageStatus.RUNNING)
                self.database.event(
                    run_id,
                    "research.required",
                    {
                        "status": "blocked",
                        "next_action": "Supply a provider-produced research JSON pack and resume",
                    },
                )
                self.database.transition_stage(
                    run_id,
                    spec.key,
                    StageStatus.BLOCKED,
                    "Provider research pack is required before product shaping.",
                )
                self.database.complete_run(run_id, RunStatus.BLOCKED)
                self._save_checkpoint(
                    run_id,
                    spec.key,
                    session_id,
                    next_action="Import a validated research pack with tramuri product research",
                )
                return run_id
            if not self.database.completed_stages(run_id).issuperset(spec.dependencies):
                self.database.complete_run(run_id, RunStatus.BLOCKED)
                return run_id
            for attempt in range(1, 4):
                agent_id = specialist_name(spec.key, spec.owner)
                try:
                    self.database.event(
                        run_id,
                        "stage.planned",
                        {
                            "stage": spec.key,
                            "owner": spec.owner,
                            "agent_id": agent_id,
                            "provider": provider,
                        },
                        idempotency_key=f"{run_id}:{spec.key}:{attempt}:planned",
                    )
                    self.database.event(
                        run_id,
                        "stage.delegated",
                        {
                            "stage": spec.key,
                            "owner": spec.owner,
                            "agent_id": agent_id,
                            "provider": provider,
                            "execution_mode": "deterministic-local"
                            if provider == "local-demo"
                            else "provider-handoff",
                            "agent_status": "not_invoked",
                            "limitation": "Core runner records deterministic delegation; provider-native agent invocation remains in the assistant session.",
                        },
                        idempotency_key=f"{run_id}:{spec.key}:{attempt}:delegated",
                    )
                    self.database.transition_stage(
                        run_id,
                        spec.key,
                        StageStatus.RUNNING,
                        idempotency_key=f"{run_id}:{spec.key}:{attempt}:running",
                    )
                    self._notify(spec.key, "running", capability_name(spec.owner))
                    payload = payloads.get(
                        spec.key, {"stage": spec.key, "classification": "generated"}
                    )
                    sensor_results = sensor_registry.run(payload, stage_key=spec.key)
                    for sensor in sensor_results:
                        self.database.record_sensor_result(
                            run_id,
                            spec.key,
                            str(sensor["sensor_key"]),
                            str(sensor["severity"]),
                            str(sensor["status"]),
                            dict(sensor.get("evidence", {})),
                            idempotency_key=f"{run_id}:{spec.key}:{attempt}:sensor:{sensor['sensor_key']}",
                        )
                    blocked = next(
                        (
                            item
                            for item in sensor_results
                            if item["severity"] == "blocking"
                            and item["status"] in {"failed", "blocked", "could_not_run"}
                        ),
                        None,
                    )
                    if blocked:
                        raise ValueError(
                            f"Blocking sensor failed: {blocked['sensor_key']} ({blocked['status']})"
                        )
                    self.database.event(
                        run_id,
                        "stage.executed",
                        {
                            "stage": spec.key,
                            "agent_id": agent_id,
                            "owner": spec.owner,
                            "provider": provider,
                            "execution_mode": "deterministic-local"
                            if provider == "local-demo"
                            else "provider-handoff",
                            "agent_status": "not_invoked",
                        },
                        idempotency_key=f"{run_id}:{spec.key}:{attempt}:executed",
                    )
                    artifact_id = self.database.add_artifact(
                        run_id,
                        spec.key,
                        "pm.market_research" if spec.key == "research" else f"pm.{spec.key}",
                        payload,
                        list(spec.dependencies),
                        idempotency_key=f"{run_id}:{spec.key}:{attempt}:artifact",
                    )
                    self.database.event(
                        run_id,
                        "agent.invocation",
                        {
                            "role": spec.owner,
                            "agent_id": agent_id,
                            "provider": provider,
                            "session_id": session_id,
                            "status": "not_invoked",
                            "mode": "deterministic-local"
                            if provider == "local-demo"
                            else "provider-handoff",
                            "input_artifacts": list(spec.dependencies),
                            "output_artifact": artifact_id,
                            "limitations": [
                                "The core runner does not invoke a remote subagent; the installed provider skill may delegate within the assistant session."
                            ],
                        },
                        idempotency_key=f"{run_id}:{spec.key}:{attempt}:agent-invocation",
                    )
                    self._persist_records(run_id, spec.key, payload)
                    self.database.validation(
                        run_id,
                        str(stage["id"]),
                        artifact_id,
                        f"pm.{spec.key}.integrity",
                        "passed",
                        "Structured payload persisted with classification",
                        idempotency_key=f"{run_id}:{spec.key}:{attempt}:validation",
                    )
                    self.database.event(
                        run_id,
                        "stage.evaluated",
                        {"stage": spec.key, "result": "passed"},
                        idempotency_key=f"{run_id}:{spec.key}:{attempt}:evaluated",
                    )
                    self.database.transition_stage(
                        run_id,
                        spec.key,
                        StageStatus.COMPLETED,
                        idempotency_key=f"{run_id}:{spec.key}:{attempt}:completed",
                    )
                    self.database.event(
                        run_id,
                        "stage.verified",
                        {"stage": spec.key, "artifact_id": artifact_id},
                        idempotency_key=f"{run_id}:{spec.key}:{attempt}:verified",
                    )
                    self._notify(spec.key, "completed", capability_name(spec.owner))
                    self._save_checkpoint(run_id, spec.key, session_id)
                    break
                except Exception as exc:
                    self.database.transition_stage(run_id, spec.key, StageStatus.FAILED, str(exc))
                    if attempt == 3:
                        self.database.complete_run(run_id, RunStatus.FAILED)
                        self._notify(spec.key, "failed", capability_name(spec.owner))
                        raise
                    self.database.event(
                        run_id,
                        "stage.retry",
                        {"stage": spec.key, "attempt": attempt + 1, "error": str(exc)},
                    )
        self.database.complete_run(run_id, RunStatus.COMPLETED)
        self._save_checkpoint(
            run_id, None, session_id, next_action="Review the report and product demo"
        )
        self.database.record_session_event(
            provider,
            "SessionEnd",
            f"tramuri-run-{run_id}",
            run_id=run_id,
            metadata={"source": "tramuri", "status": "completed"},
        )
        return run_id

    def resume(self, run_id: str, research_file: Path | None = None) -> str:
        details = self.database.run_details(run_id)
        checkpoint = self.database.latest_checkpoint(run_id)
        self.database.record_session_event(
            str(details["run"]["provider"]),
            "SessionStart",
            f"tramuri-run-{run_id}",
            run_id=run_id,
            metadata={"source": "tramuri", "phase": "resume"},
        )
        self.database.resume_run(
            run_id,
            idempotency_key=(f"resume:{run_id}:{checkpoint['id']}" if checkpoint else None),
        )
        return self.run(
            str(details["run"]["idea"]),
            Mode(str(details["run"]["mode"])),
            str(details["run"]["provider"]),
            run_id,
            research_file,
        )

    def _save_checkpoint(
        self,
        run_id: str,
        stage_key: str | None,
        session_id: str,
        *,
        next_action: str | None = None,
    ) -> None:
        details = self.database.run_details(run_id)
        settled = [
            str(item["stage_key"])
            for item in details["stages"]
            if item["status"] in {StageStatus.COMPLETED.value, StageStatus.SKIPPED.value}
        ]
        pending = [
            str(item["stage_key"])
            for item in details["stages"]
            if item["status"] not in {StageStatus.COMPLETED.value, StageStatus.SKIPPED.value}
        ]
        context = build_context_pack(
            details,
            knowledge=details.get("knowledge", []),
            sensors=details.get("sensor_results", []),
        )
        checkpoint_id = self.database.save_checkpoint(
            run_id,
            stage_key,
            session_id,
            next_action=next_action
            or (f"Continue {pending[0]}" if pending else "Review verified outcomes"),
            state={"settled_stages": settled, "run_status": details["run"]["status"]},
            pending={"stages": pending},
            context_digest=str(context["digest"]),
            idempotency_key=f"{run_id}:checkpoint:{stage_key or 'final'}:{len(settled)}",
        )
        try:
            MemoryService(self.database).show(run_id)
        except (OSError, ValidationError) as exc:
            self.database.event(
                run_id,
                "memory.projection.failed",
                {"checkpoint_id": checkpoint_id, "error_type": type(exc).__name__},
                idempotency_key=f"memory-projection-failed:{checkpoint_id}",
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
        if stage_key == "research" and payload.get("status") == "imported":
            source_by_id = {str(source.get("id")): source for source in payload.get("sources", [])}
            for collection in (
                "competitors",
                "market_signals",
                "buying_motivations",
                "build_buy_partner",
                "positioning",
            ):
                for item in payload.get(collection, []):
                    statement = str(
                        item.get("statement")
                        or item.get("signal")
                        or item.get("motivation")
                        or item.get("positioning")
                        or item.get("option")
                        or item.get("name")
                        or "Research claim"
                    )
                    refs = [source_by_id[str(item_id)]["url"] for item_id in item["source_ids"]]
                    self.database.add_record(
                        "evidence",
                        {
                            "run_id": run_id,
                            "claim": statement,
                            "source_kind": "market_research",
                            "source_ref": ",".join(refs),
                            "classification": "evidence",
                            "freshness": str(payload["researched_at"]),
                            "verified_by": "provider-research-pack",
                        },
                    )
