"""Deterministic, provider-neutral policy decisions."""

from __future__ import annotations

from tramuri.core.contracts import HookAction, HookDecision, HookEvaluation


def workflow_context(has_completed_run: bool) -> str:
    if has_completed_run:
        return (
            "Tramuri is active for this project. A completed product run is recorded; "
            "use its SQLite state, artifacts and report when continuing."
        )
    return (
        "Tramuri is active for this project, but no completed product run is recorded. "
        "Before changing project files for a product request, start `tramuri product run --idea-file "
        "<file> --mode mvp` (or run the deterministic `tramuri product demo --mode mvp`)."
    )


def evaluate_hook(event: HookEvaluation) -> HookDecision:
    """Apply the workflow gate without knowing Codex, Claude or Gemini."""
    if event.event in {"UserPromptSubmit", "BeforeAgent"}:
        return HookDecision(
            HookAction.CONTEXT,
            event.event,
            requires_workflow=event.prompt_class == "product_intent",
            context=workflow_context(event.has_completed_run),
            state_snapshot={"has_completed_run": event.has_completed_run},
        )
    if (
        event.event in {"PreToolUse", "BeforeTool"}
        and event.requires_workflow
        and not event.has_completed_run
        and event.is_mutating
        and not event.is_bootstrap
    ):
        return HookDecision(
            HookAction.BLOCK,
            event.event,
            requires_workflow=True,
            reason=(
                "Tramuri gate: this product request has no completed product run. Start the workflow "
                "with `tramuri product run --idea-file <file> --mode mvp` before modifying project files."
            ),
            state_snapshot={"reason": "missing_completed_run"},
        )
    return HookDecision(
        HookAction.ALLOW,
        event.event,
        requires_workflow=event.requires_workflow,
        state_snapshot={"has_completed_run": event.has_completed_run},
    )
