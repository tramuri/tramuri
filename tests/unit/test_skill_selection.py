from pathlib import Path

from tramuri.experience.skills import SkillCatalog


def test_frontend_context_selects_visual_and_verification_skills() -> None:
    catalog = SkillCatalog(Path("tramuri-harness-kit"))

    selection = catalog.resolve("frontend")

    assert {item.name for item in selection.selected} >= {
        "frontend-design",
        "web-design-guidelines",
        "verification-before-completion",
    }
    assert all(item.name != "python-testing" for item in selection.selected)
    assert selection.reasons["frontend-design"]


def test_python_context_does_not_load_frontend_skills() -> None:
    catalog = SkillCatalog(Path("tramuri-harness-kit"))

    selection = catalog.resolve("python")

    assert {item.name for item in selection.selected} >= {
        "python-testing",
        "test-driven-development",
        "verification-before-completion",
    }
    assert all(
        item.name not in {"frontend-design", "web-design-guidelines"} for item in selection.selected
    )


def test_browser_context_reports_missing_runtime_capability() -> None:
    catalog = SkillCatalog(Path("tramuri-harness-kit"))

    selection = catalog.resolve("browser")

    assert any(item.name == "webapp-testing" for item in selection.selected)
    assert any("playwright" in item.lower() for item in selection.unavailable)


def test_unknown_context_is_explicit_and_does_not_load_everything() -> None:
    catalog = SkillCatalog(Path("tramuri-harness-kit"))

    selection = catalog.resolve("unknown")

    assert selection.selected == ()
    assert selection.task_kind == "unknown"
    assert "No contextual skill mapping" in selection.reasons["task"]
