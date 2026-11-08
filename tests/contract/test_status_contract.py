from __future__ import annotations

from tramuri.core.models import Mode, RunStatus
from tramuri.core.status import build_status


def test_empty_project_status_is_ready(database) -> None:
    view = build_status(None)
    assert view["semantic_status"] == "ready"
    assert view["next_action"].startswith("Run tramuri product")


def test_status_projection_covers_stale_and_completed_facts(database) -> None:
    run_id = database.create_run("A product", Mode.POC)
    details = database.run_details(run_id)
    details["run"]["status"] = RunStatus.COMPLETED.value
    details["artifacts"] = [{"status": "stale"}]
    view = build_status(details)
    assert view["semantic_status"] == "stale"
    assert view["progress"]["percent"] == 0


def test_status_projection_surfaces_waiting_and_failed_runs(database) -> None:
    run_id = database.create_run("A product", Mode.POC)
    details = database.run_details(run_id)
    for stage in details["stages"]:
        stage["status"] = "completed"
    details["run"]["status"] = RunStatus.RUNNING.value
    details["knowledge"] = [{"classification": "pending_validation"}]
    assert build_status(details)["semantic_status"] == "waiting"

    details["run"]["status"] = RunStatus.FAILED.value
    assert build_status(details)["semantic_status"] == "failed"
