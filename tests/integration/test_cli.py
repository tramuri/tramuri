import json
from pathlib import Path

from typer.testing import CliRunner

from tramuri.cli.app import app


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
