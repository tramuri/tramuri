from pathlib import Path

from tramuri.core.models import Mode
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
