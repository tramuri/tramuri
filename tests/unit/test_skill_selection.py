from pathlib import Path

from tramuri.experience.skills import SkillCatalog

SKILLS = {
    "frontend-design": "Design intentional interfaces.",
    "web-design-guidelines": "Review accessible interfaces.",
    "verification-before-completion": "Verify acceptance evidence.",
    "webapp-testing": "Test web applications in a browser.",
    "python-testing": "Test Python packages.",
    "test-driven-development": "Develop with tests first.",
    "systematic-debugging": "Debug from the root cause.",
    "skill-creator": "Create focused skills.",
    "web-animation-design": "Design restrained motion.",
}


def _catalog(tmp_path: Path) -> SkillCatalog:
    skills_root = tmp_path / ".agents" / "skills"
    for name, description in SKILLS.items():
        skill_dir = skills_root / name
        skill_dir.mkdir(parents=True)
        (skill_dir / "SKILL.md").write_text(
            f"---\nname: {name}\ndescription: {description}\n---\n",
            encoding="utf-8",
        )
    return SkillCatalog(tmp_path)


def test_frontend_context_selects_visual_and_verification_skills(tmp_path: Path) -> None:
    catalog = _catalog(tmp_path)

    selection = catalog.resolve("frontend")

    assert {item.name for item in selection.selected} >= {
        "frontend-design",
        "web-design-guidelines",
        "verification-before-completion",
    }
    assert all(item.name != "python-testing" for item in selection.selected)
    assert selection.reasons["frontend-design"]


def test_python_context_does_not_load_frontend_skills(tmp_path: Path) -> None:
    catalog = _catalog(tmp_path)

    selection = catalog.resolve("python")

    assert {item.name for item in selection.selected} >= {
        "python-testing",
        "test-driven-development",
        "verification-before-completion",
    }
    assert all(
        item.name not in {"frontend-design", "web-design-guidelines"} for item in selection.selected
    )


def test_browser_context_reports_missing_runtime_capability(tmp_path: Path) -> None:
    catalog = _catalog(tmp_path)

    selection = catalog.resolve("browser")

    assert any(item.name == "webapp-testing" for item in selection.selected)
    assert any("playwright" in item.lower() for item in selection.unavailable)


def test_unknown_context_is_explicit_and_does_not_load_everything(tmp_path: Path) -> None:
    catalog = _catalog(tmp_path)

    selection = catalog.resolve("unknown")

    assert selection.selected == ()
    assert selection.task_kind == "unknown"
    assert "No contextual skill mapping" in selection.reasons["task"]
