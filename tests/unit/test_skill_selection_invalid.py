from pathlib import Path

from tramuri.experience.skills import SkillCatalog


def test_invalid_skill_frontmatter_is_not_selected(tmp_path: Path) -> None:
    skill = tmp_path / ".agents" / "skills" / "frontend-design"
    skill.mkdir(parents=True)
    (skill / "SKILL.md").write_text("# missing metadata\n", encoding="utf-8")

    selection = SkillCatalog(tmp_path).resolve("frontend")

    assert all(item.name != "frontend-design" for item in selection.selected)
    assert "frontend-design" in selection.unavailable
    assert "frontmatter" in selection.reasons["frontend-design"]
