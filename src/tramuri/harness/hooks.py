"""Normalize provider hook payloads and render core decisions natively."""

from __future__ import annotations

from typing import Any

from tramuri.core.contracts import HookDecision

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
BOOTSTRAP_COMMANDS = ("tramuri", "python -m tramuri", "python3 -m tramuri", "py -m tramuri")


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
    return value if isinstance(value, (dict, str)) else {}


def _command_or_path(payload: dict[str, Any]) -> str:
    value = _tool_input(payload)
    if isinstance(value, str):
        return value
    return " ".join(
        str(value[key])
        for key in ("command", "file_path", "path", "target_file")
        if value.get(key) is not None
    )


def _prompt_class(payload: dict[str, Any]) -> str | None:
    prompt = payload.get("prompt")
    if not isinstance(prompt, str) or not prompt.strip():
        return None
    lowered = prompt.casefold()
    return "product_intent" if any(term in lowered for term in PRODUCT_TERMS) else "general"


def normalize_payload(
    payload: dict[str, Any], *, requires_workflow: bool, completed: bool
) -> dict[str, Any]:
    """Return normalized data for the provider-neutral policy."""
    tool = (_tool_name(payload) or "").casefold()
    command = _command_or_path(payload).casefold()
    bootstrap = any(command.lstrip().startswith(item) for item in BOOTSTRAP_COMMANDS)
    normalized = command.replace("\\", "/")
    bootstrap = (
        bootstrap
        or "/tramuri_output/input/" in normalized
        or normalized.startswith("tramuri_output/input/")
    )
    mutating = tool in MUTATING_TOOLS or (
        tool.startswith("mcp__")
        and any(word in tool for word in ("write", "edit", "replace", "delete", "update"))
    )
    return {
        "event": _event_name(payload),
        "session_id": _session_id(payload),
        "tool_name": _tool_name(payload),
        "prompt_class": _prompt_class(payload),
        "is_mutating": mutating,
        "is_bootstrap": bootstrap,
        "has_completed_run": completed,
        "requires_workflow": requires_workflow,
    }


def render_decision(provider: str, decision: HookDecision) -> dict[str, Any]:
    """Render only the provider-specific response envelope."""
    if decision.action.value == "block" and decision.reason:
        if provider == "claude":
            return {
                "hookSpecificOutput": {
                    "hookEventName": decision.event,
                    "permissionDecision": "deny",
                    "permissionDecisionReason": decision.reason,
                }
            }
        if provider == "gemini":
            return {"decision": "deny", "reason": decision.reason}
        return {"decision": "block", "reason": decision.reason}
    if decision.action.value == "context" and decision.context:
        return {
            "hookSpecificOutput": {
                "hookEventName": decision.event,
                "additionalContext": decision.context,
            }
        }
    return {}
