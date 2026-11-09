"""Deterministic artifact and integrity sensors."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from tramuri.experience.models import SensorResult
from tramuri.experience.sensors import DemoQualitySensors
from tramuri.sensors.registry import SensorRegistry


def check_payload(payload: dict[str, Any], stage_key: str) -> list[dict[str, Any]]:
    """Run the trusted deterministic sensors for one stage payload."""
    return SensorRegistry().run(payload, stage_key=stage_key)


def check_run(details: dict[str, Any]) -> list[dict[str, str]]:
    checks: list[dict[str, str]] = []
    stages = details.get("stages", [])
    checks.append(
        {
            "key": "completeness",
            "result": "passed"
            if stages and all(item["status"] in {"completed", "skipped"} for item in stages)
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
            else "failed",
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


def check_experience(demo_root: Path, details: dict[str, Any]) -> list[dict[str, Any]]:
    """Project Product Experience sensors into CLI-safe dictionaries."""
    results: list[SensorResult] = DemoQualitySensors().run(demo_root, details)
    return [item.to_dict() for item in results]
