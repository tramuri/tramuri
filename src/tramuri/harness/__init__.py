"""Provider harnesses: detection, projection, installation and native hooks."""

from tramuri.harness.manifest import HarnessManifest
from tramuri.harness.registry import CapabilityRegistry, adapter_for, adapters, registry

__all__ = ["CapabilityRegistry", "HarnessManifest", "adapter_for", "adapters", "registry"]
