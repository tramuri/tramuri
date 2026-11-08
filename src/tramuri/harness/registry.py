"""Capability registry for provider harnesses."""

from __future__ import annotations

from dataclasses import asdict
from pathlib import Path

from tramuri.core.models import Capability, CapabilityStatus
from tramuri.harness.providers.base import ProviderAdapter, ProviderDetection
from tramuri.harness.providers.claude import ClaudeAdapter
from tramuri.harness.providers.codex import CodexAdapter
from tramuri.harness.providers.gemini import GeminiAdapter


class CapabilityRegistry:
    def __init__(self, adapters: list[ProviderAdapter]):
        self.adapters = adapters

    def detect(self, project_dir: Path) -> list[ProviderDetection]:
        return [adapter.detect(project_dir) for adapter in self.adapters]

    def capabilities(self, project_dir: Path) -> list[Capability]:
        return [item for adapter in self.adapters for item in adapter.capabilities(project_dir)]

    def as_dicts(self, project_dir: Path) -> list[dict[str, object]]:
        return [
            asdict(item) | {"status": item.status.value} for item in self.capabilities(project_dir)
        ]


def capability_status(installed: bool, supported: bool = True) -> CapabilityStatus:
    if not supported:
        return CapabilityStatus.UNSUPPORTED
    return CapabilityStatus.AVAILABLE if installed else CapabilityStatus.MISSING


def adapters() -> list[ProviderAdapter]:
    """Return the built-in provider harnesses."""
    return [CodexAdapter(), ClaudeAdapter(), GeminiAdapter()]


def registry() -> CapabilityRegistry:
    return CapabilityRegistry(adapters())


def adapter_for(name: str) -> ProviderAdapter:
    normalized = name.casefold()
    for adapter in adapters():
        if adapter.key == normalized:
            return adapter
    raise KeyError(f"Unknown provider: {name}")
