from __future__ import annotations

from tramuri.core.models import Mode


def test_sensor_results_preserve_unavailable_states(database) -> None:
    run_id = database.create_run("A product", Mode.POC)
    result_id = database.record_sensor_result(
        run_id,
        "intent",
        "browser",
        "advisory",
        "could_not_run",
        {"reason": "optional dependency unavailable"},
    )
    result = database.sensor_results(run_id)[0]
    assert result["id"] == result_id
    assert result["status"] == "could_not_run"
    assert result["status"] != "passed"
