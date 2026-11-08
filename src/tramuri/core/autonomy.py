"""Bounded autonomy decisions used by workflows and provider instructions."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class AutonomyDecision:
    """Explain whether a missing detail can be resolved without interrupting."""

    action: str
    requires_user: bool
    reason: str
    operational_default: str | None = None


def decide_gap(
    operational_default: str | None,
    *,
    impact: str = "low",
    requires_authorization: bool = False,
    risk: str | None = None,
) -> AutonomyDecision:
    """Proceed on safe gaps; gate only choices that require an explicit human decision."""

    high_impact = impact.casefold() in {"high", "critical"}
    sensitive = risk in {"security", "privacy", "legal"}
    if requires_authorization or high_impact or sensitive or not operational_default:
        return AutonomyDecision(
            action="gate",
            requires_user=True,
            reason=(
                "A safe operational default is unavailable or the decision requires explicit "
                "authorization, security, privacy, legal, or high-impact review."
            ),
            operational_default=operational_default,
        )
    return AutonomyDecision(
        action="proceed",
        requires_user=False,
        reason="The gap has a reversible, low-impact operational default; record it for validation.",
        operational_default=operational_default,
    )
