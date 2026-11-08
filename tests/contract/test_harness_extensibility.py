from pathlib import Path

from fixtures.providers import FixtureAdapter
from tramuri.harness.projection import project_manifest
from tramuri.harness.registry import CapabilityRegistry


def test_fixture_provider_is_public_harness_only(tmp_path: Path) -> None:
    adapter = FixtureAdapter()
    registry = CapabilityRegistry([adapter])
    assert registry.capabilities(tmp_path)[0].provider == "fixture"
    assert project_manifest(adapter.manifest(tmp_path), tmp_path).provider == "fixture"
