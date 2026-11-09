import sys
from pathlib import Path
from types import SimpleNamespace

import pytest

import tramuri.experience.browser as browser_validation
from tramuri.core.models import Mode
from tramuri.experience.browser import _exercise
from tramuri.modules.pm.demo import run_demo
from tramuri.modules.pm.runner import PMRunner
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
        page.get_by_role("link", name="Go to availability").click()
        page.get_by_role("button", name="Today · 14:00").click()
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
        assert "ainda não faz parte" in portal.locator("#stage-detail").inner_text()
        browser.close()


def test_learning_product_surfaces_in_browser(tmp_path: Path) -> None:
    database = Database(tmp_path)
    idea = "# Course marketplace\nLearners buy courses and track progress."
    run_id = PMRunner(database).run(idea, Mode.MVP)
    paths = ReportGenerator(tmp_path).generate(database.run_details(run_id))

    with playwright.sync_playwright() as api:
        browser = api.chromium.launch(headless=True)
        page = browser.new_page()
        page.goto(Path(paths["demo"]).as_uri())
        page.get_by_role("button", name="Explore catalog").click()
        page.get_by_role("button", name="View course").click()
        page.get_by_role("button", name="Start course").click()
        assert page.get_by_role("status").inner_text().startswith("Enrollment complete")
        for surface in ("marketing.html", "app.html", "admin.html"):
            assert (Path(paths["demo"]).parent / surface).is_file()
        browser.close()


def test_personal_finance_web_journey_in_browser() -> None:
    demo = """<!doctype html>
<html lang="en"><body>
  <p id="total-spent">$1,808.80</p><p id="expense-count">5</p><ul id="transaction-list"></ul>
  <button data-action="open-expense-form">New expense</button>
  <form id="expense-form" hidden>
    <input id="expense-description"><input id="expense-amount" type="number">
    <select id="expense-category"><option value="">Choose</option><option value="food">Food</option></select>
    <input id="expense-date" type="date">
    <button type="button" data-action="save-expense">Save expense</button>
  </form>
  <button data-action="create-goal">New goal</button>
  <form id="goal-form" hidden>
    <input id="goal-name"><input id="goal-target" type="number"><input id="goal-saved" type="number">
    <button type="submit" data-action="save-goal">Create goal</button>
  </form>
  <section id="goal-list"></section>
  <form id="contribution-form" hidden>
    <input id="contribution-amount" type="number">
    <button type="submit" data-action="save-contribution">Update progress</button>
  </form>
  <div id="confirmation" role="status" aria-live="polite"></div>
  <script>
    const one = (selector) => document.querySelector(selector);
    one('[data-action="open-expense-form"]').addEventListener('click', () => { one('#expense-form').hidden = false; });
    one('[data-action="save-expense"]').addEventListener('click', () => {
      one('#total-spent').textContent = '$1,821.30'; one('#expense-count').textContent = '6';
      one('#transaction-list').textContent = one('#expense-description').value;
      one('#expense-form').hidden = true; one('#confirmation').textContent = 'Expense added.';
    });
    one('[data-action="create-goal"]').addEventListener('click', () => { one('#goal-form').hidden = false; });
    one('[data-action="save-goal"]').addEventListener('click', (event) => {
      event.preventDefault();
      const card = document.createElement('article');
      card.innerHTML = '<span class="goal-name"></span><span class="goal-saved">$200.00</span><button data-action="open-contribution-form">Add value</button>';
      card.querySelector('.goal-name').textContent = one('#goal-name').value;
      one('#goal-list').append(card); one('#goal-form').hidden = true;
      card.querySelector('[data-action="open-contribution-form"]').addEventListener('click', () => { one('#contribution-form').hidden = false; });
    });
    one('[data-action="save-contribution"]').addEventListener('click', (event) => {
      event.preventDefault();
      one('#goal-list .goal-saved').textContent = '$250.00';
      one('#contribution-form').hidden = true; one('#confirmation').textContent = 'Progresso da meta atualizado.';
    });
  </script>
</body></html>"""
    with playwright.sync_playwright() as api:
        browser = api.chromium.launch(headless=True)
        page = browser.new_page()
        page.set_default_timeout(1500)
        page.set_content(demo)
        assert _exercise(page, "web_saas", "personal_finance")
        browser.close()


def test_runtime_browser_flow_errors_are_failures_not_not_run(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    demo_root = tmp_path / "demo"
    demo_root.mkdir()
    index = demo_root / "index.html"
    index.write_text("<html><body>Demo</body></html>", encoding="utf-8")
    (demo_root / "demo_spec.json").write_text(
        '{"category":"web_saas","domain":"project_management"}', encoding="utf-8"
    )

    class FakeLocator:
        @property
        def first(self) -> "FakeLocator":
            return self

        def click(self) -> None:
            raise RuntimeError("simulated missing interaction")

    class FakePage:
        def on(self, *_args: object) -> None:
            return None

        def goto(self, *_args: object, **_kwargs: object) -> None:
            return None

        def screenshot(self, **_kwargs: object) -> None:
            return None

        def set_viewport_size(self, *_args: object) -> None:
            return None

        def evaluate(self, *_args: object) -> bool:
            return True

        def locator(self, *_args: object) -> FakeLocator:
            return FakeLocator()

    class FakeBrowser:
        def new_page(self, **_kwargs: object) -> FakePage:
            return FakePage()

        def close(self) -> None:
            return None

    class FakePlaywrightContext:
        def __enter__(self) -> SimpleNamespace:
            return SimpleNamespace(chromium=SimpleNamespace(launch=lambda **_kwargs: FakeBrowser()))

        def __exit__(self, *_args: object) -> None:
            return None

    monkeypatch.setitem(
        sys.modules,
        "playwright.sync_api",
        SimpleNamespace(sync_playwright=lambda: FakePlaywrightContext()),
    )
    result = browser_validation.run_browser_validation(index)

    assert result.status == "failed"
    assert "simulated missing interaction" in result.message
