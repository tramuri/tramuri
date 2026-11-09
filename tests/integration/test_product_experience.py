import json
from pathlib import Path

from typer.testing import CliRunner

from tramuri.cli.app import app
from tramuri.core.models import Mode
from tramuri.experience.lifecycle import ExperienceLifecycle
from tramuri.experience.models import SensorResult
from tramuri.experience.reviewer import ProductLeadReviewer
from tramuri.experience.sensors import DemoQualitySensors
from tramuri.modules.pm.demo import run_demo
from tramuri.reports.generator import ReportGenerator
from tramuri.storage.database import Database


def test_report_generation_creates_portal_and_demo_journey(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    details = database.run_details(run_id)

    paths = ReportGenerator(tmp_path).generate(details)

    portal = Path(paths["portal"])
    demo = Path(paths["demo"])
    assert portal == tmp_path / "tramuri_output" / "portal" / "index.html"
    assert demo == tmp_path / "tramuri_output" / "demo" / "index.html"
    assert portal.is_file() and demo.is_file()
    portal_html = portal.read_text(encoding="utf-8")
    assert "Product Definition" in portal_html
    assert "Product Demo" in portal_html
    assert "Product UX" in portal_html and "Product Engineering" in portal_html
    assert "../reports/" in portal_html
    assert "../demo/index.html" in portal_html
    demo_html = demo.read_text(encoding="utf-8")
    assert 'id="home"' in demo_html
    assert 'id="availability"' in demo_html
    assert 'id="confirmation"' in demo_html
    assert 'data-action="confirm"' in demo_html
    assert "https://" not in demo_html

    traceability = json.loads(
        (tmp_path / "tramuri_output" / "demo" / "traceability.json").read_text(encoding="utf-8")
    )
    assert traceability["run_id"] == run_id
    assert traceability["source_artifacts"]


def test_demo_sensors_and_reviewer_report_specific_evidence(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    details = database.run_details(run_id)
    paths = ReportGenerator(tmp_path).generate(details)
    demo_root = Path(paths["demo"]).parent

    review = ProductLeadReviewer().review(details, demo_root)
    results = DemoQualitySensors().run(demo_root, details)

    assert review.status == "passed"
    assert all(result.status == "passed" for result in results)

    demo_html = demo_root.joinpath("index.html")
    demo_html.write_text(
        demo_html.read_text(encoding="utf-8").replace('href="#availability"', 'href="#missing"', 1),
        encoding="utf-8",
    )
    broken = DemoQualitySensors().run(demo_root, details)
    link_result = next(result for result in broken if result.key == "links")
    assert link_result.status == "failed"
    assert "missing" in link_result.message


def test_experience_lifecycle_events_are_idempotent(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    lifecycle = ExperienceLifecycle(database, run_id)

    lifecycle.record("product_review", {"status": "passed"})
    lifecycle.record("product_review", {"status": "passed"})
    details = database.run_details(run_id)

    events = [
        item for item in details["events"] if item["event_type"] == "experience.product_review"
    ]
    assert len(events) == 1


def test_experience_lifecycle_records_changed_retry_without_duplication(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    lifecycle = ExperienceLifecycle(database, run_id)

    lifecycle.record("browser_validation", {"status": "not_run"})
    lifecycle.record("browser_validation", {"status": "passed"})
    lifecycle.record("browser_validation", {"status": "passed"})

    details = database.run_details(run_id)
    events = [
        item for item in details["events"] if item["event_type"] == "experience.browser_validation"
    ]
    assert len(events) == 2
    assert lifecycle.status()["browser_validation"]["payload"]["status"] == "passed"


def test_experience_check_records_a_validated_delivery(monkeypatch, tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    ReportGenerator(tmp_path).generate(database.run_details(run_id))
    monkeypatch.setattr(
        "tramuri.cli.app.run_browser_validation",
        lambda demo_path: SensorResult(
            "browser",
            "passed",
            "Finance and responsive browser flow passed.",
            (str(demo_path),),
        ),
    )

    result = CliRunner().invoke(
        app,
        [
            "--project-dir",
            str(tmp_path),
            "--json",
            "experience",
            "check",
            run_id,
        ],
    )

    assert result.exit_code == 0, result.output
    payload = json.loads(result.stdout)
    delivery = payload["lifecycle"]["delivery"]["payload"]
    assert delivery["status"] == "completed"
    assert delivery["browser"]["status"] == "passed"
    assert delivery["review"]["status"] == "passed"
    assert all(item["status"] == "passed" for item in delivery["sensors"])


def test_experience_lifecycle_allows_a_previous_state_to_become_current_again(
    tmp_path: Path,
) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    lifecycle = ExperienceLifecycle(database, run_id)
    passed = {"status": "passed", "findings": []}
    warned = {"status": "passed", "findings": ["stale scope warning"]}

    lifecycle.record("critical_review", passed)
    lifecycle.record("critical_review", warned)
    lifecycle.record("critical_review", passed)

    details = database.run_details(run_id)
    reviews = [
        event for event in details["events"] if event["event_type"] == "experience.critical_review"
    ]
    assert len(reviews) == 3
    assert lifecycle.status()["critical_review"]["payload"] == passed
