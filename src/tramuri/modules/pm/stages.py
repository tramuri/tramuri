"""Product workflow stages and mode-specific output profiles."""

from __future__ import annotations

from tramuri.core.models import Mode

CAPABILITIES = {
    "product-framing": "Product framing",
    "product-discovery": "Product discovery",
    "product-analysis": "Product analysis",
    "product-scope": "Product scope",
    "product-review": "Product review",
    "product-traceability": "Product traceability",
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

SPECIALISTS = {
    "intent": "tramuri-product-framing",
    "framing": "tramuri-product-framing",
    "discovery": "tramuri-product-discovery",
    "research": "tramuri-market-research",
    "hypothesis": "tramuri-product-discovery",
    "shaping": "tramuri-product-scope",
    "success": "tramuri-product-analysis",
    "governance": "tramuri-product-analysis",
    "traceability": "tramuri-product-traceability",
    "delivery": "tramuri-product-scope",
    "experiment": "tramuri-product-discovery",
    "critical_review": "tramuri-product-review",
    "risk_review": "tramuri-product-review",
    "report": "tramuri-product-status",
    "navigation": "tramuri-product-experience",
}


def capability_name(owner: str) -> str:
    return CAPABILITIES.get(owner, owner.replace("-", " ").replace("_", " ").title())


def specialist_name(stage_key: str, owner: str) -> str:
    """Return the concrete product specialist responsible for a stage."""
    return SPECIALISTS.get(stage_key, f"tramuri-{owner}")
