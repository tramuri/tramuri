from tramuri.core.contracts import HookAction, HookEvaluation
from tramuri.core.models import Mode
from tramuri.core.policy import evaluate_hook
from tramuri.core.workflow import stage_definitions


def test_policy_is_provider_neutral_and_blocks_only_product_mutations() -> None:
    decision = evaluate_hook(
        HookEvaluation(
            "PreToolUse",
            is_mutating=True,
            has_completed_run=False,
            requires_workflow=True,
        )
    )
    assert decision.action is HookAction.BLOCK
    assert "completed product run" in (decision.reason or "")


def test_stage_definitions_expose_sensors_and_retry_limit() -> None:
    definitions = stage_definitions(Mode.MVP)
    assert definitions[-1].key == "navigation"
    assert all(
        item.sensors and item.completion_criteria and item.max_attempts == 3 for item in definitions
    )
