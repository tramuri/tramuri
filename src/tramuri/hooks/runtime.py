"""Compatibility command boundary for provider lifecycle hooks."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any

from tramuri.core.context import build_context_pack
from tramuri.core.contracts import HookAction, HookDecision, HookEvaluation
from tramuri.core.paths import output_root
from tramuri.core.policy import evaluate_hook
from tramuri.harness.hooks import normalize_payload, render_decision
from tramuri.storage.database import Database


def _project_dir(payload: dict[str, Any]) -> Path | None:
    candidates = (
        payload.get("cwd"),
        os.environ.get("TRAMURI_PROJECT_DIR"),
        os.environ.get("CLAUDE_PROJECT_DIR"),
        os.environ.get("GEMINI_PROJECT_DIR"),
    )
    for value in candidates:
        if not isinstance(value, str) or not value.strip():
            continue
        current = Path(value).expanduser().resolve(strict=False)
        for directory in (current, *current.parents):
            if output_root(directory).is_dir():
                return directory
    return None


def _event_name(payload: dict[str, Any]) -> str:
    return str(payload.get("hook_event_name") or payload.get("event") or "unknown")


def _session_id(payload: dict[str, Any]) -> str | None:
    value = payload.get("session_id") or payload.get("turn_id")
    return str(value) if value is not None else None


def _tool_name(payload: dict[str, Any]) -> str | None:
    value = payload.get("tool_name")
    return str(value) if value is not None else None


def _event_id(payload: dict[str, Any]) -> str | None:
    value = payload.get("event_id") or payload.get("hook_event_id")
    return str(value) if value is not None else None


def evaluate(payload: dict[str, Any], provider: str) -> dict[str, Any]:
    """Load local facts, evaluate core policy and render the native response."""
    project_dir = _project_dir(payload)
    if project_dir is None:
        return {}
    event = _event_name(payload)
    session_id = _session_id(payload)
    tool_name = _tool_name(payload)
    database = Database(project_dir)
    completed = database.has_completed_run()
    context_row = database.latest_hook_context(provider, session_id)
    prompt_class = (
        str(context_row["prompt_class"])
        if context_row and context_row["prompt_class"] is not None
        else None
    )
    requires_workflow = prompt_class == "product_intent"
    if event in {"UserPromptSubmit", "BeforeAgent"}:
        prompt_class = None
        requires_workflow = False
    normalized = normalize_payload(
        payload, requires_workflow=requires_workflow, completed=completed
    )
    decision = evaluate_hook(HookEvaluation(**normalized))
    runs = database.list_runs()
    active_run = runs[0] if runs else None
    if session_id:
        database.record_session_event(
            provider,
            event,
            session_id,
            run_id=str(active_run["id"]) if active_run else None,
            metadata={"source": "provider_hook", "event_kind": event},
            idempotency_key=(
                f"{provider}:{session_id}:{_event_id(payload)}" if _event_id(payload) else None
            ),
        )
    if decision.context and active_run:
        details = database.run_details(str(active_run["id"]))
        context_pack = build_context_pack(
            details,
            knowledge=database.list_knowledge(run_id=str(active_run["id"])),
            sensors=database.sensor_results(str(active_run["id"])),
            max_bytes=12_000,
        )
        decision = HookDecision(
            decision.action,
            decision.event,
            requires_workflow=decision.requires_workflow,
            reason=decision.reason,
            context=(
                f"{decision.context} Resume context is persisted locally. "
                f"Run {active_run['id']} status={details['run']['status']}; "
                f"context_digest={context_pack['digest']}."
            ),
            state_snapshot=decision.state_snapshot
            | {"context_digest": context_pack["digest"], "run_id": active_run["id"]},
        )
    database.hook_event(
        provider,
        event,
        session_id,
        tool_name,
        str(normalized["prompt_class"]) if normalized["prompt_class"] else prompt_class,
        decision.action.value,
        {
            "event": event,
            "has_completed_run": completed,
            "requires_workflow": decision.requires_workflow,
            **decision.state_snapshot,
        },
    )
    return render_decision(provider, decision)


def main(provider: str) -> None:
    """Run as a provider command hook; stdout is always JSON-only."""
    raw = sys.stdin.read(1_000_000)
    try:
        payload = json.loads(raw) if raw.strip() else {}
        if not isinstance(payload, dict):
            payload = {}
    except json.JSONDecodeError:
        payload = {}
    try:
        result = evaluate(payload, provider)
    except Exception as exc:  # pragma: no cover - defensive hook boundary
        event = _event_name(payload)
        if event in {"PreToolUse", "BeforeTool"}:
            normalized = normalize_payload(payload, requires_workflow=True, completed=False)
            decision = evaluate_hook(HookEvaluation(**normalized))
            if decision.action is HookAction.BLOCK:
                decision = HookDecision(
                    HookAction.BLOCK,
                    event,
                    reason=(
                        "Tramuri gate unavailable; start the workflow before continuing "
                        f"({type(exc).__name__})."
                    ),
                )
            result = render_decision(provider, decision)
        else:
            result = {}
    if result:
        sys.stdout.write(json.dumps(result, ensure_ascii=False) + "\n")
