"""Terminal and JSON presentation helpers."""

from __future__ import annotations

import json
from typing import Any

from rich.console import Console
from rich.panel import Panel
from rich.table import Table

from tramuri import __version__


def console(no_color: bool = False) -> Console:
    return Console(no_color=no_color, force_terminal=False)


def banner(no_color: bool = False) -> None:
    console(no_color).print(
        Panel.fit(
            "[bold #00B3A6]TRAMURI[/bold #00B3A6]\n[dim]From intent to verified outcomes.[/dim]",
            border_style="#006D6A",
        )
    )


def emit(value: Any, json_output: bool = False, no_color: bool = False) -> None:
    if json_output:
        print(json.dumps(value, ensure_ascii=False, default=str))
    else:
        console(no_color).print(value)


def version_text() -> str:
    return f"tramuri {__version__}"


def stage_table(stages: list[dict[str, Any]], no_color: bool = False) -> Table:
    table = Table(title="Product Management workflow", box=None, show_header=True)
    table.add_column("State", width=8)
    table.add_column("Stage")
    table.add_column("Owner")
    table.add_column("Attempts", justify="right")
    symbols = {
        "pending": "[ ]",
        "running": "[~]",
        "completed": "[x]",
        "blocked": "[!]",
        "failed": "[!]",
        "skipped": "[-]",
    }
    for stage in stages:
        state = str(stage["status"])
        table.add_row(
            symbols.get(state, "[?]"),
            str(stage["stage_key"]),
            str(stage["owner"]),
            str(stage["attempts"]),
        )
    return table
