from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess

import pytest

from tramuri import (
    TramuriError,
    evidence_add,
    execution_record,
    init,
    knowledge_add,
    knowledge_list,
    knowledge_withdraw,
    mission_close,
    mission_contract_revise,
    mission_create,
    mission_get,
    scan,
)


def test_init_is_idempotent_and_scan_returns_inventory(tmp_path: Path) -> None:
    source = tmp_path / "src" / "mod.rs"
    source.parent.mkdir()
    source.write_text("pub fn sample() {}\n", encoding="utf-8")

    first_init = init(tmp_path)
    second_init = init(tmp_path)
    report = scan(tmp_path)

    assert first_init["schema_version"] == 4
    assert first_init["already_initialized"] is False
    assert second_init["already_initialized"] is True
    assert report["complete"] is True
    assert [entry["path"] for entry in report["files"]] == ["src/mod.rs"]
    assert report["added"] == ["src/mod.rs"]


def test_scan_requires_init(tmp_path: Path) -> None:
    with pytest.raises(TramuriError, match="init"):
        scan(tmp_path)


def _cli_executable() -> str | None:
    return os.environ.get("TRAMURI_CLI")


def _copy_discovery_fixture(project_root: Path) -> None:
    fixture_root = Path(__file__).parents[1] / "fixtures" / "discovery"
    shutil.copytree(fixture_root, project_root)


def _normalize_report(report: dict[str, object]) -> dict[str, object]:
    normalized = dict(report)
    normalized.pop("project_root", None)
    files = normalized.get("files")
    if isinstance(files, list):
        normalized["files"] = [
            {key: value for key, value in entry.items() if key != "modified_unix_ms"}
            for entry in files
        ]
    return normalized


def _normalize_init_report(report: dict[str, object]) -> dict[str, object]:
    normalized = dict(report)
    normalized.pop("project_root", None)
    database_path = Path(str(normalized.pop("database_path")))
    normalized["database_location"] = database_path.parts[-2:]
    return normalized


def test_cli_and_sdk_init_json_have_normalized_parity(tmp_path: Path) -> None:
    cli = _cli_executable()
    if not cli:
        pytest.skip("Set TRAMURI_CLI to the built native executable to run CLI/SDK parity.")

    cli_project = tmp_path / "cli-project"
    sdk_project = tmp_path / "sdk-project"
    _copy_discovery_fixture(cli_project)
    _copy_discovery_fixture(sdk_project)

    sdk_report = init(sdk_project)
    cli_result = subprocess.run(
        [cli, "init", str(cli_project), "--json"],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    cli_report = json.loads(cli_result.stdout)

    assert cli_result.stderr == ""
    assert cli_report["project_root"] != sdk_report["project_root"]
    assert _normalize_init_report(cli_report) == _normalize_init_report(sdk_report)


def test_cli_and_sdk_scan_json_have_normalized_parity(tmp_path: Path) -> None:
    cli = _cli_executable()
    if not cli:
        pytest.skip("Set TRAMURI_CLI to the built native executable to run CLI/SDK parity.")

    cli_project = tmp_path / "cli-project"
    sdk_project = tmp_path / "sdk-project"
    for project in (cli_project, sdk_project):
        _copy_discovery_fixture(project)

    init(sdk_project)
    subprocess.run(
        [cli, "init", str(cli_project), "--json"],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    sdk_report = scan(sdk_project)
    cli_result = subprocess.run(
        [cli, "scan", str(cli_project), "--json"],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    cli_report = json.loads(cli_result.stdout)

    assert cli_result.stderr == ""
    assert cli_report["project_root"] != sdk_report["project_root"]
    assert _normalize_report(cli_report) == _normalize_report(sdk_report)
    assert "ignored/not-indexed.txt" not in {entry["path"] for entry in cli_report["files"]}
    assert "sensitive/.env.test" not in {entry["path"] for entry in cli_report["files"]}
    assert cli_report["sensitive_entries_excluded"] == 1


def test_cli_and_sdk_uninitialized_scan_errors_are_equivalent(tmp_path: Path) -> None:
    cli = _cli_executable()
    if not cli:
        pytest.skip("Set TRAMURI_CLI to the built native executable to run CLI/SDK parity.")

    project = tmp_path / "uninitialized-project"
    project.mkdir()
    cli_result = subprocess.run(
        [cli, "scan", str(project), "--json"],
        capture_output=True,
        text=True,
        encoding="utf-8",
    )

    with pytest.raises(TramuriError, match="init"):
        scan(project)

    assert cli_result.returncode != 0
    assert cli_result.stdout == ""
    assert "init" in cli_result.stderr


def test_cli_and_sdk_partial_scan_have_normalized_parity(tmp_path: Path) -> None:
    cli = _cli_executable()
    if not cli:
        pytest.skip("Set TRAMURI_CLI to the built native executable to run CLI/SDK parity.")

    outside = tmp_path / "outside.txt"
    outside.write_text("outside the project", encoding="utf-8")
    cli_project = tmp_path / "cli-partial"
    sdk_project = tmp_path / "sdk-partial"
    for project in (cli_project, sdk_project):
        _copy_discovery_fixture(project)
        try:
            (project / "external-link.txt").symlink_to(outside)
        except OSError as error:
            pytest.skip(f"Host cannot create symlinks required for partial-scan parity: {error}")
        init(project)

    sdk_report = scan(sdk_project)
    cli_result = subprocess.run(
        [cli, "scan", str(cli_project), "--json"],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    cli_report = json.loads(cli_result.stdout)

    assert cli_report["complete"] is False
    assert sdk_report["complete"] is False
    assert _normalize_report(cli_report) == _normalize_report(sdk_report)


def _normalize_knowledge(record: dict[str, object]) -> dict[str, object]:
    normalized = dict(record)
    for key in ("id", "project_id", "created_at_unix_ms"):
        normalized.pop(key, None)
    provenance = normalized.get("provenance")
    if isinstance(provenance, list):
        normalized["provenance"] = [
            {key: value for key, value in item.items() if key != "captured_at_unix_ms"}
            for item in provenance
        ]
    return normalized


def test_knowledge_sdk_persists_provenance_and_history(tmp_path: Path) -> None:
    (tmp_path / "design.md").write_text("SQLite decision\n", encoding="utf-8")
    init(tmp_path)
    scan(tmp_path)
    first = knowledge_add(
        tmp_path, type="decision", statement="Use SQLite", sources=["design.md"]
    )
    second = knowledge_add(
        tmp_path,
        type="decision",
        statement="Keep SQLite",
        sources=["design.md"],
        supersedes=first["id"],
    )

    assert first["status"] == "current"  # returned record reflects its original write
    assert second["supersedes"] == first["id"]
    assert len(knowledge_list(tmp_path)) == 1
    history = knowledge_list(tmp_path, include_history=True)
    assert [item["status"] for item in history] == ["superseded", "current"]
    assert history[0]["provenance"][0]["sha256"]

    withdrawn = knowledge_withdraw(tmp_path, second["id"])
    assert withdrawn["status"] == "withdrawn"
    assert knowledge_list(tmp_path) == []
    assert len(knowledge_list(tmp_path, include_history=True)) == 2


def test_cli_and_sdk_knowledge_add_have_normalized_parity(tmp_path: Path) -> None:
    cli = _cli_executable()
    if not cli:
        pytest.skip("Set TRAMURI_CLI to the built native executable to run CLI/SDK parity.")

    cli_project = tmp_path / "cli-project"
    sdk_project = tmp_path / "sdk-project"
    for project in (cli_project, sdk_project):
        (project / "design.md").parent.mkdir(parents=True, exist_ok=True)
        (project / "design.md").write_text("SQLite decision\n", encoding="utf-8")
        init(project)
        scan(project)

    sdk_record = knowledge_add(
        sdk_project, type="decision", statement="Use SQLite", sources=["design.md"]
    )
    cli_result = subprocess.run(
        [
            cli, "knowledge", "add", str(cli_project), "--type", "decision", "--statement",
            "Use SQLite", "--source", "design.md", "--json",
        ],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    cli_record = json.loads(cli_result.stdout)
    assert cli_result.stderr == ""
    assert _normalize_knowledge(cli_record) == _normalize_knowledge(sdk_record)


def _normalize_mission(mission: dict[str, object]) -> dict[str, object]:
    normalized = dict(mission)
    for key in ("id", "created_at_unix_ms"):
        normalized.pop(key, None)
    contract = dict(normalized["success_contract"])
    for key in ("id", "mission_id", "created_at_unix_ms"):
        contract.pop(key, None)
    contract["criteria"] = [
        {key: value for key, value in criterion.items() if key != "id"}
        for criterion in contract["criteria"]
    ]
    normalized["success_contract"] = contract
    return normalized


def test_mission_can_be_created_and_retrieved_without_scan(tmp_path: Path) -> None:
    init(tmp_path)
    created = mission_create(
        tmp_path,
        objective="Validate onboarding",
        criteria=[{"description": "Product accepts flow"}],
    )

    assert created["success_contract"]["revision"] == 1
    assert mission_get(tmp_path, mission_id=created["id"]) == created
    closed = mission_close(tmp_path, mission_id=created["id"])
    assert closed["state"] == "closed"
    assert "outcome" not in closed


def test_human_only_knowledge_does_not_require_scan(tmp_path: Path) -> None:
    init(tmp_path)
    record = knowledge_add(
        tmp_path,
        type="decision",
        statement="Keep product work repository-independent",
        human_input=True,
    )
    assert record["freshness"] == "current"
    assert record["provenance"][0]["kind"] == "human_input"


def test_cli_and_sdk_human_only_knowledge_have_parity_without_scan(tmp_path: Path) -> None:
    cli = _cli_executable()
    if not cli:
        pytest.skip("Set TRAMURI_CLI to the built native executable to run CLI/SDK parity.")

    cli_project = tmp_path / "cli-project"
    sdk_project = tmp_path / "sdk-project"
    cli_project.mkdir()
    sdk_project.mkdir()
    init(cli_project)
    init(sdk_project)
    sdk_record = knowledge_add(
        sdk_project,
        type="decision",
        statement="Keep work repository-independent",
        human_input=True,
    )
    cli_result = subprocess.run(
        [
            cli,
            "knowledge",
            "add",
            str(cli_project),
            "--type",
            "decision",
            "--statement",
            "Keep work repository-independent",
            "--human-input",
            "--json",
        ],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    assert cli_result.stderr == ""
    assert _normalize_knowledge(json.loads(cli_result.stdout)) == _normalize_knowledge(sdk_record)


def test_cli_and_sdk_mission_create_and_contract_revision_have_parity(tmp_path: Path) -> None:
    cli = _cli_executable()
    if not cli:
        pytest.skip("Set TRAMURI_CLI to the built native executable to run CLI/SDK parity.")

    cli_project = tmp_path / "cli-project"
    sdk_project = tmp_path / "sdk-project"
    cli_project.mkdir()
    sdk_project.mkdir()
    init(cli_project)
    init(sdk_project)
    criteria = [{"description": "Customer accepts integration", "required": True}]
    criteria_file = tmp_path / "criteria.json"
    criteria_file.write_text(json.dumps({"criteria": criteria}), encoding="utf-8")

    sdk_mission = mission_create(
        sdk_project, objective="Deliver customer integration", criteria=criteria
    )
    cli_result = subprocess.run(
        [
            cli,
            "mission",
            "create",
            "--path",
            str(cli_project),
            "--objective",
            "Deliver customer integration",
            "--criteria-file",
            str(criteria_file),
            "--json",
        ],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    cli_mission = json.loads(cli_result.stdout)
    assert cli_result.stderr == ""
    assert _normalize_mission(cli_mission) == _normalize_mission(sdk_mission)

    next_criteria = [*criteria, {"description": "Runbook delivered", "required": False}]
    next_file = tmp_path / "criteria-v2.json"
    next_file.write_text(json.dumps({"criteria": next_criteria}), encoding="utf-8")
    sdk_revised = mission_contract_revise(
        sdk_project, mission_id=sdk_mission["id"], criteria=next_criteria
    )
    revised_result = subprocess.run(
        [
            cli,
            "mission",
            "contract",
            "revise",
            "--path",
            str(cli_project),
            "--mission",
            cli_mission["id"],
            "--criteria-file",
            str(next_file),
            "--json",
        ],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    assert revised_result.stderr == ""
    assert _normalize_mission(json.loads(revised_result.stdout)) == _normalize_mission(sdk_revised)

    sdk_closed = mission_close(sdk_project, mission_id=sdk_mission["id"])
    cli_closed = subprocess.run(
        [
            cli,
            "mission",
            "close",
            "--path",
            str(cli_project),
            "--mission",
            cli_mission["id"],
            "--json",
        ],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    assert _normalize_mission(json.loads(cli_closed.stdout)) == _normalize_mission(sdk_closed)


def test_execution_and_external_evidence_use_rust_core(tmp_path: Path) -> None:
    init(tmp_path)
    mission = mission_create(
        tmp_path,
        objective="Validate FDE onboarding",
        criteria=[{"description": "Customer confirms onboarding", "required": True}],
    )
    execution = execution_record(
        tmp_path,
        mission_id=mission["id"],
        contract_revision=1,
        actor_kind="human",
        state="completed",
        actor_reference="operator",
    )
    assert execution["state"] == "completed"
    assert execution["ended_at_unix_ms"] is not None
    assert "outcome" not in execution

    evidence = evidence_add(
        tmp_path,
        mission_id=mission["id"],
        execution_id=execution["id"],
        kind="external_reference",
        locator="https://example.test/customer-confirmation",
    )
    assert evidence["availability"] == "unverified"
    assert evidence["locator"] == "https://example.test/customer-confirmation"

    with pytest.raises(TramuriError, match="evidence reference"):
        evidence_add(
            tmp_path,
            mission_id=mission["id"],
            execution_id=execution["id"],
            kind="external_reference",
            locator="https://user:token@example.test/private",
        )


def test_cli_and_sdk_execution_evidence_have_normalized_parity(tmp_path: Path) -> None:
    cli = _cli_executable()
    if not cli:
        pytest.skip("Set TRAMURI_CLI to the built native executable to run CLI/SDK parity.")

    cli_project = tmp_path / "cli-project"
    sdk_project = tmp_path / "sdk-project"
    for project in (cli_project, sdk_project):
        project.mkdir()
        init(project)
    criteria = [{"description": "Customer confirms"}]
    sdk_mission = mission_create(sdk_project, objective="Confirm outcome", criteria=criteria)
    criteria_file = tmp_path / "criteria.json"
    criteria_file.write_text(json.dumps({"criteria": criteria}), encoding="utf-8")
    cli_mission_result = subprocess.run(
        [cli, "mission", "create", "--path", str(cli_project), "--objective", "Confirm outcome", "--criteria-file", str(criteria_file), "--json"],
        check=True, capture_output=True, text=True, encoding="utf-8",
    )
    cli_mission = json.loads(cli_mission_result.stdout)

    sdk_execution = execution_record(sdk_project, mission_id=sdk_mission["id"], contract_revision=1,
        actor_kind="human", state="completed", actor_reference="reviewer", notes="Reviewed")
    cli_execution_result = subprocess.run(
        [cli, "execution", "record", "--path", str(cli_project), "--mission", cli_mission["id"],
         "--contract-revision", "1", "--actor-kind", "human", "--status", "completed",
         "--actor", "reviewer", "--notes", "Reviewed", "--json"],
        check=True, capture_output=True, text=True, encoding="utf-8",
    )
    cli_execution = json.loads(cli_execution_result.stdout)
    for result in (sdk_execution, cli_execution):
        assert result["state"] == "completed"
        assert result["actor_kind"] == "human"
        assert result["notes"] == "Reviewed"
        assert "outcome" not in result

    sdk_evidence = evidence_add(sdk_project, mission_id=sdk_mission["id"], execution_id=sdk_execution["id"],
        kind="human_attestation", locator="reviewed-by-operator")
    cli_evidence_result = subprocess.run(
        [cli, "evidence", "add", "--path", str(cli_project), "--mission", cli_mission["id"],
         "--execution", cli_execution["id"], "--kind", "human_attestation", "--locator", "reviewed-by-operator", "--json"],
        check=True, capture_output=True, text=True, encoding="utf-8",
    )
    cli_evidence = json.loads(cli_evidence_result.stdout)
    assert {key: sdk_evidence[key] for key in ("kind", "locator", "availability", "sha256", "hash_status")} == {
        key: cli_evidence[key] for key in ("kind", "locator", "availability", "sha256", "hash_status")
    }
