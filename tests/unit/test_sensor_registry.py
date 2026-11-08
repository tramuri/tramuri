from __future__ import annotations

from tramuri.sensors.registry import SensorRegistry


def test_sensor_registry_reports_deterministic_failures() -> None:
    results = SensorRegistry().run(
        {"stage": "framing", "classification": ""},
        stage_key="framing",
    )
    classification = next(item for item in results if item["sensor_key"] == "classification")
    assert classification["status"] == "failed"
    assert classification["severity"] == "blocking"


def test_sensor_registry_does_not_execute_arbitrary_commands() -> None:
    registry = SensorRegistry()
    assert all(item.command is None for item in registry.definitions())
