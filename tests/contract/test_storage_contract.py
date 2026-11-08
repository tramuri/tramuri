from tramuri.core.models import Mode, StageStatus
from tramuri.storage.database import Database


def test_database_persists_run_state_and_events(database: Database) -> None:
    run_id = database.create_run("A product idea", Mode.MVP)
    database.transition_stage(run_id, "intent", StageStatus.RUNNING)
    database.add_artifact(run_id, "intent", "pm.intent", {"classification": "confirmed"})
    database.transition_stage(run_id, "intent", StageStatus.COMPLETED)
    details = database.run_details(run_id)
    assert details["run"]["id"] == run_id
    assert details["stages"][0]["status"] == "completed"
    assert details["artifacts"][0]["payload"] == {"classification": "confirmed"}
    assert details["artifacts"][0]["lineage_hash"]
    assert any(event["event_type"] == "artifact.created" for event in details["events"])


def test_database_reuses_project_identity(database: Database) -> None:
    assert database.project_id() == database.project_id()
    assert len(database.list_runs()) == 0


def test_artifact_revisions_and_selective_invalidation(database: Database) -> None:
    run_id = database.create_run("Idea", Mode.MVP)
    database.transition_stage(run_id, "intent", StageStatus.RUNNING)
    database.add_artifact(run_id, "intent", "pm.intent", {"revision": 1})
    database.transition_stage(run_id, "intent", StageStatus.COMPLETED)
    database.transition_stage(run_id, "framing", StageStatus.RUNNING)
    database.add_artifact(run_id, "framing", "pm.framing", {"revision": 1}, ["intent"])
    database.transition_stage(run_id, "framing", StageStatus.COMPLETED)
    database.add_artifact(run_id, "framing", "pm.framing", {"revision": 2}, ["intent"])
    details = database.run_details(run_id)
    assert details["artifacts"][1]["status"] == "superseded"
    assert database.invalidate_from_stage(run_id, "intent") == 1
    refreshed = database.run_details(run_id)
    assert refreshed["artifacts"][-1]["status"] == "stale"
