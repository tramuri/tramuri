"""Mode-specific stages and deterministic transition rules."""

from __future__ import annotations

from dataclasses import dataclass

from tramuri.core.contracts import StageDefinition
from tramuri.core.errors import ValidationError
from tramuri.core.models import Mode, StageSpec, StageStatus

MODE_STAGES: dict[Mode, tuple[StageSpec, ...]] = {
    Mode.POC: (
        StageSpec("intent", "product-framing"),
        StageSpec("framing", "product-framing", ("intent",)),
        StageSpec("experiment", "product-discovery", ("framing",)),
        StageSpec("success", "product-analysis", ("experiment",)),
        StageSpec("critical_review", "product-review", ("success",)),
        StageSpec("report", "system", ("critical_review",)),
    ),
    Mode.MVP: (
        StageSpec("intent", "product-framing"),
        StageSpec("framing", "product-framing", ("intent",)),
        StageSpec("discovery", "product-discovery", ("framing",)),
        StageSpec("research", "product-discovery", ("discovery",)),
        StageSpec("hypothesis", "product-discovery", ("research",)),
        StageSpec("shaping", "product-scope", ("hypothesis",)),
        StageSpec("success", "product-analysis", ("shaping",)),
        StageSpec("delivery", "product-scope", ("success",)),
        StageSpec("critical_review", "product-review", ("delivery",)),
        StageSpec("report", "system", ("critical_review",)),
        StageSpec("navigation", "system", ("report",)),
    ),
    Mode.ENTERPRISE: (
        StageSpec("intent", "product-framing"),
        StageSpec("framing", "product-framing", ("intent",)),
        StageSpec("discovery", "product-discovery", ("framing",)),
        StageSpec("research", "product-discovery", ("discovery",)),
        StageSpec("hypothesis", "product-discovery", ("research",)),
        StageSpec("shaping", "product-scope", ("hypothesis",)),
        StageSpec("success", "product-analysis", ("shaping",)),
        StageSpec("governance", "product-analysis", ("success",)),
        StageSpec("traceability", "product-discovery", ("governance",)),
        StageSpec("delivery", "product-scope", ("traceability",)),
        StageSpec("risk_review", "product-review", ("delivery",)),
        StageSpec("critical_review", "product-review", ("risk_review",)),
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
    # A local demo may skip provider research, then a later provider handoff
    # can replace that explicit not-run state with an imported evidence pack.
    StageStatus.SKIPPED: {StageStatus.RUNNING},
}


def stage_specs(mode: Mode) -> tuple[StageSpec, ...]:
    return MODE_STAGES[mode]


def stage_definitions(mode: Mode) -> tuple[StageDefinition, ...]:
    """Expose declarative stage metadata without coupling it to a provider."""
    definitions: list[StageDefinition] = []
    for spec in stage_specs(mode):
        output = f"pm.{spec.key}"
        definitions.append(
            StageDefinition(
                key=spec.key,
                owner=spec.owner,
                dependencies=spec.dependencies,
                inputs=spec.dependencies,
                outputs=(output,),
                sensors=("structured_payload", "classification"),
                completion_criteria=("artifact_persisted", "validation_recorded"),
                max_attempts=3,
            )
        )
    return tuple(definitions)


def validate_transition(old: StageStatus, new: StageStatus) -> Transition:
    if new not in ALLOWED_TRANSITIONS[old]:
        raise ValidationError(f"Invalid stage transition: {old.value} -> {new.value}")
    return Transition(old, new)


def dependencies_satisfied(stage: StageSpec, completed: set[str]) -> bool:
    return all(dependency in completed for dependency in stage.dependencies)
