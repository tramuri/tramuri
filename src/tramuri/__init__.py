"""Tramuri: from intent to a verifiable product definition."""

from importlib.metadata import PackageNotFoundError, version

try:
    __version__ = version("tramuri")
except PackageNotFoundError:  # pragma: no cover - source checkout fallback
    __version__ = "0.14.0"

__all__ = ["__version__"]
