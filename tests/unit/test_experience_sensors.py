from pathlib import Path

from tramuri.core.models import Mode
from tramuri.experience.sensors import DemoQualitySensors
from tramuri.modules.pm.demo import run_demo
from tramuri.reports.generator import ReportGenerator
from tramuri.storage.database import Database


def test_sensors_find_missing_screen_and_external_asset(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    details = database.run_details(run_id)
    demo = Path(ReportGenerator(tmp_path).generate(details)["demo"])
    html = demo.read_text(encoding="utf-8")
    demo.write_text(
        html.replace('href="#availability"', 'href="#missing"', 1)
        .replace('id="confirmation"', 'id="removed"', 1)
        .replace('href="styles.css"', 'href="https://invalid.example/style.css"', 1),
        encoding="utf-8",
    )

    results = {item.key: item for item in DemoQualitySensors().run(demo.parent, details)}

    assert results["links"].status == "failed"
    assert "missing" in results["links"].message
    assert results["screens"].status == "failed"
    assert results["offline"].status == "failed"
