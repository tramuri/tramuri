"""Provider hook entry point; reads JSON but does not execute its contents."""

from __future__ import annotations

import json
import sys


def main() -> None:
    raw = sys.stdin.read(1_000_000)
    try:
        payload = json.loads(raw) if raw.strip() else {}
        event = (
            payload.get("hook_event_name", "unknown") if isinstance(payload, dict) else "unknown"
        )
    except json.JSONDecodeError:
        event = "invalid-json"
    sys.stdout.write(json.dumps({"tramuri": "checkpoint-observed", "event": event}) + "\n")
