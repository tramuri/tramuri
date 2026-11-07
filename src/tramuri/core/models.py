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
