"""Product-management roles and mode-specific output profiles."""

from __future__ import annotations

from tramuri.core.models import Mode

SPECIALISTS = {
    "product_strategist": "Product Strategist",
    "discovery_specialist": "Discovery Specialist",
    "product_analyst": "Product Analyst",
    "scope_specialist": "Scope Specialist",
    "critical_reviewer": "Critical Reviewer",
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


def specialist_name(owner: str) -> str:
    return SPECIALISTS.get(owner, owner.replace("_", " ").title())
