"""Capability registry shared by provider discovery and diagnostics."""

from __future__ import annotations

from dataclasses import asdict
from pathlib import Path

from tramuri.core.models import Capability, CapabilityStatus
from tramuri.providers.base import ProviderAdapter, ProviderDetection


class CapabilityRegistry:
    def __init__(self, adapters: list[ProviderAdapter]):
        self.adapters = adapters

    def detect(self, project_dir: Path) -> list[ProviderDetection]:
        return [adapter.detect(project_dir) for adapter in self.adapters]

    def capabilities(self, project_dir: Path) -> list[Capability]:
        capabilities: list[Capability] = []
        for adapter in self.adapters:
            capabilities.extend(adapter.capabilities(project_dir))
        return capabilities

    def as_dicts(self, project_dir: Path) -> list[dict[str, object]]:
        return [
            asdict(item) | {"status": item.status.value} for item in self.capabilities(project_dir)
        ]


def capability_status(installed: bool, supported: bool = True) -> CapabilityStatus:
    if not supported:
        return CapabilityStatus.UNSUPPORTED
    return CapabilityStatus.AVAILABLE if installed else CapabilityStatus.MISSING
