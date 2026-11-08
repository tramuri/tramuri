"""Provider-neutral contracts shared by workflows and harnesses."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import StrEnum
from typing import Any


class HookAction(StrEnum):
    """Neutral result of evaluating a host hook event."""

    OBSERVE = "observe"
    CONTEXT = "context"
    ALLOW = "allow"
    BLOCK = "block"


@dataclass(frozen=True, slots=True)
class HookEvaluation:
    """Normalized event data; no provider-specific output is allowed here."""

    event: str
    prompt_class: str | None = None
    tool_name: str | None = None
    is_mutating: bool = False
    is_bootstrap: bool = False
    has_completed_run: bool = False
    session_id: str | None = None
    requires_workflow: bool = False


@dataclass(frozen=True, slots=True)
class HookDecision:
    """Core decision rendered by a provider harness."""

    action: HookAction
    event: str
    requires_workflow: bool = False
    reason: str | None = None
    context: str | None = None
    state_snapshot: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True, slots=True)
class StageDefinition:
    """Declarative metadata for a workflow stage."""

    key: str
    owner: str
    dependencies: tuple[str, ...] = ()
    inputs: tuple[str, ...] = ()
    outputs: tuple[str, ...] = ()
    sensors: tuple[str, ...] = ()
    completion_criteria: tuple[str, ...] = ()
    max_attempts: int = 3
