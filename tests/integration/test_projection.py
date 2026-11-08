from pathlib import Path

from tramuri.harness import adapters
from tramuri.harness.projection import project_manifest


def test_all_builtin_harnesses_have_stable_projection(tmp_path: Path) -> None:
    for adapter in adapters():
        manifest = adapter.manifest(tmp_path)
        first = project_manifest(manifest, tmp_path)
        second = project_manifest(manifest, tmp_path)
        assert manifest.provider == adapter.key
        assert [item.relative_path for item in first.components] == [
            item.relative_path for item in second.components
        ]
        assert first.components
