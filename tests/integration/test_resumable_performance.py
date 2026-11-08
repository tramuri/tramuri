from __future__ import annotations

from time import perf_counter

from tramuri.core.status import build_status


def test_status_projection_is_bounded_for_local_fixture_scale() -> None:
    details = {
        "run": {"id": "run", "mode": "mvp", "provider": "local-demo", "status": "running"},
        "stages": [
            {"stage_key": f"stage-{index}", "status": "completed" if index < 50 else "pending"}
            for index in range(100)
        ],
        "artifacts": [{"status": "valid"} for _ in range(100)],
        "events": [{"event_type": "stage.completed"} for _ in range(500)],
        "knowledge": [{"classification": "fact"} for _ in range(500)],
        "sensor_results": [{"severity": "advisory", "status": "passed"} for _ in range(100)],
    }
    started = perf_counter()
    view = build_status(details)
    elapsed = perf_counter() - started
    assert view["progress"] == {"completed": 50, "total": 100, "percent": 50}
    assert elapsed < 1.0
