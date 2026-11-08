from tramuri.core.contracts import HookAction, HookDecision
from tramuri.harness.hooks import render_decision


def test_provider_golden_denial_shapes() -> None:
    decision = HookDecision(HookAction.BLOCK, "PreToolUse", reason="blocked")
    assert render_decision("codex", decision) == {"decision": "block", "reason": "blocked"}
    assert render_decision("gemini", decision) == {"decision": "deny", "reason": "blocked"}
    assert render_decision("claude", decision)["hookSpecificOutput"]["permissionDecision"] == "deny"
