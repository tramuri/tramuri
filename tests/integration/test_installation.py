from pathlib import Path

from tramuri.core.models import InstallationComponent, InstallationPlan
from tramuri.harness import adapter_for
from tramuri.harness.installer import Installer
from tramuri.storage.database import Database


def test_installer_is_idempotent_and_preserves_user_changes(tmp_path: Path) -> None:
    database = Database(tmp_path)
    installer = Installer(tmp_path, database)
    plan = adapter_for("codex").installation_plan(tmp_path)
    first = installer.install(plan)
    second = installer.install(plan)
    first_components = [
        {key: value for key, value in item.items() if key != "installed_at"}
        for item in first["components"]
    ]
    second_components = [
        {key: value for key, value in item.items() if key != "installed_at"}
        for item in second["components"]
    ]
    assert first_components == second_components
    managed = tmp_path / ".agents" / "skills" / "tramuri" / "SKILL.md"
    managed.write_text(managed.read_text(encoding="utf-8") + "\nuser change\n", encoding="utf-8")
    result = installer.uninstall("codex")
    assert ".agents/skills/tramuri/SKILL.md" in [
        str(value).replace("\\", "/") for value in result["preserved"]
    ]
    assert managed.is_file()


def test_json_settings_merge_keeps_existing_hook(tmp_path: Path) -> None:
    settings = tmp_path / ".claude" / "settings.json"
    settings.parent.mkdir()
    settings.write_text('{"custom": true, "hooks": {"Other": []}}', encoding="utf-8")
    installer = Installer(tmp_path, Database(tmp_path))
    installer.install(adapter_for("claude").installation_plan(tmp_path))
    content = settings.read_text(encoding="utf-8")
    assert '"custom": true' in content
    assert "PostToolUse" in content and "Other" in content


def test_project_instructions_add_identity_without_overwriting_user_content(tmp_path: Path) -> None:
    instructions = tmp_path / "AGENTS.md"
    instructions.write_text(
        "# Existing project guidance\n\nKeep the API stable.\n", encoding="utf-8"
    )
    installer = Installer(tmp_path, Database(tmp_path))
    plan = adapter_for("codex").installation_plan(tmp_path)

    installer.install(plan)
    installed = instructions.read_text(encoding="utf-8")
    assert "# Existing project guidance" in installed
    assert "# Tramuri — Product companion" in installed
    assert "<!-- tramuri:managed:start -->" in installed

    installer.install(plan)
    assert instructions.read_text(encoding="utf-8") == installed

    result = installer.uninstall("codex")
    assert "AGENTS.md" in [str(value).replace("\\", "/") for value in result["removed"]]
    assert (
        instructions.read_text(encoding="utf-8")
        == "# Existing project guidance\n\nKeep the API stable.\n"
    )


def test_codex_update_migrates_skill_identifier_in_place(tmp_path: Path) -> None:
    installer = Installer(tmp_path, Database(tmp_path))
    legacy_plan = InstallationPlan(
        "codex",
        [
            InstallationComponent(
                "skill-product-status",
                ".agents/skills/product-status/SKILL.md",
                "---\nname: product-status\ndescription: Legacy status skill.\n---\n",
            )
        ],
        [],
    )
    installer.install(legacy_plan)

    installer.install(adapter_for("codex").installation_plan(tmp_path))

    migrated = tmp_path / ".agents" / "skills" / "product-status" / "SKILL.md"
    assert "name: tramuri-status" in migrated.read_text(encoding="utf-8")
    assert not (tmp_path / ".agents" / "skills" / "tramuri-status").exists()
