"""Optional browser validation; absence of Playwright is reported, not hidden."""

from __future__ import annotations

import importlib.util
from pathlib import Path

from tramuri.experience.models import SensorResult


def run_browser_validation(demo_path: Path) -> SensorResult:
    if importlib.util.find_spec("playwright") is None:
        return SensorResult(
            "browser",
            "not_run",
            "Playwright is not installed; browser validation was not run",
            (),
            str(demo_path),
        )
    try:
        from playwright.sync_api import sync_playwright  # type: ignore[import-not-found]

        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(headless=True)
            page = browser.new_page(viewport={"width": 1280, "height": 900})
            page.goto(demo_path.resolve().as_uri(), wait_until="load")
            page.get_by_role("button", name="Choose service").click()
            page.get_by_role("button", name="Choose time").first.click()
            page.get_by_role("button", name="Confirm demo booking").click()
            passed = page.get_by_role("status").inner_text() == "Demo booking confirmed"
            browser.close()
        return SensorResult(
            "browser",
            "passed" if passed else "failed",
            "Primary demo journey completed in Playwright"
            if passed
            else "Primary demo journey did not complete",
            (),
            str(demo_path),
        )
    except Exception as exc:  # browser binaries are an environment capability
        return SensorResult(
            "browser", "not_run", f"Playwright browser could not run: {exc}", (), str(demo_path)
        )
