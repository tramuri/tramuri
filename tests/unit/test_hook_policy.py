from tramuri.core.contracts import HookAction, HookEvaluation
from tramuri.core.policy import evaluate_hook


def test_prompt_context_is_a_core_decision() -> None:
    decision = evaluate_hook(HookEvaluation("BeforeAgent", prompt_class="product_intent"))
    assert decision.action is HookAction.CONTEXT
    assert decision.context and "Tramuri" in decision.context
