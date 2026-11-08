from importlib import metadata
from pathlib import Path

from typer.testing import CliRunner

from tramuri.cli.app import app
from tramuri.cli.brand import (
    _lockup,
    compact_logo,
    package_version,
    terminal_compact_logo,
    wordmark,
)


def test_brand_assets_are_versioned_and_have_compact_mono_variants() -> None:
    asset_dir = Path(__file__).parents[2] / "src" / "tramuri" / "cli" / "assets"

    assert (asset_dir / "tramuri-logo.txt").is_file()
    assert (asset_dir / "tramuri-logo-compact.txt").is_file()
    assert (asset_dir / "tramuri-logo-mono.txt").is_file()
    assert (asset_dir / "tramuri-logo-ascii.txt").is_file()
    assert "█" in (asset_dir / "tramuri-logo.txt").read_text(encoding="utf-8")
    assert compact_logo()
    assert len(wordmark().splitlines()) == 5


def test_brand_version_comes_from_package_metadata() -> None:
    assert package_version() == metadata.version("tramuri")


def test_narrow_terminal_uses_compact_lockup_without_ellipsis() -> None:
    assert all(len(line) <= 50 for line in _lockup(50, True).splitlines())


def test_legacy_windows_encoding_uses_ascii_logo_fallback() -> None:
    ascii_logo = terminal_compact_logo("cp1252")

    assert ascii_logo
    assert ascii_logo.isascii()
    assert compact_logo().isascii() is False


def test_init_presents_branded_real_installation_summary(tmp_path: Path) -> None:
    result = CliRunner().invoke(
        app, ["--project-dir", str(tmp_path), "init", "--provider", "codex"]
    )

    assert result.exit_code == 0, result.stdout
    assert "TRAMURI" in result.stdout
    assert package_version() in result.stdout
    assert "Codex" in result.stdout
    assert ".agents/skills/tramuri/SKILL.md" in result.stdout
    assert "/tramuri ." in result.stdout
    assert "$tramuri ." in result.stdout
    assert "Next" in result.stdout


def test_install_help_keeps_brand_identity_and_no_color_is_clean(tmp_path: Path) -> None:
    result = CliRunner().invoke(
        app,
        ["--project-dir", str(tmp_path), "--no-color", "install", "--provider", "codex"],
    )

    assert result.exit_code == 0, result.stdout
    assert "TRAMURI" in result.stdout
    assert "\\x1b[" not in result.stdout
    assert "Product companion" in result.stdout


def test_root_help_includes_brand_identity() -> None:
    result = CliRunner().invoke(app, ["--help"])

    assert result.exit_code == 0
    assert "TRAMURI" in result.stdout
    assert package_version() in result.stdout
