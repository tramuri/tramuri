from pathlib import Path

import pytest

from fixtures.providers import FixtureAdapter
from tramuri.core.contracts import HookAction, HookEvaluation
from tramuri.core.errors import UnsafePathError
from tramuri.core.policy import evaluate_hook
from tramuri.harness.projection import project_manifest


def test_core_contracts_do_not_require_a_provider() -> None:
    decision = evaluate_hook(HookEvaluation("PreToolUse", is_mutating=True))
    assert decision.action is HookAction.ALLOW


def test_fixture_manifest_projects_deterministically(tmp_path: Path) -> None:
    manifest = FixtureAdapter().manifest(tmp_path)
    first = project_manifest(manifest, tmp_path)
    second = project_manifest(manifest, tmp_path)
    assert first.components[0].content == second.components[0].content
    assert first.components[0].relative_path == ".fixture/skill.md"


def test_projection_rejects_traversal(tmp_path: Path) -> None:
    manifest = FixtureAdapter().manifest(tmp_path)
    bad = type(manifest)(
        manifest.provider,
        manifest.version,
        (manifest.components[0].__class__("bad", "../escape", "x"),),
    )
    with pytest.raises(UnsafePathError):
        project_manifest(bad, tmp_path)
