"""Terminal brand system for Tramuri's CLI surfaces."""

from __future__ import annotations

import sys
from importlib import metadata
from importlib.metadata import PackageNotFoundError
from pathlib import Path

from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.text import Text

from tramuri import __version__

ASSET_DIR = Path(__file__).with_name("assets")
COLORS = {
    "deep_blue": "#0B2530",
    "turquoise": "#00B3A6",
    "petrol": "#006D6A",
    "light_turquoise": "#4FD6C7",
    "orange": "#FFA229",
    "soft_green": "#A8EFE1",
}

_WORDMARK = (
    "█████  ████  █   █  █   █  █   █  ████   █████",
    "  █    █  █  ██ ██  ██ ██  █   █  █  █     █  ",
    "  █    ████  █ █ █  █ █ █  █   █  ████     █  ",
    "  █    █ █   █   █  █   █  █   █  █ █      █  ",
    "  █    █  ██  █   █  █   █   █   █  █  █  █████",
)


def package_version() -> str:
    """Return the installed distribution version with an editable fallback."""
    try:
        return metadata.version("tramuri")
    except PackageNotFoundError:
        return __version__


def _asset(name: str) -> str:
    return (ASSET_DIR / name).read_text(encoding="utf-8").rstrip("\n")


def logo(compact: bool = False, monochrome: bool = False, ascii_only: bool = False) -> str:
    """Load the reviewed terminal logo variant from packaged text assets."""
    if ascii_only:
        return _asset("tramuri-logo-ascii.txt")
    if monochrome:
        return _asset("tramuri-logo-mono.txt")
    return _asset("tramuri-logo-compact.txt" if compact else "tramuri-logo.txt")


def compact_logo(ascii_only: bool = False) -> str:
    """Return the compact logo used by narrow terminals and tests."""
    return logo(compact=True, ascii_only=ascii_only)


def terminal_compact_logo(encoding: str | None = None) -> str:
    """Return Unicode art only when the output encoding can represent it."""
    return compact_logo(ascii_only=not supports_unicode(encoding))


def supports_unicode(encoding: str | None = None) -> bool:
    """Check whether a terminal encoding can safely receive the brand art."""
    value = (encoding or getattr(sys.stdout, "encoding", None) or "utf-8").lower()
    return value.replace("-", "").replace("_", "") in {"utf8", "cp65001"}


def wordmark() -> str:
    """Return the five-row block-letter Tramuri wordmark."""
    return "\n".join(_WORDMARK)


def _styled(text: str, color: str, no_color: bool) -> str:
    return text if no_color else f"[bold {color}]{text}[/bold {color}]"


def _lockup(width: int, no_color: bool, encoding: str | None = None) -> str:
    ascii_only = not supports_unicode(encoding)
    logo_lines = logo(compact=width < 100, ascii_only=ascii_only).splitlines()
    word_lines = wordmark().splitlines()
    if ascii_only:
        word_lines = ["TRAMURI"]
    if width < 60:
        return "\n".join(
            [_styled(line, COLORS["turquoise"], no_color) for line in logo_lines]
            + [""]
            + [_styled("TRAMURI", COLORS["light_turquoise"], no_color)]
        )
    rows = []
    logo_width = max(len(line) for line in logo_lines)
    for index in range(max(len(logo_lines), len(word_lines))):
        word_line = word_lines[index] if index < len(word_lines) else ""
        mark = logo_lines[index] if index < len(logo_lines) else ""
        rows.append(
            f"{_styled(mark.ljust(logo_width), COLORS['turquoise'], no_color)}   "
            f"{_styled(word_line, COLORS['light_turquoise'], no_color)}"
        )
    return "\n".join(rows)


def banner(console: Console, no_color: bool = False) -> None:
    """Render the compact Tramuri identity lockup."""
    content = Text.from_markup(
        _lockup(console.width, no_color, console.encoding)
        + "\n\n"
        + _styled("From intent to verified outcomes.", COLORS["soft_green"], no_color)
        + "  "
        + _styled(f"v{package_version()}", COLORS["orange"], no_color)
    )
    console.print(
        Panel(
            content,
            title="TRAMURI"
            if no_color
            else f"[bold {COLORS['turquoise']}]TRAMURI[/bold {COLORS['turquoise']}]",
            title_align="left",
            border_style=COLORS["petrol"],
            padding=(1, 2),
            expand=False,
        )
    )


def installation_table(
    console: Console,
    detected: list[dict[str, object]],
    results: list[dict[str, object]],
    project_dir: Path,
) -> None:
    """Show provider capabilities and the components really planned/installed."""
    table = Table(title="Tramuri / installation", show_lines=False, pad_edge=False)
    table.add_column("Provider", style=COLORS["light_turquoise"])
    table.add_column("Status")
    table.add_column("Components")
    table.add_column("Project", overflow="ellipsis")
    by_provider = {str(item.get("provider")): item for item in results}
    for item in detected:
        provider = str(item.get("provider", "unknown"))
        result = by_provider.get(provider, {})
        components = result.get("components", [])
        component_items = components if isinstance(components, list) else []
        paths = [
            str(component.get("path"))
            for component in component_items
            if isinstance(component, dict) and component.get("path")
        ]
        installed = bool(item.get("installed"))
        status = "available" if installed else "not detected"
        if result:
            status = "dry-run" if result.get("dry_run") else "installed"
        table.add_row(
            provider.capitalize(),
            status,
            str(len(paths)) if paths else "—",
            str(project_dir),
        )
        if paths:
            console.print(
                f"  [bold {COLORS['turquoise']}]Product companion[/bold {COLORS['turquoise']}]"
                f" · {provider.capitalize()} · "
                + ", ".join(paths[:3])
                + (f" (+{len(paths) - 3})" if len(paths) > 3 else "")
            )
    console.print(table)


def next_steps(console: Console, selected: list[str], dry_run: bool = False) -> None:
    """Explain the next action in a newly installed project."""
    if dry_run:
        message = "Review the plan, then run the same command without --dry-run."
    elif selected:
        message = (
            "Open your assistant and run `/tramuri .` "
            "(Codex: `$tramuri .`). Results go to `tramuri_output/`."
        )
    else:
        message = (
            "No provider was detected. Auto only installs providers whose CLI is available. "
            "Choose one explicitly: `tramuri install --provider codex|claude|gemini`."
        )
    console.print(
        Panel(
            f"[bold {COLORS['orange']}]Next[/bold {COLORS['orange']}]  {message}",
            border_style=COLORS["petrol"],
            padding=(0, 1),
            expand=False,
        )
    )
