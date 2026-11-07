"""Provider adapters and registry construction."""

from tramuri.providers.base import ProviderAdapter
from tramuri.providers.claude import ClaudeAdapter
from tramuri.providers.codex import CodexAdapter
from tramuri.providers.gemini import GeminiAdapter
from tramuri.providers.registry import CapabilityRegistry


def adapters() -> list[ProviderAdapter]:
    return [CodexAdapter(), ClaudeAdapter(), GeminiAdapter()]


def registry() -> CapabilityRegistry:
    return CapabilityRegistry(adapters())


def adapter_for(name: str) -> ProviderAdapter:
    normalized = name.lower()
    for adapter in adapters():
        if adapter.key == normalized:
            return adapter
    raise KeyError(f"Unknown provider: {name}")


__all__ = ["adapter_for", "adapters", "registry"]
