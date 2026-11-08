from __future__ import annotations

from tramuri.hooks.runtime import evaluate
from tramuri.storage.database import Database


def test_provider_lifecycle_events_are_safe_and_replayable(tmp_path) -> None:
    database = Database(tmp_path)
    payload = {
        "hook_event_name": "SessionStart",
        "session_id": "provider-session",
        "cwd": str(tmp_path),
        "prompt": "password=should never be stored",
    }
    evaluate(payload, "codex")
    evaluate(payload, "codex")
    with database.connection() as connection:
        rows = connection.execute("SELECT * FROM session_events").fetchall()
        metadata = connection.execute("SELECT metadata_json FROM sessions").fetchone()[0]
    assert len(rows) == 2
    assert "password" not in metadata.lower()
    assert "should never" not in metadata.lower()
