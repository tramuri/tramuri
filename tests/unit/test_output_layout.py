from pathlib import Path

from tramuri.core.models import Mode
from tramuri.modules.pm.demo import run_demo
from tramuri.reports.generator import ReportGenerator
from tramuri.storage.database import Database


def test_tramuri_keeps_state_and_generated_artifacts_under_one_output_directory(
    tmp_path: Path,
) -> None:
    database = Database(tmp_path)
    assert database.path == tmp_path / "tramuri_output" / "database" / "tramuri.sqlite3"

    run_id = run_demo(database, Mode.MVP)
    paths = ReportGenerator(tmp_path).generate(database.run_details(run_id))

    assert all(
        Path(value).is_relative_to(tmp_path / "tramuri_output")
        for key, value in paths.items()
        if key in {"report", "prototype", "portal", "demo"}
    )


def test_provider_installation_uses_a_single_tramuri_skill_name(tmp_path: Path) -> None:
    from tramuri.harness import adapter_for

    for provider, root in (("codex", ".agents"), ("claude", ".claude"), ("gemini", ".gemini")):
        plan = adapter_for(provider).installation_plan(tmp_path)
        paths = {item.relative_path for item in plan.components}
        assert f"{root}/skills/tramuri/SKILL.md" in paths
        assert f"{root}/skills/tramuri-product/SKILL.md" not in paths


def test_tramuri_skill_requires_intent_and_limits_questions(tmp_path: Path) -> None:
    from tramuri.harness import adapter_for

    for provider in ("codex", "claude", "gemini"):
        skill = next(
            item
            for item in adapter_for(provider).installation_plan(tmp_path).components
            if item.key == "product-skill"
        )
        assert "Do not invent a product" in skill.content
        assert "ask one concise" in skill.content
        assert "tramuri --json explain" in skill.content
