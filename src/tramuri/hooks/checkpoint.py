"""Safe provider hook entry point with an observation-only fallback."""

from __future__ import annotations

import json
import sys


def main(provider: str | None = None) -> None:
    """Persist lifecycle facts through the runtime when a provider is known.

    The provider-less form remains a safe diagnostic fallback for copied hook
    scripts and never attempts to infer a provider or execute payload values.
    """
    raw = sys.stdin.read(1_000_000)
    try:
        payload = json.loads(raw) if raw.strip() else {}
        if not isinstance(payload, dict):
            payload = {}
    except json.JSONDecodeError:
        payload = {}
    event = str(payload.get("hook_event_name", "unknown"))
    if provider in {"codex", "claude", "gemini"}:
        from tramuri.hooks.runtime import evaluate

        result = evaluate(payload, provider)
        if result:
            sys.stdout.write(json.dumps(result, ensure_ascii=False) + "\n")
        return
    sys.stdout.write(json.dumps({"tramuri": "checkpoint-observed", "event": event}) + "\n")
