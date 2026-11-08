from __future__ import annotations

import json

from typer.testing import CliRunner

from tramuri.cli.app import app


def test_status_json_and_no_color_are_actionable(tmp_path) -> None:
    runner = CliRunner()
    empty = runner.invoke(app, ["--project-dir", str(tmp_path), "--json", "status"])
    assert empty.exit_code == 0, empty.stdout
    empty_payload = json.loads(empty.stdout)
    assert empty_payload["semantic_status"] == "ready"
    assert "next_action" in empty_payload

    demo = runner.invoke(
        app, ["--project-dir", str(tmp_path), "--no-color", "product", "demo", "--mode", "poc"]
    )
    assert demo.exit_code == 0, demo.stdout
    assert "[x] Intent" in demo.stdout
    status = runner.invoke(app, ["--project-dir", str(tmp_path), "--no-color", "status"])
    assert status.exit_code == 0, status.stdout
    assert "WORKFLOW STATUS" in status.stdout
    assert "Next action" in status.stdout
    assert "STAGES" in status.stdout
    assert "[x]" in status.stdout
    assert "Product Strategist" not in status.stdout
    assert "Product Analyst" not in status.stdout
