"""Declarative description of a provider harness projection."""

from __future__ import annotations

from dataclasses import dataclass, field

from tramuri.core.models import InstallationComponent


@dataclass(frozen=True, slots=True)
class HarnessManifest:
    """Capabilities and assets a provider harness can project safely."""

    provider: str
    version: str
    components: tuple[InstallationComponent, ...] = ()
    capabilities: tuple[str, ...] = ()
    hook_events: tuple[str, ...] = ()
    notes: tuple[str, ...] = ()
    metadata: dict[str, str] = field(default_factory=dict)
