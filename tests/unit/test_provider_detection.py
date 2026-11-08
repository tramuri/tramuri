from types import SimpleNamespace

from tramuri.harness.providers import base


def test_probe_executable_keeps_provider_available_when_version_probe_fails(monkeypatch) -> None:
    monkeypatch.setattr(base.shutil, "which", lambda command: f"/bin/{command}")
    monkeypatch.setattr(
        base.subprocess,
        "run",
        lambda *args, **kwargs: SimpleNamespace(returncode=1, stdout="", stderr=""),
    )

    installed, executable, version = base.probe_executable("claude")

    assert installed is True
    assert executable == "/bin/claude"
    assert version == "version probe exited with code 1"


def test_probe_executable_reports_missing_provider(monkeypatch) -> None:
    monkeypatch.setattr(base.shutil, "which", lambda command: None)

    installed, executable, version = base.probe_executable("gemini")

    assert installed is False
    assert executable is None
    assert version == "gemini was not found on PATH"
