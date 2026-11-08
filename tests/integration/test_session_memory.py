from __future__ import annotations

from tramuri.core.models import Mode


def test_checkpoint_supersession_and_replay_are_deterministic(database) -> None:
    run_id = database.create_run("A product", Mode.POC)
    first = database.save_checkpoint(
        run_id,
        "intent",
        None,
        next_action="Continue framing",
        state={"stage": "intent"},
        pending={"stage": "framing"},
        idempotency_key="same-checkpoint",
    )
    assert first == database.save_checkpoint(
        run_id,
        "intent",
        None,
        next_action="Continue framing",
        state={"stage": "intent"},
        pending={"stage": "framing"},
        idempotency_key="same-checkpoint",
    )
    second = database.save_checkpoint(
        run_id,
        "framing",
        None,
        next_action="Continue discovery",
        state={"stage": "framing"},
        pending={"stage": "discovery"},
    )
    assert database.latest_checkpoint(run_id)["id"] == second
    assert len(database.run_details(run_id)["checkpoints"]) == 2
