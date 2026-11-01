"""Console entry point; argument parsing and command execution stay in Rust."""

from __future__ import annotations

import sys

from ._native import _cli_main


def main() -> None:
    raise SystemExit(_cli_main(sys.argv))
