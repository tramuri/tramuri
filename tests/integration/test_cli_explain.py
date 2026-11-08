import json
from pathlib import Path

from typer.testing import CliRunner

from tramuri.cli.app import app
from tramuri.core.models import Mode
from tramuri.modules.pm.demo import run_demo
from tramuri.storage.database import Database


def test_cli_explain_renders_persisted_product_connections(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_demo(database, Mode.MVP)

    result = CliRunner().invoke(app, ["--project-dir", str(tmp_path), "explain", "scope"])

    assert result.exit_code == 0, result.stdout
    assert "TRAMURI / EXPLAIN" in result.stdout
    assert "Sources" in result.stdout
    assert "pm.shaping" in result.stdout
    assert "No persisted Tramuri evidence" not in result.stdout


def test_cli_explain_json_is_machine_readable(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.POC)

    result = CliRunner().invoke(
        app,
        ["--project-dir", str(tmp_path), "--json", "explain", "hypothesis", "--run-id", run_id],
    )

    assert result.exit_code == 0, result.stdout
    payload = json.loads(result.stdout)
    assert payload["schema_version"] == "tramuri.explain.v1"
    assert payload["run_id"] == run_id
    assert payload["found"] is True
