from pathlib import Path

from tramuri.core.models import Mode
from tramuri.modules.pm.demo import run_demo
from tramuri.reports.generator import ReportGenerator
from tramuri.storage.database import Database


def test_demo_completes_and_generates_offline_artifacts(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    details = database.run_details(run_id)
    assert details["run"]["status"] == "completed"
    assert len(details["artifacts"]) == 10
    assert details["hypotheses"]
    paths = ReportGenerator(tmp_path).generate(details)
    ReportGenerator(tmp_path).generate(details)
    report = Path(paths["report"]).read_text(encoding="utf-8")
    prototype = Path(paths["prototype"]).read_text(encoding="utf-8")
    assert "Product report" in report
    assert "#service" in prototype
    assert "https://" not in report
    assert "https://" not in prototype
    assert Path(paths["report"]).with_name("report-v1.html").is_file()


def test_modes_are_executable(tmp_path: Path) -> None:
    database = Database(tmp_path)
    for mode in Mode:
        run_id = run_demo(database, mode)
        assert database.run_details(run_id)["run"]["status"] == "completed"
