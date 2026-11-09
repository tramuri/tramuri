from pathlib import Path

from tramuri.core.models import Mode
from tramuri.modules.pm.research import import_research_pack
from tramuri.modules.pm.runner import PMRunner
from tramuri.reports.generator import ReportGenerator
from tramuri.storage.database import Database


def test_report_escapes_untrusted_idea(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = PMRunner(database).run("<script>alert(1)</script>", Mode.POC)
    paths = ReportGenerator(tmp_path).generate(database.run_details(run_id))
    content = Path(paths["report"]).read_text(encoding="utf-8")
    assert "<script>alert(1)</script>" not in content
    assert "&lt;script&gt;" in content


def test_report_escapes_untrusted_research_source_metadata(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = PMRunner(database).run("A course marketplace", Mode.MVP, provider="codex")
    research = tmp_path / "research.json"
    research.write_text(
        """{
          "version": "tramuri.research.v1",
          "researched_at": "2026-11-08T12:00:00Z",
          "query": "course market",
          "sources": [{"id":"s1","url":"https://example.com","title":"<script>alert(1)</script>","publisher":"Research","accessed_at":"2026-11-08","confidence":"low"}],
          "competitors": [{"name":"<img src=x>","source_ids":["s1"]}],
          "market_signals": [], "buying_motivations": [], "build_buy_partner": [],
          "positioning": [], "market_sizing": {"status":"pending_validation","claims":[]}
        }""",
        encoding="utf-8",
    )
    import_research_pack(database, run_id, research)
    PMRunner(database).resume(run_id)
    content = Path(
        ReportGenerator(tmp_path).generate(database.run_details(run_id))["report"]
    ).read_text(encoding="utf-8")
    assert "<script>alert(1)</script>" not in content
    assert "&lt;script&gt;" in content


def test_report_redacts_secrets_in_text_and_structured_sensor_evidence(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = PMRunner(database).run("Build a service with api_key=report-secret-value", Mode.POC)
    details = database.run_details(run_id)
    details["sensor_results"] = [
        {
            "sensor_key": "safe-sensor",
            "status": "passed",
            "severity": "advisory",
            "stage_key": "framing",
            "evidence": {
                "authorization": "Bearer private-session-token",
                "note": "token=another-private-value",
            },
        }
    ]

    paths = ReportGenerator(tmp_path).generate(details)
    outputs = [paths[key] for key in ("report", "prototype", "portal", "demo")]
    rendered = [Path(path).read_text(encoding="utf-8") for path in outputs]

    assert all("report-secret-value" not in content for content in rendered)
    assert all("private-session-token" not in content for content in rendered)
    assert all("another-private-value" not in content for content in rendered)
    assert "[REDACTED]" in rendered[0]
