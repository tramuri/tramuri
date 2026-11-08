"""Shared domain contracts for Tramuri."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from enum import StrEnum
from typing import Any
from uuid import uuid4


def now_iso() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def new_id() -> str:
    return str(uuid4())


class Mode(StrEnum):
    POC = "poc"
    MVP = "mvp"
    ENTERPRISE = "enterprise"


class RunStatus(StrEnum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    BLOCKED = "blocked"
    FAILED = "failed"
    CANCELLED = "cancelled"


class StageStatus(StrEnum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    BLOCKED = "blocked"
    FAILED = "failed"
    SKIPPED = "skipped"


class ArtifactStatus(StrEnum):
    DRAFT = "draft"
    VALID = "valid"
    STALE = "stale"
    SUPERSEDED = "superseded"


class CapabilityStatus(StrEnum):
    VERIFIED = "verified"
    AVAILABLE = "available"
    MISSING = "missing"
    UNSUPPORTED = "unsupported"
    UNKNOWN = "unknown"


class SessionStatus(StrEnum):
    STARTED = "started"
    ACTIVE = "active"
    ENDED = "ended"
    COMPACTED = "compacted"
    UNKNOWN = "unknown"


class CheckpointStatus(StrEnum):
    VALID = "valid"
    SUPERSEDED = "superseded"
    INVALID = "invalid"


class KnowledgeTier(StrEnum):
    METHODOLOGY = "methodology"
    PROJECT = "project"


class KnowledgeClassification(StrEnum):
    FACT = "fact"
    EVIDENCE = "evidence"
    INFERENCE = "inference"
    HYPOTHESIS = "hypothesis"
    ASSUMPTION = "assumption"
    DECISION = "decision"
    PENDING_VALIDATION = "pending_validation"


class KnowledgeFreshness(StrEnum):
    CURRENT = "current"
    STALE = "stale"
    UNKNOWN = "unknown"


class KnowledgeStatus(StrEnum):
    ACTIVE = "active"
    INVALIDATED = "invalidated"
    UNAVAILABLE = "unavailable"


class SensorSeverity(StrEnum):
    ADVISORY = "advisory"
    BLOCKING = "blocking"


class SensorStatus(StrEnum):
    PASSED = "passed"
    FAILED = "failed"
    BLOCKED = "blocked"
    NOT_RUN = "not_run"
    COULD_NOT_RUN = "could_not_run"


@dataclass(slots=True)
class Capability:
    provider: str
    key: str
    status: CapabilityStatus
    source_path: str | None = None
    observed_version: str | None = None
    details: str = ""

    def to_dict(self) -> dict[str, Any]:
        return asdict(self) | {"status": self.status.value}


@dataclass(slots=True)
class InstallationComponent:
    key: str
    relative_path: str
    content: str
    optional: bool = False
    description: str = ""
    merge_json: bool = False
    merge_text: bool = False


@dataclass(slots=True)
class InstallationPlan:
    provider: str
    components: list[InstallationComponent] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)


@dataclass(slots=True)
class StageSpec:
    key: str
    owner: str
    dependencies: tuple[str, ...] = ()


def json_dumps(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
