import json
from pathlib import Path

from typer.testing import CliRunner

from tramuri.cli.app import app
from tramuri.core.models import Mode
from tramuri.modules.pm.runner import PMRunner
from tramuri.storage.database import Database


def test_cli_demo_json_is_machine_readable(tmp_path: Path) -> None:
    result = CliRunner().invoke(app, ["--project-dir", str(tmp_path), "--json", "product", "demo"])
    assert result.exit_code == 0, result.stdout
    payload = json.loads(result.stdout)
    assert payload["status"] == "completed"
    assert payload["synthetic"] is True
    assert Path(payload["reports"]["report"]).is_file()


def test_cli_help_and_provider_diagnostics(tmp_path: Path) -> None:
    runner = CliRunner()
    help_result = runner.invoke(app, ["--help"])
    assert help_result.exit_code == 0
    assert "providers" in help_result.stdout and "product" in help_result.stdout
    assert " pm " not in f" {help_result.stdout} "
    doctor = runner.invoke(app, ["--project-dir", str(tmp_path), "--json", "doctor"])
    assert doctor.exit_code == 0, doctor.stdout
    assert len(json.loads(doctor.stdout)["providers"]) == 3

    legacy = runner.invoke(app, ["pm", "demo"])
    assert legacy.exit_code != 0
    assert "No such command 'pm'" in legacy.output


def test_external_harness_skills_are_not_a_product_cli_feature(tmp_path: Path) -> None:
    help_result = CliRunner().invoke(app, ["experience", "--help"])
    result = CliRunner().invoke(
        app,
        ["--project-dir", str(tmp_path), "experience", "skills", "frontend"],
    )

    assert "skills" not in help_result.stdout.lower()
    assert result.exit_code != 0
    assert "No such command 'skills'" in result.output


def test_cli_memory_commands_save_show_and_search_project_memory(tmp_path: Path) -> None:
    runner = CliRunner()
    saved = runner.invoke(
        app,
        [
            "--project-dir",
            str(tmp_path),
            "--json",
            "memory",
            "save",
            "--title",
            "Preferred language",
            "--content",
            "The user prefers Brazilian Portuguese.",
            "--classification",
            "fact",
        ],
    )
    assert saved.exit_code == 0, saved.output
    entry_id = json.loads(saved.stdout)["entry_id"]

    shown = runner.invoke(app, ["--project-dir", str(tmp_path), "--json", "memory", "show"])
    searched = runner.invoke(
        app,
        ["--project-dir", str(tmp_path), "--json", "memory", "search", "language"],
    )

    assert shown.exit_code == 0, shown.output
    assert searched.exit_code == 0, searched.output
    show_payload = json.loads(shown.stdout)
    search_payload = json.loads(searched.stdout)
    assert "Preferred language" in Path(show_payload["current_file"]).read_text(encoding="utf-8")
    assert search_payload["results"][0]["id"] == entry_id


def test_cli_install_status_runs_and_report(tmp_path: Path) -> None:
    runner = CliRunner()
    installed = runner.invoke(
        app, ["--project-dir", str(tmp_path), "--json", "install", "--provider", "codex"]
    )
    assert installed.exit_code == 0, installed.stdout
    init = runner.invoke(
        app,
        ["--project-dir", str(tmp_path), "--json", "init", "--provider", "codex", "--dry-run"],
    )
    assert init.exit_code == 0, init.stdout
    status = runner.invoke(app, ["--project-dir", str(tmp_path), "--json", "status"])
    assert status.exit_code == 0
    idea = tmp_path / "idea.md"
    idea.write_text("A local scheduling product", encoding="utf-8")
    run = runner.invoke(
        app,
        [
            "--project-dir",
            str(tmp_path),
            "--json",
            "product",
            "run",
            "--idea-file",
            str(idea),
            "--mode",
            "poc",
        ],
    )
    assert run.exit_code == 0, run.stdout
    run_id = json.loads(run.stdout)["run_id"]
    opened = runner.invoke(
        app,
        ["--project-dir", str(tmp_path), "--json", "report", "open", run_id, "--no-open"],
    )
    assert opened.exit_code == 0
    updated = runner.invoke(
        app, ["--project-dir", str(tmp_path), "--json", "update", "--provider", "codex"]
    )
    assert updated.exit_code == 0, updated.stdout
    removed = runner.invoke(
        app, ["--project-dir", str(tmp_path), "--json", "uninstall", "--provider", "codex"]
    )
    assert removed.exit_code == 0, removed.stdout


def test_cli_research_import_resumes_provider_run(tmp_path: Path) -> None:
    runner = CliRunner()
    idea = tmp_path / "idea.md"
    idea.write_text("# Course marketplace\nLearners buy and complete courses.", encoding="utf-8")
    started = runner.invoke(
        app,
        [
            "--project-dir",
            str(tmp_path),
            "--json",
            "product",
            "run",
            "--idea-file",
            str(idea),
            "--provider",
            "codex",
        ],
    )
    assert started.exit_code == 0, started.stdout
    run_id = json.loads(started.stdout)["run_id"]
    research = tmp_path / "research.json"
    research.write_text(
        json.dumps(
            {
                "version": "tramuri.research.v1",
                "researched_at": "2026-11-08T12:00:00Z",
                "query": "course marketplace",
                "sources": [
                    {
                        "id": "s1",
                        "url": "https://example.com",
                        "title": "Market signal",
                        "publisher": "Example",
                        "accessed_at": "2026-11-08",
                        "confidence": "low",
                    }
                ],
                "competitors": [],
                "market_signals": [],
                "buying_motivations": [],
                "build_buy_partner": [],
                "positioning": [],
                "market_sizing": {"status": "pending_validation", "claims": []},
            }
        ),
        encoding="utf-8",
    )
    resumed = runner.invoke(
        app,
        [
            "--project-dir",
            str(tmp_path),
            "--json",
            "product",
            "research",
            "--run-id",
            run_id,
            "--research-file",
            str(research),
        ],
    )
    assert resumed.exit_code == 0, resumed.stdout
    assert json.loads(resumed.stdout)["status"] == "completed"


def test_experience_render_persists_manual_category_correction(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = PMRunner(database).run(
        "A local team product that shares updates and records decisions.", Mode.MVP
    )

    result = CliRunner().invoke(
        app,
        [
            "--project-dir",
            str(tmp_path),
            "--json",
            "experience",
            "render",
            run_id,
            "--category",
            "desktop_app",
        ],
    )

    assert result.exit_code == 0, result.output
    payload = json.loads(result.stdout)
    assert payload["demo_category"] == "desktop_app"
    assert payload["demo_template"] == "desktop"
    details = database.run_details(run_id)
    assert any(
        event["event_type"] == "experience.classification_override"
        and event["payload"]["category"] == "desktop_app"
        for event in details["events"]
    )
