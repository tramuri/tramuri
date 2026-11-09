from pathlib import Path

import pytest

from tramuri.core.models import Mode
from tramuri.modules.pm.demo import run_demo
from tramuri.reports.generator import ReportGenerator
from tramuri.storage.database import Database

playwright = pytest.importorskip("playwright.sync_api")


def test_report_language_theme_navigation_responsiveness_and_print(tmp_path: Path) -> None:
    database = Database(tmp_path)
    run_id = run_demo(database, Mode.MVP)
    paths = ReportGenerator(tmp_path).generate(database.run_details(run_id))

    with playwright.sync_playwright() as api:
        browser = api.chromium.launch(headless=True)
        page = browser.new_page(viewport={"width": 1440, "height": 1000})
        page_errors: list[str] = []
        external_requests: list[str] = []
        page.on("pageerror", lambda error: page_errors.append(str(error)))
        page.on(
            "request",
            lambda request: (
                external_requests.append(request.url)
                if request.url.startswith(("http://", "https://"))
                else None
            ),
        )
        page.goto(Path(paths["report"]).as_uri(), wait_until="load")

        assert page.get_by_role("heading", name="Product journey").is_visible()
        assert page.locator(".graph-edge").count() > 0
        assert page.locator(".graph-node").evaluate_all(
            "nodes => nodes.every(node => document.querySelector(node.getAttribute('href')))"
        )
        page.wait_for_function(
            "document.querySelector('.report-nav a[href=\\\"#overview\\\"]')"
            ".getAttribute('aria-current') === 'location'"
        )
        page.keyboard.press("Tab")
        assert page.locator(".skip-link").evaluate("node => document.activeElement === node")
        assert page.locator(".skip-link").evaluate(
            "node => getComputedStyle(node).outlineStyle !== 'none'"
        )
        page.emulate_media(reduced_motion="reduce")
        assert page.locator("html").evaluate(
            "node => getComputedStyle(node).scrollBehavior === 'auto'"
        )
        page.emulate_media(reduced_motion="no-preference")

        page.get_by_role("button", name="Portugues (Brasil)").click()
        assert page.locator("html").get_attribute("lang") == "pt-BR"
        assert page.get_by_role("link", name="Visão geral").is_visible()

        page.get_by_role("button", name="Alternar para tema escuro").click()
        assert page.locator("html").get_attribute("data-theme") == "dark"
        page.wait_for_timeout(200)
        assert page.locator(".report-main").evaluate("node => getComputedStyle(node).color")
        assert (
            page.locator(".demo-link").first.evaluate(
                "node => getComputedStyle(node).backgroundColor"
            )
            != "rgb(255, 255, 255)"
        )

        page.get_by_role("link", name="Jornada do ciclo de produto").click()
        assert page.evaluate("location.hash") == "#journey"
        page.wait_for_function(
            "document.querySelector('.report-nav a[href=\\\"#journey\\\"]')"
            ".getAttribute('aria-current') === 'location'"
        )
        assert (
            page.locator('.report-nav a[href="#journey"]').get_attribute("aria-current")
            == "location"
        )

        for width, height in ((1440, 1000), (768, 1024), (390, 844)):
            page.set_viewport_size({"width": width, "height": height})
            assert page.evaluate("document.documentElement.scrollWidth <= window.innerWidth")

        assert page.locator(".graph-scroll-hint").is_visible()
        graph = page.locator(".graph-scroll")
        assert graph.evaluate("node => node.scrollWidth > node.clientWidth")
        graph.evaluate("node => { node.scrollLeft = 200; }")
        assert graph.evaluate("node => node.scrollLeft > 0")

        page.evaluate(
            "window.__tramuriPrintCalled = false; window.print = () => { window.__tramuriPrintCalled = true; }"
        )
        page.get_by_role("button", name="Imprimir ou salvar como PDF").click()
        assert page.evaluate("window.__tramuriPrintCalled")

        page.emulate_media(media="print")
        assert page.locator(".sidebar").evaluate("node => getComputedStyle(node).display") == "none"
        page.emulate_media(media="screen")
        assert not page_errors
        assert not external_requests
        browser.close()
