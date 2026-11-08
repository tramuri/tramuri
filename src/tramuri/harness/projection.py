"""Provider-independent projection helpers."""

from __future__ import annotations

from dataclasses import replace
from pathlib import Path

from tramuri.core.models import InstallationPlan
from tramuri.core.safety import safe_child
from tramuri.harness.manifest import HarnessManifest
from tramuri.harness.providers.base import ProviderAdapter


def project_manifest(
    manifest: HarnessManifest,
    project_dir: Path | None = None,
    variables: dict[str, str] | None = None,
) -> InstallationPlan:
    """Validate and materialize a deterministic installer plan."""
    variables = {"provider": manifest.provider, "version": manifest.version, **(variables or {})}
    components = []
    for component in manifest.components:
        content = component.content
        path = component.relative_path
        for key, value in variables.items():
            content = content.replace("{{" + key + "}}", value)
            path = path.replace("{{" + key + "}}", value)
        if project_dir is not None:
            safe_child(project_dir, path)
        components.append(replace(component, relative_path=path, content=content))
    return InstallationPlan(
        manifest.provider,
        components,
        list(manifest.notes) + [f"Harness manifest {manifest.provider}@{manifest.version}"],
    )


def manifest_for(adapter: ProviderAdapter, project_dir: Path) -> HarnessManifest:
    """Build a manifest from any adapter implementing the shared protocol."""
    provider = str(adapter.key)
    plan = adapter.installation_plan(project_dir)
    capabilities = tuple(item.key for item in adapter.capabilities(project_dir))
    return HarnessManifest(
        provider, "1", tuple(plan.components), capabilities, notes=tuple(plan.notes)
    )
