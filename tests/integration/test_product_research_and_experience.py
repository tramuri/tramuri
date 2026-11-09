import json
from pathlib import Path

from tramuri.core.models import Mode
from tramuri.core.status import build_status
from tramuri.experience.reviewer import ProductLeadReviewer
from tramuri.experience.sensors import DemoQualitySensors
from tramuri.modules.pm.research import import_research_pack
from tramuri.modules.pm.runner import PMRunner
from tramuri.reports.generator import ReportGenerator
from tramuri.storage.database import Database


def _course_idea() -> str:
    return """# SaaS para vender cursos online

## Intenção
Quero criar uma plataforma para especialistas venderem cursos de IA.

## Público
Alunos e criadores independentes.

## Resultado desejado
O aluno compra uma trilha, assiste às aulas e acompanha seu progresso.
"""


def test_course_generates_complete_product_experience(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = PMRunner(database).run(_course_idea(), Mode.MVP)
    details = database.run_details(run_id)
    paths = ReportGenerator(tmp_path).generate(details)
    demo_root = Path(paths["demo"]).parent
    report_html = Path(paths["report"]).read_text(encoding="utf-8")

    assert (demo_root / "marketing.html").is_file()
    assert (demo_root / "app.html").is_file()
    assert (demo_root / "admin.html").is_file()
    assert "curso" in (demo_root / "index.html").read_text(encoding="utf-8").lower()
    assert "catalog" in (demo_root / "app.html").read_text(encoding="utf-8").lower()
    assert "admin" in (demo_root / "admin.html").read_text(encoding="utf-8").lower()

    traceability = json.loads((demo_root / "traceability.json").read_text(encoding="utf-8"))
    assert {"marketing", "customer", "admin"}.issubset(traceability["surfaces"])
    assert len(traceability["screens"]) >= 5
    assert len(traceability["interactions"]) >= 3
    assert ProductLeadReviewer().review(details, demo_root).status == "passed"
    assert all(item.status == "passed" for item in DemoQualitySensors().run(demo_root, details))
    assert "English" in report_html
    assert "Portugues (Brasil)" in report_html
    assert "Market Research" in report_html
    assert "Buying motivations" in report_html
    assert "Research not run" in report_html
    assert build_status(details)["research"]["status"] == "not_run"
    assert build_status(details)["agents"]["delegated"] >= 1


def test_local_demo_declares_research_as_not_run(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = PMRunner(database).run(_course_idea(), Mode.MVP)
    details = database.run_details(run_id)
    research_stage = next(item for item in details["stages"] if item["stage_key"] == "research")

    assert research_stage["status"] == "skipped"
    research_events = [
        item for item in details["events"] if item["event_type"] == "research.not_run"
    ]
    assert research_events
    assert research_events[-1]["payload"]["reason"] == "local-demo"


def test_provider_run_waits_for_research_pack_then_resumes(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = PMRunner(database).run(_course_idea(), Mode.MVP, provider="codex")
    assert database.run_details(run_id)["run"]["status"] == "blocked"

    pack = {
        "version": "tramuri.research.v1",
        "researched_at": "2026-11-08T12:00:00Z",
        "query": "online learning platforms",
        "sources": [
            {
                "id": "s1",
                "url": "https://example.com/learning",
                "title": "Learning signals",
                "publisher": "Example Research",
                "accessed_at": "2026-11-08",
                "confidence": "medium",
            }
        ],
        "competitors": [{"name": "Alternative", "gaps": ["progress"], "source_ids": ["s1"]}],
        "market_signals": [],
        "buying_motivations": [],
        "build_buy_partner": [],
        "positioning": [],
        "market_sizing": {"status": "pending_validation", "claims": []},
    }
    research_file = tmp_path / "research.json"
    research_file.write_text(json.dumps(pack), encoding="utf-8")

    import_research_pack(database, run_id, research_file)
    PMRunner(database).resume(run_id)
    details = database.run_details(run_id)
    assert details["run"]["status"] == "completed"
    assert any(item["kind"] == "pm.market_research" for item in details["artifacts"])
    assert any(item["source_kind"] == "market_research" for item in details["evidence"])
    report = Path(ReportGenerator(tmp_path).generate(details)["report"]).read_text(encoding="utf-8")
    assert "Learning signals" in report
    assert "Alternative" in report
    assert "https://example.com/learning" in report
    assert database.invalidate_from_stage(run_id, "research") > 0
