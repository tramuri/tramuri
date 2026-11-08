"""Bounded, redacted context packs for provider session handoff."""

from __future__ import annotations

import hashlib
import re
from typing import Any, cast

from tramuri.core.models import json_dumps

_SECRET_VALUE = re.compile(
    r"(?i)(?:api[_-]?key|password|passwd|token|secret|authorization)\s*[:=]\s*[^\s,;]+"
)
_BEARER = re.compile(r"(?i)bearer\s+[a-z0-9._~+/=-]+")
_OPENAI_KEY = re.compile(r"\bsk-[A-Za-z0-9_-]{8,}\b")
_SENSITIVE_KEY = re.compile(r"(?i)(?:api[_-]?key|password|passwd|token|secret|authorization)")


def _redact(value: Any, *, depth: int = 0) -> Any:
    if depth > 6:
        return "[TRUNCATED]"
    if isinstance(value, dict):
        clean: dict[str, Any] = {}
        for raw_key, raw_value in value.items():
            key = str(raw_key)
            if _SENSITIVE_KEY.fullmatch(key):
                clean[key] = "[REDACTED]"
            else:
                clean[key] = _redact(raw_value, depth=depth + 1)
        return clean
    if isinstance(value, list):
        return [_redact(item, depth=depth + 1) for item in value[:80]]
    if isinstance(value, str):
        text = _SECRET_VALUE.sub("[REDACTED]", value)
        text = _BEARER.sub("[REDACTED]", text)
        text = _OPENAI_KEY.sub("[REDACTED]", text)
        return text[:4_000]
    return value


def _bounded(value: Any, max_bytes: int) -> Any:
    """Trim optional context without changing the required state sections."""
    serialized = json_dumps(value)
    if len(serialized.encode("utf-8")) <= max_bytes:
        return value
    if not isinstance(value, dict):
        return serialized[: max_bytes // 2] + "…"
    result = dict(value)
    for key in ("events", "knowledge", "sensors", "pending", "artifacts"):
        item = result.get(key)
        if isinstance(item, list):
            while item and len(json_dumps(result).encode("utf-8")) > max_bytes:
                item.pop(0)
    if len(json_dumps(result).encode("utf-8")) > max_bytes:
        result["events"] = []
        result["knowledge"] = []
        result["sensors"] = []
    if len(json_dumps(result).encode("utf-8")) > max_bytes:
        run = result.get("run")
        if isinstance(run, dict):
            run["idea"] = str(run.get("idea", ""))[:512]
            result["run"] = run
    if len(json_dumps(result).encode("utf-8")) > max_bytes:
        result = {
            "schema_version": result.get("schema_version", "tramuri.context-pack.v1"),
            "run": result.get("run", {}),
            "completed_stages": result.get("completed_stages", []),
            "pending_stages": result.get("pending_stages", []),
            "next_action": result.get("next_action", "Resume the workflow"),
        }
    return result


def build_context_pack(
    details: dict[str, Any],
    *,
    knowledge: list[dict[str, Any]] | tuple[dict[str, Any], ...] = (),
    sensors: list[dict[str, Any]] | tuple[dict[str, Any], ...] = (),
    max_bytes: int = 32_000,
) -> dict[str, Any]:
    """Build a stable handoff containing state, decisions, evidence and next action."""

    stages = details.get("stages", [])
    completed = [str(item.get("stage_key")) for item in stages if item.get("status") == "completed"]
    pending = [str(item.get("stage_key")) for item in stages if item.get("status") != "completed"]
    base: dict[str, Any] = {
        "schema_version": "tramuri.context-pack.v1",
        "run": {
            key: details.get("run", {}).get(key)
            for key in ("id", "mode", "provider", "status", "idea", "updated_at")
        },
        "completed_stages": completed,
        "pending_stages": pending,
        "decisions": details.get("decisions", []),
        "hypotheses": details.get("hypotheses", []),
        "evidence": details.get("evidence", []),
        "knowledge": list(knowledge),
        "sensors": list(sensors),
        "events": details.get("events", [])[-80:],
        "next_action": "Resume the first pending stage" if pending else "Review verified outcomes",
    }
    safe = _redact(base)
    bounded = _bounded(safe, max(1_024, max_bytes - 100))
    digest = hashlib.sha256(json_dumps(bounded).encode("utf-8")).hexdigest()
    if isinstance(bounded, dict):
        bounded["digest"] = digest
    return cast(dict[str, Any], bounded)
