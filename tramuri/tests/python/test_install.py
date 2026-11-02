from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import venv
import pytest


def test_wheel_installs_and_sdk_imports_without_a_rust_build(tmp_path: Path) -> None:
    wheel_value = os.environ.get("TRAMURI_WHEEL")
    if not wheel_value:
        pytest.skip("Set TRAMURI_WHEEL to a built wheel to run the clean-install smoke test.")

    wheel = Path(wheel_value).resolve(strict=True)
    assert wheel.suffix == ".whl"

    environment = tmp_path / "venv"
    venv.EnvBuilder(with_pip=True).create(environment)
    python = environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    subprocess.run(
        [
            str(python),
            "-m",
            "pip",
            "install",
            "--no-index",
            "--no-deps",
            str(wheel),
        ],
        check=True,
        capture_output=True,
        text=True,
    )

    command = environment / ("Scripts/tramuri.exe" if os.name == "nt" else "bin/tramuri")
    version = subprocess.run(
        [str(command), "--version"],
        check=True,
        capture_output=True,
        text=True,
    )
    assert version.stdout.strip().startswith("tramuri ")

    uv = shutil.which("uv")
    if uv:
        tool_dir = tmp_path / "uv-tool"
        bin_dir = tmp_path / "uv-bin"
        uv_environment = os.environ.copy()
        uv_environment["UV_TOOL_DIR"] = str(tool_dir)
        uv_environment["UV_TOOL_BIN_DIR"] = str(bin_dir)
        subprocess.run(
            [
                uv,
                "tool",
                "install",
                "--python",
                sys.executable,
                str(wheel),
            ],
            check=True,
            capture_output=True,
            text=True,
            env=uv_environment,
            timeout=120,
        )
        uv_command = bin_dir / ("tramuri.exe" if os.name == "nt" else "tramuri")
        uv_version = subprocess.run(
            [str(uv_command), "--version"],
            check=True,
            capture_output=True,
            text=True,
        )
        assert uv_version.stdout.strip().startswith("tramuri ")
        uv_project = tmp_path / "uv-tool-project"
        uv_project.mkdir()
        (uv_project / "tool.txt").write_text("installed via uv tool\n", encoding="utf-8")
        subprocess.run(
            [str(uv_command), "init", str(uv_project), "--json"],
            check=True,
            capture_output=True,
            text=True,
        )
        uv_scan = subprocess.run(
            [str(uv_command), "scan", str(uv_project), "--json"],
            check=True,
            capture_output=True,
            text=True,
        )
        assert [file["path"] for file in json.loads(uv_scan.stdout)["files"]] == ["tool.txt"]

    project = tmp_path / "smoke-project"
    project.mkdir()
    source = project / "hello.txt"
    source.write_text("installed wheel smoke test\n", encoding="utf-8")
    smoke = """
import json
import tramuri
from pathlib import Path
import tramuri._native
assert callable(tramuri.execution_list)
assert callable(tramuri.execution_get)
assert callable(tramuri.evidence_list)
assert callable(tramuri.evidence_get)
assert callable(tramuri.mission_relationships)
assert callable(tramuri.mission_resume)
assert callable(tramuri.mission_workflow_activity_set)
assert callable(tramuri.mission_define_review)
assert callable(tramuri.mission_define_history)
assert callable(tramuri.mission_define_assess_brief)
assert callable(tramuri.mission_define_brief_assessment)
assert Path(tramuri.__file__).with_name("py.typed").is_file()
product_skill = Path(tramuri.__file__).parent / "skills" / "tramuri-product-engineering" / "SKILL.md"
assert product_skill.is_file()
assert "agent hypothesis" in product_skill.read_text(encoding="utf-8").lower()
for skill_name in (
    "tramuri-pe-objective-context",
    "tramuri-pe-business-discovery",
    "tramuri-pe-requirements",
    "tramuri-pe-evidence-hypotheses",
    "tramuri-pe-product-brief",
    "tramuri-pe-decision-support",
):
    define_skill = Path(tramuri.__file__).parent / "skills" / skill_name / "SKILL.md"
    assert define_skill.is_file()
    assert f"name: {skill_name}" in define_skill.read_text(encoding="utf-8")
report = tramuri.init(project_path)
assert report["schema_version"] == 10
assert Path(report["database_path"]).parent.name == "database"
exploratory = tramuri.mission_create(
    project_path,
    objective="Installed wheel exploratory Mission",
)
assert exploratory["success_contract"]["criteria"] == []
assert tramuri.outcome_get(project_path, mission_id=exploratory["id"])["state"] == "pending"
assessment = tramuri.readiness_assess(
    project_path,
    mission_id=exploratory["id"],
    activity="discover",
    state="ready",
    criteria_applied=["The investigation objective is clear"],
)
assert tramuri.readiness_get(project_path, mission_id=exploratory["id"])["assessment"] == assessment
assert len(tramuri.readiness_history(project_path, mission_id=exploratory["id"])) == 1
guided = tramuri.mission_start(
    project_path,
    objective="Create a SaaS for selling online courses to learners",
)
assert guided["workflow"]["next_activity"] == "product_framing"
updated_workflow = tramuri.mission_workflow_activity_set(
    project_path,
    mission_id=guided["mission"]["id"],
    next_activity="discover",
)
assert updated_workflow["next_activity"] == "discover"
assert updated_workflow["perspective"] == "product_engineer"
skill = Path(project_path) / ".agents" / "skills" / "release" / "SKILL.md"
skill.parent.mkdir(parents=True)
skill.write_text("---\\nname: release\\ndescription: Verify a release\\n---\\n", encoding="utf-8")
scan = tramuri.scan(project_path)
assert scan["complete"] is True
assert [item["path"] for item in scan["files"]] == [".agents/skills/release/SKILL.md", "hello.txt"]
capability = tramuri.capabilities(project_path, type="skill")["capabilities"][0]
mission = tramuri.mission_create(
    project_path,
    objective="Installed wheel recovery smoke test",
    criteria=[{"description": "Reference is recorded", "required": True}],
)
knowledge = tramuri.knowledge_add(
    project_path,
    type="decision",
    statement="Use synthetic customer data only",
    human_input=True,
)
tramuri.mission_knowledge_link(
    project_path, mission_id=mission["id"], knowledge_id=knowledge["id"], actor_reference="wheel-smoke"
)
tramuri.mission_capability_link(
    project_path, mission_id=mission["id"], capability_id=capability["id"], actor_reference="wheel-smoke"
)
execution = tramuri.execution_record(
    project_path,
    mission_id=mission["id"],
    contract_revision=1,
    actor_kind="tool",
    state="completed",
    notes="recovery test",
)
evidence = tramuri.evidence_add(
    project_path,
    mission_id=mission["id"],
    execution_id=execution["id"],
    kind="external_reference",
    locator="https://example.invalid/recovery-smoke",
)
assert tramuri.execution_list(project_path, mission_id=mission["id"]) == [execution]
assert tramuri.execution_get(project_path, execution_id=execution["id"]) == execution
assert tramuri.evidence_list(project_path, mission_id=mission["id"]) == [evidence]
assert tramuri.evidence_get(project_path, evidence_id=evidence["id"]) == evidence
resumed = tramuri.mission_resume(project_path, mission_id=mission["id"])
assert resumed["relationships"]["knowledge"][0]["knowledge"]["id"] == knowledge["id"]
assert resumed["relationships"]["capabilities"][0]["capability_id"] == capability["id"]
print(json.dumps({"mission_id": mission["id"], "execution": execution, "evidence": evidence}))
"""
    smoke_result = subprocess.run(
        [str(python), "-c", f"project_path = {str(project)!r}\n{smoke}"],
        check=True,
        cwd=tmp_path,
        capture_output=True,
        text=True,
    )
    smoke_records = json.loads(smoke_result.stdout)
    execution = smoke_records["execution"]
    evidence = smoke_records["evidence"]
    mission_id = smoke_records["mission_id"]
    listed_executions = subprocess.run(
        [str(command), "execution", "list", "--path", str(project), "--mission", mission_id, "--json"],
        check=True,
        capture_output=True,
        text=True,
    )
    assert json.loads(listed_executions.stdout) == [execution]
    shown_execution = subprocess.run(
        [str(command), "execution", "show", "--path", str(project), "--id", execution["id"], "--json"],
        check=True,
        capture_output=True,
        text=True,
    )
    assert json.loads(shown_execution.stdout) == execution
    listed_evidences = subprocess.run(
        [str(command), "evidence", "list", "--path", str(project), "--mission", mission_id, "--json"],
        check=True,
        capture_output=True,
        text=True,
    )
    assert json.loads(listed_evidences.stdout) == [evidence]
    shown_evidence = subprocess.run(
        [str(command), "evidence", "show", "--path", str(project), "--id", evidence["id"], "--json"],
        check=True,
        capture_output=True,
        text=True,
    )
    assert json.loads(shown_evidence.stdout) == evidence
    scan_cli = subprocess.run(
        [str(command), "scan", str(project), "--json"],
        check=True,
        capture_output=True,
        text=True,
    )
    scan_report = json.loads(scan_cli.stdout)
    assert scan_report["complete"] is True
    assert [entry["path"] for entry in scan_report["files"]] == [".agents/skills/release/SKILL.md", "hello.txt"]

    integration_project = tmp_path / "integration-project"
    integration_project.mkdir()
    instructions = integration_project / "AGENTS.md"
    instructions.write_text("# Existing guidance\nKeep this.\n", encoding="utf-8")
    subprocess.run(
        [
            str(command),
            "install",
            "--project",
            "--platform",
            "gemini",
            "--path",
            str(integration_project),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    installed_skill = integration_project / ".gemini" / "skills" / "tramuri" / "SKILL.md"
    assert installed_skill.is_file()
    assert "name: tramuri" in installed_skill.read_text(encoding="utf-8")
    for skill_name in (
        "tramuri-start",
        "tramuri-product-engineering",
        "tramuri-discovery",
        "tramuri-missions",
        "tramuri-integrity",
        "tramuri-pe-objective-context",
        "tramuri-pe-business-discovery",
        "tramuri-pe-requirements",
        "tramuri-pe-evidence-hypotheses",
        "tramuri-pe-product-brief",
        "tramuri-pe-decision-support",
    ):
        focused_skill = integration_project / ".gemini" / "skills" / skill_name / "SKILL.md"
        assert focused_skill.is_file()
        assert f"name: {skill_name}" in focused_skill.read_text(encoding="utf-8")
    installed_reference = (
        integration_project
        / ".gemini"
        / "skills"
        / "tramuri"
        / "references"
        / "project-discovery.md"
    )
    assert installed_reference.is_file()
    assert "Safety and interpretation" in installed_reference.read_text(encoding="utf-8")
