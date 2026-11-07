"""Tramuri command-line interface."""

from __future__ import annotations

import webbrowser
from pathlib import Path
from typing import Any

import typer

from tramuri.cli.output import banner, console, emit, stage_table, version_text
from tramuri.core.errors import TramuriError, UsageError
from tramuri.core.models import Mode
from tramuri.core.safety import project_root
from tramuri.pm.demo import run_demo
from tramuri.pm.runner import PMRunner, ProgressCallback
from tramuri.providers import adapter_for, adapters, registry
from tramuri.providers.installer import Installer
from tramuri.reports.generator import ReportGenerator
from tramuri.sensors.checks import check_run
from tramuri.storage.database import Database

app = typer.Typer(
    help="Local-first Product Management workflows for Tramuri.",
    no_args_is_help=True,
    rich_markup_mode="markdown",
)
providers_app = typer.Typer(help="Inspect and manage AI-agent provider integrations.")
runs_app = typer.Typer(help="Inspect persisted Tramuri runs.")
pm_app = typer.Typer(help="Run Product Management workflows.")
report_app = typer.Typer(help="Generate and open offline reports.")
app.add_typer(providers_app, name="providers")
app.add_typer(runs_app, name="runs")
app.add_typer(pm_app, name="pm")
app.add_typer(report_app, name="report")


def _version_callback(value: bool) -> None:
    if value:
        typer.echo(version_text())
        raise typer.Exit()


@app.callback()
def callback(
    ctx: typer.Context,
    project_dir: Path = typer.Option(
        Path.cwd(), "--project-dir", help="Project directory for local state."
    ),
    json_output: bool = typer.Option(False, "--json", help="Emit machine-readable JSON."),
    no_color: bool = typer.Option(False, "--no-color", help="Disable ANSI styling."),
    version: bool = typer.Option(False, "--version", callback=_version_callback, is_eager=True),
) -> None:
    ctx.ensure_object(dict)
    ctx.obj.update(
        {"project_dir": project_root(project_dir), "json": json_output, "no_color": no_color}
    )


def _ctx(ctx: typer.Context) -> dict[str, Any]:
    return ctx.ensure_object(dict)


def _database(ctx: typer.Context) -> Database:
    return Database(_ctx(ctx)["project_dir"])


def _mode(value: str) -> Mode:
    try:
        return Mode(value.lower())
    except ValueError as exc:
        raise typer.BadParameter("mode must be poc, mvp or enterprise") from exc


def _selected_provider_names(provider: str, project_dir: Path) -> list[str]:
    if provider.lower() == "all":
        return [item.key for item in adapters()]
    if provider.lower() == "auto":
        return [item.provider for item in registry().detect(project_dir) if item.installed]
    try:
        adapter_for(provider)
    except KeyError as exc:
        raise UsageError(f"Unknown provider: {provider}") from exc
    return [provider.lower()]


def _progress(ctx: typer.Context) -> ProgressCallback:
    def callback(stage: str, status: str, owner: str) -> None:
        if not _ctx(ctx)["json"]:
            symbol = {"running": "[~]", "completed": "[x]", "failed": "[!]"}.get(status, "[ ]")
            console(_ctx(ctx)["no_color"]).print(f"{symbol} {stage} · {owner} · {status}")

    return callback


@app.command()
def init(
    ctx: typer.Context,
    provider: str = typer.Option("auto", "--provider", help="codex, claude, gemini, all or auto"),
    dry_run: bool = typer.Option(False, "--dry-run"),
    force: bool = typer.Option(False, "--force"),
) -> None:
    """Detect providers and install project-local Tramuri components."""
    if not _ctx(ctx)["json"]:
        banner(_ctx(ctx)["no_color"])
    project_dir = _ctx(ctx)["project_dir"]
    database = _database(ctx)
    detected = registry().detect(project_dir)
    selected = _selected_provider_names(provider, project_dir)
    results = []
    for item in detected:
        if not _ctx(ctx)["json"]:
            console(_ctx(ctx)["no_color"]).print(
                f"{item.provider}: {'available' if item.installed else 'not detected'}"
            )
    for name in selected:
        results.append(
            Installer(project_dir, database).install(
                adapter_for(name).installation_plan(project_dir), dry_run=dry_run, force=force
            )
        )
    payload = {
        "project_dir": str(project_dir),
        "selected": selected,
        "detected": [
            {
                "provider": item.provider,
                "installed": item.installed,
                "executable": item.executable,
                "version": item.version,
                "paths": item.project_paths,
                "details": item.details,
            }
            for item in detected
        ],
        "results": results,
    }
    emit(payload, _ctx(ctx)["json"], _ctx(ctx)["no_color"])


@app.command()
def doctor(ctx: typer.Context) -> None:
    """Diagnose provider availability and persist capability observations."""
    project_dir = _ctx(ctx)["project_dir"]
    database = _database(ctx)
    detections = registry().detect(project_dir)
    capabilities = registry().as_dicts(project_dir)
    database.save_capabilities(capabilities)
    payload = {
        "providers": [
            {
                "provider": item.provider,
                "installed": item.installed,
                "executable": item.executable,
                "version": item.version,
                "paths": item.project_paths,
                "details": item.details,
            }
            for item in detections
        ],
        "capabilities": capabilities,
    }
    if _ctx(ctx)["json"]:
        emit(payload, True)
    else:
        banner(_ctx(ctx)["no_color"])
        for item in detections:
            console(_ctx(ctx)["no_color"]).print(
                f"{item.provider}: {'available' if item.installed else 'missing'} — {item.details}"
            )


@providers_app.command("list")
def providers_list(ctx: typer.Context) -> None:
    doctor(ctx)


@app.command()
def install(
    ctx: typer.Context,
    provider: str = typer.Option(..., "--provider"),
    dry_run: bool = typer.Option(False, "--dry-run"),
    force: bool = typer.Option(False, "--force"),
) -> None:
    """Install one provider's managed components."""
    names = _selected_provider_names(provider, _ctx(ctx)["project_dir"])
    results = [
        Installer(_ctx(ctx)["project_dir"], _database(ctx)).install(
            adapter_for(name).installation_plan(_ctx(ctx)["project_dir"]),
            dry_run=dry_run,
            force=force,
        )
        for name in names
    ]
    emit({"results": results}, _ctx(ctx)["json"], _ctx(ctx)["no_color"])


@app.command()
def update(
    ctx: typer.Context,
    provider: str = typer.Option("all", "--provider"),
    force: bool = typer.Option(False, "--force"),
) -> None:
    """Reconcile managed components with current Tramuri templates."""
    install(ctx, provider, False, force)


@app.command()
def uninstall(
    ctx: typer.Context,
    provider: str = typer.Option(..., "--provider"),
    force: bool = typer.Option(False, "--force"),
) -> None:
    """Remove only unchanged Tramuri-managed files."""
    names = _selected_provider_names(provider, _ctx(ctx)["project_dir"])
    results = [
        Installer(_ctx(ctx)["project_dir"], _database(ctx)).uninstall(name, force=force)
        for name in names
    ]
    emit({"results": results}, _ctx(ctx)["json"], _ctx(ctx)["no_color"])


@app.command()
def status(ctx: typer.Context) -> None:
    """Show the most recent local run and provider manifests."""
    database = _database(ctx)
    runs = database.list_runs()
    payload = {
        "project_dir": str(_ctx(ctx)["project_dir"]),
        "runs": runs,
        "capabilities": database.list_capabilities(),
        "manifests": database.list_manifests(),
    }
    if _ctx(ctx)["json"]:
        emit(payload, True)
    else:
        banner(_ctx(ctx)["no_color"])
        emit(
            f"Project: {_ctx(ctx)['project_dir']}\nRuns: {len(runs)}\n"
            f"Capabilities: {len(payload['capabilities'])}\nManifests: {len(payload['manifests'])}",
            False,
            _ctx(ctx)["no_color"],
        )


@runs_app.command("list")
def runs_list(ctx: typer.Context) -> None:
    emit(_database(ctx).list_runs(), _ctx(ctx)["json"], _ctx(ctx)["no_color"])


@runs_app.command("show")
def runs_show(ctx: typer.Context, run_id: str) -> None:
    details = _database(ctx).run_details(run_id)
    if _ctx(ctx)["json"]:
        emit(details, True)
    else:
        console(_ctx(ctx)["no_color"]).print(stage_table(details["stages"], _ctx(ctx)["no_color"]))


@app.command()
def resume(ctx: typer.Context, run_id: str) -> None:
    """Resume pending or failed stages of an existing run."""
    database = _database(ctx)
    result = PMRunner(database, _progress(ctx)).resume(run_id)
    details = database.run_details(result)
    emit(
        details if _ctx(ctx)["json"] else f"Resumed {result} ({details['run']['status']})",
        _ctx(ctx)["json"],
        _ctx(ctx)["no_color"],
    )


@pm_app.command("run")
def pm_run(
    ctx: typer.Context,
    idea_file: Path = typer.Option(..., "--idea-file", exists=True, dir_okay=False),
    mode: str = typer.Option("mvp", "--mode"),
    provider: str = typer.Option("local-demo", "--provider"),
) -> None:
    """Run the PM workflow from a local idea file."""
    idea = idea_file.read_text(encoding="utf-8")
    database = _database(ctx)
    run_id = PMRunner(database, _progress(ctx)).run(idea, _mode(mode), provider)
    details = database.run_details(run_id)
    reports = ReportGenerator(_ctx(ctx)["project_dir"]).generate(details)
    emit(
        {"run_id": run_id, "status": details["run"]["status"], "reports": reports},
        _ctx(ctx)["json"],
        _ctx(ctx)["no_color"],
    )


@pm_app.command("demo")
def pm_demo(ctx: typer.Context, mode: str = typer.Option("mvp", "--mode")) -> None:
    """Run the deterministic appointment-service demo offline."""
    database = _database(ctx)
    run_id = run_demo(database, _mode(mode), _progress(ctx))
    details = database.run_details(run_id)
    reports = ReportGenerator(_ctx(ctx)["project_dir"]).generate(details)
    payload = {
        "demo": True,
        "synthetic": True,
        "run_id": run_id,
        "status": details["run"]["status"],
        "reports": reports,
        "sensors": check_run(details),
    }
    emit(payload, _ctx(ctx)["json"], _ctx(ctx)["no_color"])


@report_app.command("open")
def report_open(
    ctx: typer.Context, run_id: str, no_open: bool = typer.Option(False, "--no-open")
) -> None:
    """Regenerate and optionally open a run's offline HTML report."""
    details = _database(ctx).run_details(run_id)
    paths = ReportGenerator(_ctx(ctx)["project_dir"]).generate(details)
    if not no_open:
        webbrowser.open(Path(paths["report"]).as_uri())
    emit(paths, _ctx(ctx)["json"], _ctx(ctx)["no_color"])


def main() -> None:
    try:
        app()
    except TramuriError as exc:
        typer.echo(f"tramuri: {exc}", err=True)
        raise typer.Exit(code=2) from exc
