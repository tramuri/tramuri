from __future__ import annotations

from tramuri.core.context import build_context_pack
from tramuri.core.models import Mode
from tramuri.storage.database import SCHEMA_VERSION


def test_resumable_storage_persists_session_checkpoint_knowledge_and_sensors(database) -> None:
    assert SCHEMA_VERSION == 6
    run_id = database.create_run("Build a useful product", Mode.MVP)

    session_id = database.record_session_event(
        "codex",
        "SessionStart",
        "external-session-1",
        run_id=run_id,
        metadata={"source": "provider", "prompt": "must not be persisted"},
    )
    database.record_session_event(
        "codex",
        "PreCompact",
        "external-session-1",
        run_id=run_id,
        metadata={"reason": "context_limit"},
    )
    checkpoint_id = database.save_checkpoint(
        run_id,
        "framing",
        session_id,
        next_action="Continue discovery",
        state={"completed_stages": ["intent"]},
        pending={"validation": ["target user"]},
        context_digest="abc123",
        idempotency_key="checkpoint-1",
    )
    assert checkpoint_id == database.save_checkpoint(
        run_id,
        "framing",
        session_id,
        next_action="Continue discovery",
        state={"completed_stages": ["intent"]},
        pending={"validation": ["target user"]},
        context_digest="abc123",
        idempotency_key="checkpoint-1",
    )
    checkpoint = database.latest_checkpoint(run_id)
    assert checkpoint is not None
    assert checkpoint["next_action"] == "Continue discovery"

    knowledge_id = database.add_knowledge(
        run_id=run_id,
        tier="project",
        title="Operating assumption",
        content="The first release is local-first.",
        classification="assumption",
        source_kind="decision",
        source_ref="run:" + run_id,
    )
    assert knowledge_id
    entries = database.list_knowledge(run_id=run_id)
    assert entries[0]["classification"] == "assumption"
    assert entries[0]["status"] == "active"

    sensor_id = database.record_sensor_result(
        run_id,
        "framing",
        "structured_payload",
        "blocking",
        "passed",
        {"evidence": "artifact:1"},
        idempotency_key="sensor-1",
    )
    assert sensor_id == database.record_sensor_result(
        run_id,
        "framing",
        "structured_payload",
        "blocking",
        "passed",
        {"evidence": "artifact:1"},
        idempotency_key="sensor-1",
    )
    assert database.sensor_results(run_id)[0]["status"] == "passed"


def test_context_pack_is_bounded_redacted_and_digest_stable(database) -> None:
    run_id = database.create_run("Build a product", Mode.MVP)
    details = database.run_details(run_id)
    pack = build_context_pack(
        details,
        knowledge=[{"title": "Secret", "content": "api_key=sk-live-123456789"}],
        sensors=[{"sensor_key": "integrity", "status": "passed"}],
        max_bytes=8_000,
    )

    assert pack["schema_version"] == "tramuri.context-pack.v1"
    assert pack["digest"]
    assert "sk-live" not in str(pack)
    assert "api_key" not in str(pack).lower()
    assert len(str(pack).encode("utf-8")) <= 8_000
    assert (
        pack["digest"]
        == build_context_pack(
            details,
            knowledge=[{"title": "Secret", "content": "api_key=sk-live-123456789"}],
            sensors=[{"sensor_key": "integrity", "status": "passed"}],
            max_bytes=8_000,
        )["digest"]
    )
