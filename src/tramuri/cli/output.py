"""Terminal and JSON presentation helpers."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.text import Text

from tramuri import __version__
from tramuri.cli.brand import (
    banner as brand_banner,
)
from tramuri.cli.brand import (
    installation_table,
    next_steps,
)


def console(no_color: bool = False) -> Console:
    return Console(no_color=no_color, force_terminal=False)


def banner(no_color: bool = False) -> None:
    brand_banner(console(no_color), no_color)


def installation_summary(
    payload: dict[str, Any], no_color: bool = False, show_next_steps: bool = True
) -> None:
    """Render a real provider/component installation summary."""
    output = console(no_color)
    detected = payload.get("detected", [])
    results = payload.get("results", [])
    detected_items = detected if isinstance(detected, list) else []
    result_items = results if isinstance(results, list) else []
    installation_table(output, detected_items, result_items, Path(str(payload["project_dir"])))
    if show_next_steps:
        selected = payload.get("selected", [])
        selected_items = [str(item) for item in selected] if isinstance(selected, list) else []
        dry_run = any(isinstance(item, dict) and bool(item.get("dry_run")) for item in result_items)
        next_steps(output, selected_items, dry_run)


def emit(value: Any, json_output: bool = False, no_color: bool = False) -> None:
    if json_output:
        print(json.dumps(value, ensure_ascii=False, default=str))
    else:
        console(no_color).print(value)


def version_text() -> str:
    return f"tramuri {__version__}"


def stage_table(stages: list[dict[str, Any]], no_color: bool = False) -> Table:
    table = Table(title="TRAMURI / PRODUCT STAGES", box=None, show_header=True)
    table.add_column("State", width=8)
    table.add_column("Stage")
    table.add_column("Agent")
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
            Text(symbols.get(state, "[?]")),
            str(stage.get("stage_key", stage.get("key", "unknown"))).replace("_", " ").title(),
            str(stage.get("agent_id") or stage.get("owner") or "-"),
            str(stage.get("attempts", 0)),
        )
    return table


def render_status(view: dict[str, Any], no_color: bool = False) -> None:
    """Render the persisted status projection with a truthful next action."""
    output = console(no_color)
    semantic = str(view.get("semantic_status", "unknown"))
    palette = {
        "completed": "green",
        "running": "cyan",
        "ready": "cyan",
        "waiting": "yellow",
        "stale": "yellow",
        "blocked": "red",
        "failed": "red",
    }
    color = "white" if no_color else palette.get(semantic, "white")
    progress = view.get("progress", {})
    gates = view.get("gates", {})
    counts = view.get("counts", {})
    table = Table(box=None, show_header=False, padding=(0, 1))
    table.add_column("Fact", style="dim" if not no_color else None, no_wrap=True)
    table.add_column("Value")
    table.add_row(
        "Status", f"[{color}]{semantic.upper()}[/{color}]" if not no_color else semantic.upper()
    )
    table.add_row("Run", str(view.get("run_id") or "No run selected"))
    table.add_row(
        "Progress",
        f"{progress.get('completed', 0)}/{progress.get('total', 0)} stages ({progress.get('percent', 0)}%)",
    )
    table.add_row("Current", str(view.get("current_stage") or "-"))
    table.add_row("Last completed", str(view.get("last_completed_stage") or "-"))
    table.add_row("Next action", str(view.get("next_action") or "-"))
    research = view.get("research", {})
    table.add_row(
        "Research",
        f"{research.get('status', 'not_started')} · {research.get('source_count', 0)} source(s)",
    )
    agents = view.get("agents", {})
    table.add_row(
        "Delegation",
        f"{agents.get('delegated', 0)} stage(s) · {', '.join(agents.get('execution_modes', [])) or 'not recorded'} · "
        f"{', '.join(agents.get('invocation_statuses', [])) or 'unknown'}",
    )
    experience = view.get("experience", {})
    table.add_row(
        "Experience",
        f"{', '.join(experience.get('surfaces', [])) or 'not generated'} · browser {experience.get('browser', 'not_run')}",
    )
    table.add_row(
        "Gates",
        f"{gates.get('blocking_failures', 0)} blocking | {gates.get('pending_validation', 0)} pending validation",
    )
    table.add_row(
        "Evidence",
        f"{counts.get('artifacts', 0)} artifacts | {counts.get('sensors', 0)} sensor receipts | "
        f"{counts.get('knowledge', 0)} knowledge entries",
    )
    output.print(Panel(table, title="TRAMURI / WORKFLOW STATUS", border_style=color))
    output.print(stage_table(view.get("stages", []), no_color))


def render_explanation(view: dict[str, Any], no_color: bool = False) -> None:
    """Render a provenance-aware explanation without semantic invention."""
    output = console(no_color)
    subject = str(view.get("subject", "current product"))
    header = Table(box=None, show_header=False, padding=(0, 1))
    header.add_column("Fact", style="dim" if not no_color else None, no_wrap=True)
    header.add_column("Value")
    header.add_row("Subject", Text(subject))
    header.add_row("Run", Text(str(view.get("run_id") or "No run selected")))
    header.add_row("Status", Text(str(view.get("run_status", "unknown"))))
    header.add_row("Found", Text(str(view.get("found", False))))
    output.print(Panel(header, title="TRAMURI / EXPLAIN", border_style="cyan"))

    sources = view.get("sources", [])
    source_table = Table(title="Sources", box=None)
    source_table.add_column("Artifact")
    source_table.add_column("Stage")
    source_table.add_column("Classification")
    source_table.add_column("Revision", justify="right")
    for source in sources if isinstance(sources, list) else []:
        if isinstance(source, dict):
            source_table.add_row(
                Text(str(source.get("artifact", "unknown"))),
                Text(str(source.get("stage", "unknown"))),
                Text(str(source.get("classification", "unknown"))),
                Text(str(source.get("revision", "-"))),
            )
    output.print(source_table)

    connections = view.get("connections", [])
    connection_table = Table(title="Connections", box=None)
    connection_table.add_column("From")
    connection_table.add_column("Relation")
    connection_table.add_column("To")
    connection_table.add_column("Evidence")
    for connection in connections if isinstance(connections, list) else []:
        if isinstance(connection, dict):
            connection_table.add_row(
                Text(str(connection.get("from", "unknown"))),
                Text(str(connection.get("relation", "unknown"))),
                Text(str(connection.get("to", "unknown"))),
                Text(str(connection.get("evidence", "unknown"))),
            )
    output.print(connection_table)

    unknowns = view.get("unknowns", [])
    if isinstance(unknowns, list) and unknowns:
        output.print(Panel(Text("\n".join(str(item) for item in unknowns)), title="Unknowns"))
