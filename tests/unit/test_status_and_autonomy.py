from __future__ import annotations

from tramuri.core.autonomy import decide_gap
from tramuri.core.models import Mode
from tramuri.core.status import build_status


def test_status_is_actionable_and_truthful(database) -> None:
    run_id = database.create_run("Create a scheduling product", Mode.MVP)
    details = database.run_details(run_id)
    view = build_status(details)

    assert view["run_id"] == run_id
    assert view["semantic_status"] == "running"
    assert view["progress"]["total"] == len(details["stages"])
    assert view["current_stage"] == "intent"
    assert view["next_action"]
    assert view["counts"]["completed_stages"] == 0


def test_status_surfaces_blocking_sensor(database) -> None:
    run_id = database.create_run("Create a scheduling product", Mode.MVP)
    database.record_sensor_result(
        run_id,
        "intent",
        "classification",
        "blocking",
        "failed",
        {"reason": "missing"},
    )
    view = build_status(database.run_details(run_id))
    assert view["semantic_status"] == "blocked"
    assert view["gates"]["blocking_failures"] == 1


def test_autonomy_uses_safe_defaults_and_only_gates_high_impact_choices() -> None:
    proceed = decide_gap("Use a narrow MVP scope", impact="low")
    assert proceed.action == "proceed"
    assert proceed.requires_user is False

    gate = decide_gap(None, impact="high", requires_authorization=True)
    assert gate.action == "gate"
    assert gate.requires_user is True
    assert gate.reason
