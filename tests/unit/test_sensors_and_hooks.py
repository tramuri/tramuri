import json
from io import StringIO
from unittest.mock import patch

from tramuri.core.models import Mode
from tramuri.hooks import checkpoint
from tramuri.hooks.runtime import evaluate
from tramuri.sensors.checks import validate_payload
from tramuri.storage.database import Database


def test_payload_sensor_requires_classification() -> None:
    assert validate_payload({"value": 1}) == ["classification is required"]
    assert validate_payload({"classification": "hypothesis"}) == []


def test_checkpoint_hook_returns_json_only() -> None:
    output = StringIO()
    with (
        patch("sys.stdin", StringIO(json.dumps({"hook_event_name": "PostToolUse"}))),
        patch("sys.stdout", output),
    ):
        checkpoint.main()
    assert json.loads(output.getvalue())["event"] == "PostToolUse"


def test_runtime_hook_blocks_product_mutation_until_run_completes(tmp_path) -> None:
    database = Database(tmp_path)
    prompt = {
        "hook_event_name": "UserPromptSubmit",
        "session_id": "session-1",
        "cwd": str(tmp_path),
        "prompt": "Quero criar um SaaS para agendamento de serviços",
    }
    context = evaluate(prompt, "codex")
    assert "additionalContext" in context["hookSpecificOutput"]

    blocked = evaluate(
        {
            "hook_event_name": "PreToolUse",
            "session_id": "session-1",
            "cwd": str(tmp_path),
            "tool_name": "Bash",
            "tool_input": {"command": "echo change > product.md"},
        },
        "codex",
    )
    assert blocked["decision"] == "block"

    run_id = database.create_run("Scheduling SaaS", Mode.POC)
    database.complete_run(run_id)
    assert (
        evaluate(
            {
                "hook_event_name": "PreToolUse",
                "session_id": "session-1",
                "cwd": str(tmp_path),
                "tool_name": "Bash",
                "tool_input": {"command": "echo change > product.md"},
            },
            "codex",
        )
        == {}
    )


def test_runtime_hook_uses_provider_specific_denial_shapes(tmp_path) -> None:
    Database(tmp_path)
    prompt = {
        "hook_event_name": "BeforeAgent",
        "session_id": "session-2",
        "cwd": str(tmp_path),
        "prompt": "Defina o MVP deste produto",
    }
    evaluate(prompt, "gemini")
    payload = {
        "hook_event_name": "BeforeTool",
        "session_id": "session-2",
        "cwd": str(tmp_path),
        "tool_name": "write_file",
        "tool_input": {"file_path": "product.md"},
    }
    assert "decision" in evaluate(payload, "gemini")
    evaluate(prompt, "claude")
    payload["hook_event_name"] = "PreToolUse"
    payload["tool_name"] = "Write"
    assert evaluate(payload, "claude")["hookSpecificOutput"]["permissionDecision"] == "deny"


def test_database_migrates_hook_events_table(tmp_path) -> None:
    import sqlite3

    database_path = tmp_path / ".tramuri" / "database" / "tramuri.sqlite3"
    database_path.parent.mkdir(parents=True)
    connection = sqlite3.connect(database_path)
    try:
        connection.execute("CREATE TABLE projects (id TEXT PRIMARY KEY, root_path TEXT)")
        connection.execute("PRAGMA user_version = 1")
    finally:
        connection.close()
    Database(tmp_path)
    connection = sqlite3.connect(database_path)
    try:
        assert connection.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='hook_events'"
        ).fetchone()
    finally:
        connection.close()
