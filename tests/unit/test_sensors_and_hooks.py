import json
from io import StringIO
from unittest.mock import patch

from tramuri.hooks import checkpoint
from tramuri.sensors.checks import validate_payload


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
