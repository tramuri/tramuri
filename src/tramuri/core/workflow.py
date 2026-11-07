"""Mode-specific stages and deterministic transition rules."""

from __future__ import annotations

from dataclasses import dataclass

from tramuri.core.errors import ValidationError
from tramuri.core.models import Mode, StageSpec, StageStatus

MODE_STAGES: dict[Mode, tuple[StageSpec, ...]] = {
    Mode.POC: (
        StageSpec("intent", "product_strategist"),
        StageSpec("framing", "product_strategist", ("intent",)),
        StageSpec("experiment", "discovery_specialist", ("framing",)),
        StageSpec("success", "product_analyst", ("experiment",)),
        StageSpec("critical_review", "critical_reviewer", ("success",)),
        StageSpec("report", "system", ("critical_review",)),
    ),
    Mode.MVP: (
        StageSpec("intent", "product_strategist"),
        StageSpec("framing", "product_strategist", ("intent",)),
        StageSpec("discovery", "discovery_specialist", ("framing",)),
        StageSpec("hypothesis", "discovery_specialist", ("discovery",)),
        StageSpec("shaping", "scope_specialist", ("hypothesis",)),
        StageSpec("success", "product_analyst", ("shaping",)),
        StageSpec("delivery", "scope_specialist", ("success",)),
        StageSpec("critical_review", "critical_reviewer", ("delivery",)),
        StageSpec("report", "system", ("critical_review",)),
        StageSpec("navigation", "system", ("report",)),
    ),
    Mode.ENTERPRISE: (
        StageSpec("intent", "product_strategist"),
        StageSpec("framing", "product_strategist", ("intent",)),
        StageSpec("discovery", "discovery_specialist", ("framing",)),
        StageSpec("hypothesis", "discovery_specialist", ("discovery",)),
        StageSpec("shaping", "scope_specialist", ("hypothesis",)),
        StageSpec("success", "product_analyst", ("shaping",)),
        StageSpec("governance", "product_analyst", ("success",)),
        StageSpec("traceability", "discovery_specialist", ("governance",)),
        StageSpec("delivery", "scope_specialist", ("traceability",)),
        StageSpec("risk_review", "critical_reviewer", ("delivery",)),
        StageSpec("critical_review", "critical_reviewer", ("risk_review",)),
        StageSpec("report", "system", ("critical_review",)),
        StageSpec("navigation", "system", ("report",)),
    ),
}


@dataclass(frozen=True, slots=True)
class Transition:
    old: StageStatus
    new: StageStatus


ALLOWED_TRANSITIONS = {
    StageStatus.PENDING: {StageStatus.RUNNING, StageStatus.SKIPPED},
    StageStatus.RUNNING: {
        StageStatus.COMPLETED,
        StageStatus.FAILED,
        StageStatus.BLOCKED,
    },
    StageStatus.FAILED: {StageStatus.RUNNING, StageStatus.BLOCKED},
    StageStatus.BLOCKED: {StageStatus.RUNNING},
    StageStatus.COMPLETED: set(),
    StageStatus.SKIPPED: set(),
}


def stage_specs(mode: Mode) -> tuple[StageSpec, ...]:
    return MODE_STAGES[mode]


def validate_transition(old: StageStatus, new: StageStatus) -> Transition:
    if new not in ALLOWED_TRANSITIONS[old]:
        raise ValidationError(f"Invalid stage transition: {old.value} -> {new.value}")
    return Transition(old, new)


def dependencies_satisfied(stage: StageSpec, completed: set[str]) -> bool:
    return all(dependency in completed for dependency in stage.dependencies)
