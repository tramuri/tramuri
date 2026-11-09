from pathlib import Path

from tramuri.core.models import Mode
from tramuri.modules.pm.demo import run_demo
from tramuri.reports.generator import ReportGenerator
from tramuri.storage.database import Database


def test_report_has_bilingual_offline_product_intelligence_layout(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    report_path = Path(ReportGenerator(tmp_path).generate(database.run_details(run_id))["report"])
    report = report_path.read_text(encoding="utf-8")

    for marker in (
        'id="overview"',
        'id="journey"',
        'id="intelligence"',
        'id="decision-evidence"',
        'id="execution"',
        "data-theme-toggle",
        'data-language="pt-BR"',
        'data-i18n="nav.overview"',
        "prefers-reduced-motion",
        "@media print",
        "Product report",
        "No market evidence",
    ):
        assert marker in report

    assert "fonts.googleapis.com" not in report
    assert "cdn.jsdelivr.net" not in report


def test_checkpoint_fallback_does_not_render_internal_payload(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    details = database.run_details(run_id)
    details["checkpoints"] = [
        {
            "id": "checkpoint-1",
            "stage_key": "product_framing",
            "status": "paused",
            "created_at": "2026-11-08T12:00:00Z",
            "pending": {"internal_marker": "CHECKPOINT_INTERNAL_PAYLOAD"},
        }
    ]

    report_path = Path(ReportGenerator(tmp_path).generate(details)["report"])
    report = report_path.read_text(encoding="utf-8")

    assert "product_framing" in report
    assert "No next action recorded." in report
    assert "CHECKPOINT_INTERNAL_PAYLOAD" not in report
