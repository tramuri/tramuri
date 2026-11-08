from importlib.metadata import files, version
from importlib.resources import files as resource_files

from tramuri import __version__
from tramuri.core.contracts import HookDecision
from tramuri.harness import adapters
from tramuri.modules.pm.runner import PMRunner


def test_package_identity_and_version() -> None:
    assert version("tramuri") == __version__
    assert __version__ == "0.13.2"


def test_public_core_harness_and_module_boundaries_import() -> None:
    assert HookDecision and PMRunner
    assert {adapter.key for adapter in adapters()} == {"codex", "claude", "gemini"}


def test_legacy_facade_packages_are_not_part_of_the_public_package() -> None:
    package_files = {str(path).replace("\\", "/") for path in (files("tramuri") or [])}
    assert not any(path.startswith("tramuri/pm/") for path in package_files)
    assert not any(path.startswith("tramuri/providers/") for path in package_files)


def test_terminal_brand_assets_are_packaged() -> None:
    assets = resource_files("tramuri.cli.assets")
    assert assets.joinpath("tramuri-logo.txt").is_file()
    assert assets.joinpath("tramuri-logo-compact.txt").is_file()
    assert assets.joinpath("tramuri-logo-mono.txt").is_file()
    assert assets.joinpath("tramuri-logo-ascii.txt").is_file()
