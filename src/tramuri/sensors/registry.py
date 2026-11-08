"""Trusted deterministic sensor registry.

Sensors are Python callables registered by the Tramuri package. They never
execute commands supplied by a repository, document or generated artifact.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

SensorFunction = Callable[[dict[str, Any]], tuple[str, dict[str, Any]]]


@dataclass(frozen=True, slots=True)
class SensorDefinition:
    key: str
    severity: str
    applies_to: tuple[str, ...]
    function: SensorFunction
    command: str | None = None


def _structured(payload: dict[str, Any]) -> tuple[str, dict[str, Any]]:
    passed = bool(payload)
    return ("passed" if passed else "failed", {"non_empty": passed})


def _classification(payload: dict[str, Any]) -> tuple[str, dict[str, Any]]:
    value = payload.get("classification")
    passed = isinstance(value, str) and bool(value.strip())
    return ("passed" if passed else "failed", {"classification_present": passed})


class SensorRegistry:
    """Run only trusted, stage-scoped deterministic checks."""

    def __init__(self) -> None:
        self._definitions = (
            SensorDefinition("structured_payload", "blocking", (), _structured),
            SensorDefinition("classification", "blocking", (), _classification),
        )

    def definitions(self) -> tuple[SensorDefinition, ...]:
        return self._definitions

    def run(self, payload: dict[str, Any], *, stage_key: str) -> list[dict[str, Any]]:
        results: list[dict[str, Any]] = []
        for definition in self._definitions:
            if definition.applies_to and stage_key not in definition.applies_to:
                continue
            try:
                status, evidence = definition.function(payload)
            except Exception as exc:  # pragma: no cover - defensive sensor boundary
                status, evidence = "could_not_run", {"error": type(exc).__name__}
            results.append(
                {
                    "sensor_key": definition.key,
                    "severity": definition.severity,
                    "status": status,
                    "evidence": evidence,
                    "stage_key": stage_key,
                }
            )
        return results
