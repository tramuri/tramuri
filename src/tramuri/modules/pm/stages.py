"""Product workflow stages and mode-specific output profiles."""

from __future__ import annotations

from tramuri.core.models import Mode

CAPABILITIES = {
    "product-framing": "Product framing",
    "product-discovery": "Product discovery",
    "product-analysis": "Product analysis",
    "product-scope": "Product scope",
    "product-review": "Product review",
    "system": "Tramuri system",
}

MODE_PROFILES = {
    Mode.POC: {"label": "Proof of Concept", "focus": "test the smallest viable idea"},
    Mode.MVP: {"label": "Minimum Viable Product", "focus": "define a first real product increment"},
    Mode.ENTERPRISE: {
        "label": "Enterprise",
        "focus": "add governance and traceability where relevant",
    },
}


def capability_name(owner: str) -> str:
    return CAPABILITIES.get(owner, owner.replace("-", " ").replace("_", " ").title())
