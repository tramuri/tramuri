"""Tramuri command-line interface."""

from __future__ import annotations

import webbrowser
from pathlib import Path
from typing import Any

import typer
from rich.text import Text

from tramuri.cli.brand import package_version, terminal_compact_logo
from tramuri.cli.output import (
    banner,
    console,
    emit,
    installation_summary,
    render_explanation,
    render_status,
    stage_table,
    version_text,
)
from tramuri.core.errors import TramuriError, UsageError
from tramuri.core.explain import explain_run
from tramuri.core.models import InstallationPlan, Mode
from tramuri.core.paths import output_root
from tramuri.core.safety import project_root
from tramuri.core.status import build_status
from tramuri.experience.browser import run_browser_validation
from tramuri.experience.classifier import ExperienceCategory
from tramuri.experience.lifecycle import ExperienceLifecycle
from tramuri.experience.models import SensorResult
from tramuri.experience.reviewer import ProductLeadReviewer
from tramuri.harness import adapter_for, adapters, registry
from tramuri.harness.installer import Installer
from tramuri.harness.projection import project_manifest
from tramuri.hooks.runtime import main as hook_runtime_main
from tramuri.knowledge.memory import MemoryService
from tramuri.modules.pm.demo import run_demo
from tramuri.modules.pm.research import import_research_pack
from tramuri.modules.pm.runner import PMRunner, ProgressCallback
from tramuri.reports.generator import ReportGenerator
from tramuri.sensors.checks import check_experience, check_run
from tramuri.storage.database import Database

app = typer.Typer(
    help=f"TRAMURI v{package_version()} · local-first product workflows.",
    epilog=f"```text\n{terminal_compact_logo()}\n```\n\nFrom intent to verified outcomes.",
    no_args_is_help=True,
    rich_markup_mode="markdown",
)
providers_app = typer.Typer(help="Inspect and manage AI-agent provider integrations.")
runs_app = typer.Typer(help="Inspect persisted Tramuri runs.")
product_app = typer.Typer(help="Run product workflows.")
report_app = typer.Typer(help="Generate and open offline reports.")
hooks_app = typer.Typer(help="Run deterministic provider hook handlers.")
experience_app = typer.Typer(help="Inspect Product Experience artifacts and quality checks.")
memory_app = typer.Typer(help="Save and retrieve curated project memory.")
app.add_typer(providers_app, name="providers")
app.add_typer(runs_app, name="runs")
app.add_typer(product_app, name="product")
app.add_typer(report_app, name="report")
app.add_typer(hooks_app, name="hooks")
app.add_typer(experience_app, name="experience")
app.add_typer(memory_app, name="memory")


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
            del owner
            label = stage.replace("_", " ").title()
            console(_ctx(ctx)["no_color"]).print(Text(f"{symbol} {label} | {status}"))

    return callback


def _installation_for(name: str, project_dir: Path) -> InstallationPlan:
    adapter = adapter_for(name)
    manifest_factory = getattr(adapter, "manifest", None)
    if callable(manifest_factory):
        return project_manifest(manifest_factory(project_dir), project_dir)
    return adapter.installation_plan(project_dir)


def _record_experience_delivery(
    lifecycle: ExperienceLifecycle,
    run_status: str,
    sensors: list[dict[str, Any]],
    review: dict[str, Any],
    browser: SensorResult,
) -> dict[str, Any]:
    """Persist correction and delivery status from the latest experience checks."""
    browser_payload = browser.to_dict()
    sensors_passed = bool(sensors) and all(item.get("status") == "passed" for item in sensors)
    review_passed = review.get("status") == "passed"
    complete = run_status == "completed" and sensors_passed and review_passed
    delivery_status = (
        "completed"
        if complete and browser.status == "passed"
        else "completed_with_browser_not_run"
        if complete and browser.status == "not_run"
        else "blocked"
    )
    lifecycle.record(
        "correction",
        {"status": "not_required" if delivery_status.startswith("completed") else "required"},
    )
    payload = {
        "status": delivery_status,
        "sensors": sensors,
        "browser": browser_payload,
        "review": review,
    }
    lifecycle.record("delivery", payload)
    return payload


def _record_experience(database: Database, run_id: str, reports: dict[str, Any]) -> dict[str, Any]:
    lifecycle = ExperienceLifecycle(database, run_id)
    run_status = str(database.run_details(run_id)["run"].get("status", "unknown"))
    review = reports.get("review", {})
    lifecycle.record(
        "product_definition",
        {"status": "completed" if run_status == "completed" else run_status},
    )
    lifecycle.record("product_review", review)
    lifecycle.record("demo_generation", {"status": "completed", "path": reports.get("demo")})
    browser = run_browser_validation(Path(str(reports["demo"])))
    lifecycle.record("browser_validation", browser.to_dict())
    lifecycle.record("critical_review", review)
    sensor_statuses = reports.get("experience_sensors", [])
    _record_experience_delivery(lifecycle, run_status, sensor_statuses, review, browser)
    return browser.to_dict()


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
    for name in selected:
        results.append(
            Installer(project_dir, database).install(
                _installation_for(name, project_dir), dry_run=dry_run, force=force
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
    if _ctx(ctx)["json"]:
        emit(payload, True)
    else:
        installation_summary(payload, _ctx(ctx)["no_color"])


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
            _installation_for(name, _ctx(ctx)["project_dir"]),
            dry_run=dry_run,
            force=force,
        )
        for name in names
    ]
    payload = {
        "project_dir": str(_ctx(ctx)["project_dir"]),
        "selected": names,
        "detected": [
            {
                "provider": item.provider,
                "installed": item.installed,
                "executable": item.executable,
                "version": item.version,
                "paths": item.project_paths,
                "details": item.details,
            }
            for item in registry().detect(_ctx(ctx)["project_dir"])
            if item.provider in names
        ],
        "results": results,
    }
    if _ctx(ctx)["json"]:
        emit({"results": results}, True)
    else:
        banner(_ctx(ctx)["no_color"])
        installation_summary(payload, _ctx(ctx)["no_color"])


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
def status(
    ctx: typer.Context,
    run_id: str | None = typer.Argument(None, help="Optional run id to inspect."),
) -> None:
    """Show a read-only, actionable projection of local workflow state."""
    database = _database(ctx)
    runs = database.list_runs()
    selected_run = run_id or (str(runs[0]["id"]) if runs else None)
    details = database.run_details(selected_run) if selected_run else None
    view = build_status(
        details,
        checkpoint=database.latest_checkpoint(selected_run) if selected_run else None,
    )
    payload = view | {
        "project_dir": str(_ctx(ctx)["project_dir"]),
        "runs": runs,
        "capabilities": database.list_capabilities(),
        "manifests": database.list_manifests(),
    }
    if _ctx(ctx)["json"]:
        emit(payload, True)
    else:
        banner(_ctx(ctx)["no_color"])
        render_status(view, _ctx(ctx)["no_color"])
        emit(
            f"Project: {_ctx(ctx)['project_dir']}\nRuns: {len(runs)}\n"
            f"Capabilities: {len(payload['capabilities'])}\nManifests: {len(payload['manifests'])}",
            False,
            _ctx(ctx)["no_color"],
        )


@app.command()
def explain(
    ctx: typer.Context,
    subject: str | None = typer.Argument(None, help="Product concept or decision to explain."),
    run_id: str | None = typer.Option(None, "--run-id", help="Run to inspect."),
) -> None:
    """Explain a persisted product concept and its recorded connections."""
    database = _database(ctx)
    runs = database.list_runs()
    selected_run = run_id or (str(runs[0]["id"]) if runs else None)
    details = database.run_details(selected_run) if selected_run else None
    view = (
        explain_run(details, subject)
        if details is not None
        else {
            "schema_version": "tramuri.explain.v1",
            "found": False,
            "subject": (subject or "current product").strip() or "current product",
            "run_id": None,
            "run_status": "ready",
            "mode": None,
            "summary": "No product run is available to explain.",
            "sources": [],
            "connections": [],
            "evidence": [],
            "records": {"decisions": 0, "hypotheses": 0, "knowledge": 0},
            "unknowns": ["No persisted Tramuri evidence is available yet."],
        }
    )
    if _ctx(ctx)["json"]:
        emit(view, True)
    else:
        banner(_ctx(ctx)["no_color"])
        render_explanation(view, _ctx(ctx)["no_color"])


@memory_app.command("show")
def memory_show(
    ctx: typer.Context,
    run_id: str | None = typer.Option(None, "--run-id", help="Run memory to display."),
) -> None:
    """Refresh and display the saved project memory and current work."""
    view = MemoryService(_database(ctx)).show(run_id=run_id)
    if _ctx(ctx)["json"]:
        emit(view, True)
    else:
        typer.echo(Path(view["display_file"]).read_text(encoding="utf-8"))


@memory_app.command("search")
def memory_search(
    ctx: typer.Context,
    query: str = typer.Argument(..., help="Topic to retrieve from saved project memory."),
    run_id: str | None = typer.Option(
        None, "--run-id", help="Limit results to a run and project memory."
    ),
    limit: int = typer.Option(8, min=1, max=50, help="Maximum number of records."),
) -> None:
    """Search persisted knowledge and saved project memories."""
    results = MemoryService(_database(ctx)).search(query, run_id=run_id, limit=limit)
    payload = {"schema_version": "tramuri.memory-search.v1", "query": query, "results": results}
    if _ctx(ctx)["json"]:
        emit(payload, True)
    elif not results:
        typer.echo("No stored memory matches that topic.")
    else:
        for item in results:
            typer.echo(
                f"[{item.get('classification')}] {item.get('title')}\n"
                f"{item.get('content')}\n"
                f"Source: {item.get('source_ref') or item.get('source_kind')}\n"
            )


@memory_app.command("save")
def memory_save(
    ctx: typer.Context,
    title: str = typer.Option(..., help="Short title for a durable memory."),
    content: str = typer.Option(..., help="Concise fact, preference, decision or next action."),
    classification: str = typer.Option(
        ...,
        help="fact, evidence, inference, hypothesis, assumption, decision or pending_validation.",
    ),
    source_ref: str | None = typer.Option(
        None, "--source-ref", help="Where this memory came from, without copying a transcript."
    ),
    run_id: str | None = typer.Option(
        None, "--run-id", help="Attach this memory to a persisted run."
    ),
) -> None:
    """Save a concise, provenance-labelled memory in SQLite and refresh its files."""
    result = MemoryService(_database(ctx)).save(
        title=title,
        content=content,
        classification=classification,
        source_ref=source_ref,
        run_id=run_id,
    )
    emit(result, _ctx(ctx)["json"], _ctx(ctx)["no_color"])


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
    reports = ReportGenerator(_ctx(ctx)["project_dir"]).generate(details)
    reports["browser_validation"] = _record_experience(database, result, reports)
    emit(
        (details | {"experience": reports})
        if _ctx(ctx)["json"]
        else f"Resumed {result} ({details['run']['status']})",
        _ctx(ctx)["json"],
        _ctx(ctx)["no_color"],
    )


@product_app.command("run")
def product_run(
    ctx: typer.Context,
    idea_file: Path = typer.Option(..., "--idea-file", exists=True, dir_okay=False),
    mode: str = typer.Option("mvp", "--mode"),
    provider: str = typer.Option("local-demo", "--provider"),
    research_file: Path | None = typer.Option(
        None,
        "--research-file",
        exists=True,
        dir_okay=False,
        help="Validated provider research JSON.",
    ),
) -> None:
    """Run the product workflow from a local idea file."""
    idea = idea_file.read_text(encoding="utf-8")
    database = _database(ctx)
    run_id = PMRunner(database, _progress(ctx)).run(
        idea, _mode(mode), provider, research_file=research_file
    )
    details = database.run_details(run_id)
    reports = ReportGenerator(_ctx(ctx)["project_dir"]).generate(details)
    reports["browser_validation"] = _record_experience(database, run_id, reports)
    emit(
        {"run_id": run_id, "status": details["run"]["status"], "reports": reports},
        _ctx(ctx)["json"],
        _ctx(ctx)["no_color"],
    )


@product_app.command("research")
def product_research(
    ctx: typer.Context,
    run_id: str = typer.Option(..., "--run-id", help="Existing blocked or demo run."),
    research_file: Path = typer.Option(
        ..., "--research-file", exists=True, dir_okay=False, help="Provider-produced research JSON."
    ),
) -> None:
    """Import source-backed research and resume the dependent product stages."""
    database = _database(ctx)
    import_research_pack(database, run_id, research_file)
    result = PMRunner(database, _progress(ctx)).resume(run_id)
    details = database.run_details(result)
    reports = ReportGenerator(_ctx(ctx)["project_dir"]).generate(details)
    reports["browser_validation"] = _record_experience(database, result, reports)
    emit(
        {"run_id": result, "status": details["run"]["status"], "reports": reports},
        _ctx(ctx)["json"],
        _ctx(ctx)["no_color"],
    )


@product_app.command("demo")
def product_demo(ctx: typer.Context, mode: str = typer.Option("mvp", "--mode")) -> None:
    """Run the deterministic product demo offline."""
    database = _database(ctx)
    run_id = run_demo(database, _mode(mode), _progress(ctx))
    details = database.run_details(run_id)
    reports = ReportGenerator(_ctx(ctx)["project_dir"]).generate(details)
    reports["browser_validation"] = _record_experience(database, run_id, reports)
    payload = {
        "demo": True,
        "synthetic": True,
        "run_id": run_id,
        "status": details["run"]["status"],
        "reports": reports,
        "sensors": check_run(details) + reports.get("experience_sensors", []),
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


@experience_app.command("check")
def experience_check(ctx: typer.Context, run_id: str) -> None:
    """Re-run deterministic and optional browser checks for a product experience."""
    database = _database(ctx)
    details = database.run_details(run_id)
    demo_path = output_root(_ctx(ctx)["project_dir"]) / "demo" / "index.html"
    sensors = check_experience(demo_path.parent, details)
    browser = run_browser_validation(demo_path)
    review = ProductLeadReviewer().review(details, demo_path.parent)
    lifecycle = ExperienceLifecycle(database, run_id)
    review_payload = review.to_dict()
    lifecycle.record("browser_validation", browser.to_dict())
    lifecycle.record("critical_review", review_payload)
    delivery = _record_experience_delivery(
        lifecycle,
        str(details.get("run", {}).get("status", "unknown")),
        sensors,
        review_payload,
        browser,
    )
    payload = {
        "run_id": run_id,
        "sensors": sensors,
        "browser": browser.to_dict(),
        "review": review_payload,
        "delivery": delivery,
        "lifecycle": lifecycle.status(),
    }
    emit(payload, _ctx(ctx)["json"], _ctx(ctx)["no_color"])


@experience_app.command("render")
def experience_render(
    ctx: typer.Context,
    run_id: str,
    category: str | None = typer.Option(
        None,
        "--category",
        help=(
            "Correct the inferred category: mobile_app, web_saas, cli_developer_tool, "
            "desktop_app, api_backend, data_ai_pipeline, iot_hardware, hybrid, or other."
        ),
    ),
) -> None:
    """Regenerate the offline Product Demo, optionally correcting its category."""
    database = _database(ctx)
    details = database.run_details(run_id)
    if category:
        category = category.casefold()
        categories = {item.value for item in ExperienceCategory}
        if category not in categories:
            raise typer.BadParameter(f"Choose one of: {', '.join(sorted(categories))}")
        ExperienceLifecycle(database, run_id).record(
            "classification_override",
            {"category": category, "classification": "decision"},
        )
        details = database.run_details(run_id)
    reports = ReportGenerator(_ctx(ctx)["project_dir"]).generate(details)
    reports["browser_validation"] = _record_experience(database, run_id, reports)
    emit(
        {
            "run_id": run_id,
            "demo_category": reports["demo_category"],
            "demo_template": reports["demo_template"],
            "reports": reports,
        },
        _ctx(ctx)["json"],
        _ctx(ctx)["no_color"],
    )


@hooks_app.command("runtime")
def hooks_runtime(
    provider: str = typer.Option(..., "--provider", help="codex, claude or gemini"),
) -> None:
    """Evaluate one provider hook event from JSON on stdin."""
    hook_runtime_main(provider.lower())


def main() -> None:
    try:
        app()
    except TramuriError as exc:
        typer.echo(f"tramuri: {exc}", err=True)
        raise typer.Exit(code=2) from exc
