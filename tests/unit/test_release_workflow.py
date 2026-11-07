from pathlib import Path


def test_release_workflow_is_tagged_and_oidc_only() -> None:
    workflow = Path(".github/workflows/publish.yml").read_text(encoding="utf-8")
    assert 'tags: ["v*.*.*"]' in workflow
    assert "id-token: write" in workflow
    assert "gh-action-pypi-publish" in workflow
    assert "password:" not in workflow


def test_ci_contains_matrix_and_aggregator() -> None:
    workflow = Path(".github/workflows/ci.yml").read_text(encoding="utf-8")
    assert 'python-version: ["3.12", "3.13", "3.14"]' in workflow
    assert "all-checks-passed" in workflow
