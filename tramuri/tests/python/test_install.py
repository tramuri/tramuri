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
import tramuri
from pathlib import Path
import tramuri._native
assert Path(tramuri.__file__).with_name("py.typed").is_file()
report = tramuri.init(project_path)
assert report["schema_version"] == 5
assert Path(report["database_path"]).parent.name == "database"
scan = tramuri.scan(project_path)
assert scan["complete"] is True
assert [item["path"] for item in scan["files"]] == ["hello.txt"]
"""
    subprocess.run(
        [str(python), "-c", f"project_path = {str(project)!r}\n{smoke}"],
        check=True,
        cwd=tmp_path,
        capture_output=True,
        text=True,
    )
    scan_cli = subprocess.run(
        [str(command), "scan", str(project), "--json"],
        check=True,
        capture_output=True,
        text=True,
    )
    scan_report = json.loads(scan_cli.stdout)
    assert scan_report["complete"] is True
    assert [entry["path"] for entry in scan_report["files"]] == ["hello.txt"]

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
