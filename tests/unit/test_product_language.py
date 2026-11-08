from __future__ import annotations

from pathlib import Path

from tramuri.harness import adapters


def test_provider_projection_uses_product_language_not_profession_names() -> None:
    forbidden = ("product-manager", "product strategist", "discovery specialist", "product analyst")
    for adapter in adapters():
        plan = adapter.installation_plan(Path("."))
        for component in plan.components:
            text = f"{component.relative_path} {component.content}".casefold()
            assert not any(term in text for term in forbidden)
