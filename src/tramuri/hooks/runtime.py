"""Deterministic provider hook gate backed by the project SQLite database."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any

from tramuri.storage.database import Database

PRODUCT_TERMS = (
    "tramuri",
    "produto",
    "product",
    "saas",
    "mvp",
    "poc",
    "feature",
    "funcionalidade",
    "ideia",
    "idea",
    "usuário",
    "usuario",
    "user",
    "cliente",
    "customer",
)
MUTATING_TOOLS = {
    "bash",
    "apply_patch",
    "edit",
    "write",
    "write_file",
    "replace",
    "run_shell_command",
    "run_command",
}
BOOTSTRAP_COMMANDS = (
    "tramuri",
    "python -m tramuri",
    "python3 -m tramuri",
    "py -m tramuri",
)


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
            if (directory / ".tramuri").is_dir():
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


def _tool_input(payload: dict[str, Any]) -> dict[str, Any] | str:
    value = payload.get("tool_input")
    if isinstance(value, (dict, str)):
        return value
    return {}


def _command_or_path(payload: dict[str, Any]) -> str:
    value = _tool_input(payload)
    if isinstance(value, str):
        return value
    values = [value.get(key) for key in ("command", "file_path", "path", "target_file")]
    return " ".join(str(item) for item in values if item is not None)


def _prompt_class(payload: dict[str, Any]) -> str | None:
    prompt = payload.get("prompt")
    if not isinstance(prompt, str) or not prompt.strip():
        return None
    lowered = prompt.casefold()
    return "product_intent" if any(term in lowered for term in PRODUCT_TERMS) else "general"


def _is_mutating(payload: dict[str, Any]) -> bool:
    tool = (_tool_name(payload) or "").casefold()
    return (
        tool in MUTATING_TOOLS
        or tool.startswith("mcp__")
        and any(word in tool for word in ("write", "edit", "replace", "delete", "update"))
    )


def _is_bootstrap_action(payload: dict[str, Any]) -> bool:
    command = _command_or_path(payload).casefold()
    if any(command.lstrip().startswith(candidate) for candidate in BOOTSTRAP_COMMANDS):
        return True
    normalized = command.replace("\\", "/")
    return "/.tramuri/input/" in normalized or normalized.startswith(".tramuri/input/")


def _context(has_completed_run: bool) -> str:
    if has_completed_run:
        return (
            "Tramuri is active for this project. A completed Product Management run is recorded; "
            "use its SQLite state, artifacts and report when continuing."
        )
    return (
        "Tramuri is active for this project, but no completed Product Management run is recorded. "
        "Before changing project files for a product request, start `tramuri pm run --idea-file "
        "<file> --mode mvp` (or run the deterministic `tramuri pm demo --mode mvp`)."
    )


def _output(
    provider: str, event: str, reason: str | None = None, context: str | None = None
) -> dict[str, Any]:
    if reason is not None:
        if provider == "claude":
            return {
                "hookSpecificOutput": {
                    "hookEventName": event,
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason,
                }
            }
        if provider == "gemini":
            return {"decision": "deny", "reason": reason}
        return {"decision": "block", "reason": reason}
    if context is not None:
        return {
            "hookSpecificOutput": {
                "hookEventName": event,
                "additionalContext": context,
            }
        }
    return {}


def evaluate(payload: dict[str, Any], provider: str) -> dict[str, Any]:
    """Evaluate one provider event without executing its input or any project code."""
    project_dir = _project_dir(payload)
    event = _event_name(payload)
    session_id = _session_id(payload)
    tool_name = _tool_name(payload)
    prompt_class = _prompt_class(payload)
    if project_dir is None:
        return {}

    database = Database(project_dir)
    has_completed_run = database.has_completed_run()
    metadata = {
        "event": event,
        "has_completed_run": has_completed_run,
        "requires_workflow": prompt_class == "product_intent",
    }
    if event in {"UserPromptSubmit", "BeforeAgent"}:
        database.hook_event(
            provider,
            event,
            session_id,
            None,
            prompt_class,
            "context",
            metadata,
        )
        return _output(provider, event, context=_context(has_completed_run))

    context_row = database.latest_hook_context(provider, session_id)
    requires_workflow = bool(context_row and context_row["prompt_class"] == "product_intent")
    if (
        event in {"PreToolUse", "BeforeTool"}
        and requires_workflow
        and not has_completed_run
        and _is_mutating(payload)
        and not _is_bootstrap_action(payload)
    ):
        reason = (
            "Tramuri gate: this product request has no completed PM run. Start the workflow "
            "with `tramuri pm run --idea-file <file> --mode mvp` before modifying project files."
        )
        database.hook_event(
            provider,
            event,
            session_id,
            tool_name,
            prompt_class,
            "blocked",
            metadata | {"reason": "missing_completed_run"},
        )
        return _output(provider, event, reason=reason)

    database.hook_event(
        provider,
        event,
        session_id,
        tool_name,
        prompt_class,
        "allowed",
        metadata,
    )
    return {}


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
        if event in {"PreToolUse", "BeforeTool"} and _is_mutating(payload):
            result = _output(
                provider,
                event,
                reason=f"Tramuri gate unavailable; start the workflow before continuing ({type(exc).__name__}).",
            )
        else:
            result = {}
    if result:
        sys.stdout.write(json.dumps(result, ensure_ascii=False) + "\n")
