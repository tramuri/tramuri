from pathlib import Path

import pytest

from tramuri.core.models import Mode
from tramuri.modules.pm.demo import run_demo
from tramuri.reports.generator import ReportGenerator
from tramuri.storage.database import Database

playwright = pytest.importorskip("playwright.sync_api")


def test_product_demo_primary_journey_in_browser(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    paths = ReportGenerator(tmp_path).generate(database.run_details(run_id))

    with playwright.sync_playwright() as api:
        browser = api.chromium.launch(headless=True)
        page = browser.new_page()
        page_errors: list[str] = []
        page.on("pageerror", lambda error: page_errors.append(str(error)))
        page.goto(Path(paths["demo"]).as_uri())
        page.get_by_role("button", name="Choose service").click()
        page.get_by_role("button", name="Choose time").click()
        page.get_by_role("button", name="Confirm demo booking").click()
        assert page.get_by_role("status").inner_text() == "Demo booking confirmed"
        assert page.locator("#confirmation").is_visible()
        assert not page_errors
        page.set_viewport_size({"width": 390, "height": 844})
        assert page.evaluate("document.documentElement.scrollWidth <= window.innerWidth")
        portal = browser.new_page()
        portal.goto(Path(paths["portal"]).as_uri())
        assert portal.get_by_role("heading", name="From intent to verified outcomes.").is_visible()
        assert portal.get_by_role("link", name="Open stage").count() == 2
        portal.get_by_role("button", name="Explore later").first.click()
        assert "ainda não faz parte" in portal.get_by_role("status").inner_text()
        browser.close()
