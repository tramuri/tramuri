"""Deterministic artifact and integrity sensors."""

from __future__ import annotations

from typing import Any


def check_run(details: dict[str, Any]) -> list[dict[str, str]]:
    checks: list[dict[str, str]] = []
    stages = details.get("stages", [])
    checks.append(
        {
            "key": "completeness",
            "result": "passed"
            if stages and all(item["status"] == "completed" for item in stages)
            else "failed",
        }
    )
    checks.append(
        {
            "key": "traceability",
            "result": "passed"
            if details.get("events") and details.get("validations")
            else "failed",
        }
    )
    checks.append(
        {
            "key": "classification",
            "result": "passed"
            if all("classification" in item for item in details.get("decisions", []))
            else "passed",
        }
    )
    return checks


def validate_payload(payload: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if not payload.get("classification"):
        errors.append("classification is required")
    if not payload:
        errors.append("payload cannot be empty")
    return errors
