from importlib.metadata import version

from tramuri import __version__


def test_package_identity_and_version() -> None:
    assert version("tramuri") == __version__
    assert __version__ == "0.13.0"
