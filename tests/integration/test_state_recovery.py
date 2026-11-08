from tramuri.core.models import Mode, StageStatus
from tramuri.modules.pm.runner import PMRunner


def test_duplicate_event_key_is_idempotent(database) -> None:
    run_id = database.create_run("idea", Mode.POC)
    database.event(run_id, "stage.planned", {"stage": "intent"}, idempotency_key="same")
    database.event(run_id, "stage.planned", {"stage": "intent"}, idempotency_key="same")
    details = database.run_details(run_id)
    assert sum(event["event_type"] == "stage.planned" for event in details["events"]) == 1


def test_duplicate_artifact_key_reuses_revision(database) -> None:
    run_id = database.create_run("idea", Mode.POC)
    first = database.add_artifact(
        run_id, "intent", "pm.intent", {"value": 1}, idempotency_key="artifact"
    )
    second = database.add_artifact(
        run_id, "intent", "pm.intent", {"value": 2}, idempotency_key="artifact"
    )
    assert first == second
    assert len(database.run_details(run_id)["artifacts"]) == 1


def test_duplicate_validation_key_is_idempotent(database) -> None:
    run_id = database.create_run("idea", Mode.POC)
    database.validation(run_id, None, None, "check", "passed", "ok", "validation")
    database.validation(run_id, None, None, "check", "passed", "ok", "validation")
    assert len(database.run_details(run_id)["validations"]) == 1


def test_transitive_invalidation(database) -> None:
    run_id = database.create_run("idea", Mode.MVP)
    for key in ("intent", "framing", "discovery"):
        database.transition_stage(run_id, key, StageStatus.RUNNING)
        database.add_artifact(
            run_id,
            key,
            f"pm.{key}",
            {"stage": key},
            [] if key == "intent" else [("intent" if key == "framing" else "framing")],
        )
        database.transition_stage(run_id, key, StageStatus.COMPLETED)
    assert database.invalidate_from_stage(run_id, "intent") == 2


def test_completed_run_resume_does_not_rebuild_validated_stages(database) -> None:
    run_id = PMRunner(database).run("idea", Mode.POC)
    before = database.run_details(run_id)
    PMRunner(database).resume(run_id)
    after = database.run_details(run_id)
    assert len(after["artifacts"]) == len(before["artifacts"])
    assert after["run"]["status"] == "completed"
