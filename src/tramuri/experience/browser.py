"""Optional Playwright interaction, responsive-layout and screenshot validation."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
from typing import Any

from tramuri.experience.models import SensorResult


def run_browser_validation(demo_path: Path) -> SensorResult:
    """Exercise one category-specific journey and capture desktop/mobile screenshots."""
    if importlib.util.find_spec("playwright") is None:
        return SensorResult(
            "browser",
            "not_run",
            "Playwright is not installed; browser validation and screenshots were not run",
            (),
            str(demo_path),
        )
    try:
        from playwright.sync_api import sync_playwright  # type: ignore[import-not-found]
    except Exception as exc:
        return SensorResult(
            "browser",
            "not_run",
            f"Playwright runtime is unavailable: {exc}",
            (),
            str(demo_path),
        )
    try:
        spec = json.loads((demo_path.parent / "demo_spec.json").read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return SensorResult(
            "browser",
            "failed",
            f"Browser validation could not read the demo specification: {exc}",
            (),
            str(demo_path),
        )

    category = str(spec.get("category", "other"))
    domain = str(spec.get("domain", ""))
    screenshot_dir = demo_path.parent / "screenshots"
    screenshot_dir.mkdir(parents=True, exist_ok=True)
    evidence: list[str] = []
    page_errors: list[str] = []
    console_errors: list[str] = []
    try:
        with sync_playwright() as api:
            try:
                browser = api.chromium.launch(headless=True)
            except Exception:
                try:
                    browser = api.chromium.launch(headless=True, channel="chrome")
                except Exception as exc:
                    return SensorResult(
                        "browser",
                        "not_run",
                        f"Playwright browser is unavailable: {exc}",
                        (),
                        str(demo_path),
                    )
            try:
                page = browser.new_page(viewport={"width": 1440, "height": 1000})
                page.on("pageerror", lambda error: page_errors.append(str(error)))
                page.on(
                    "console",
                    lambda message: (
                        console_errors.append(message.text) if message.type == "error" else None
                    ),
                )
                page.goto(demo_path.resolve().as_uri(), wait_until="load")
                desktop_capture = screenshot_dir / "demo-desktop.png"
                page.screenshot(path=str(desktop_capture), full_page=True)
                evidence.append(str(desktop_capture))
                page.set_viewport_size({"width": 390, "height": 844})
                mobile_capture = screenshot_dir / "demo-mobile.png"
                page.screenshot(path=str(mobile_capture), full_page=True)
                evidence.append(str(mobile_capture))
                responsive = bool(
                    page.evaluate("document.documentElement.scrollWidth <= window.innerWidth")
                )
                journey = _exercise(page, category, domain)
                passed = responsive and journey and not page_errors and not console_errors
                evidence.extend(f"page_error: {error}" for error in page_errors)
                evidence.extend(f"console_error: {error}" for error in console_errors)
                return SensorResult(
                    "browser",
                    "passed" if passed else "failed",
                    "Category-specific flow and mobile layout passed; desktop and mobile screenshots captured"
                    if passed
                    else "Browser flow, responsive layout, or JavaScript error check failed",
                    tuple(evidence),
                    str(demo_path),
                    None
                    if passed
                    else "Inspect browser errors and responsive layout at the saved screenshots.",
                )
            except Exception as exc:
                evidence.append(f"browser_error: {exc}")
                return SensorResult(
                    "browser",
                    "failed",
                    f"Browser validation failed during the product flow: {exc}",
                    tuple(evidence),
                    str(demo_path),
                    "Inspect the failing interaction and saved screenshots before delivery.",
                )
            finally:
                browser.close()
    except Exception as exc:  # Playwright can fail before a browser session is available
        return SensorResult(
            "browser",
            "not_run",
            f"Playwright runtime could not start: {exc}",
            (),
            str(demo_path),
        )


def _exercise(page: Any, category: str, domain: str) -> bool:
    """Use only predefined browser actions; never submit input to an OS shell."""
    if category == "mobile_app":
        action = "add-expense" if domain == "personal_finance" else "complete-mobile-step"
        expected = "Sample expense added" if domain == "personal_finance" else "Next step completed"
        page.locator(f'[data-action="{action}"]').click()
        action_passed = expected in page.get_by_role("status").inner_text()
        page.get_by_role("tab", name="Activity").click()
        activity_visible = page.locator('[data-mobile-screen="activity"]').is_visible()
        if domain == "personal_finance":
            activity_visible = activity_visible and page.get_by_text("Local coffee").is_visible()
        page.get_by_role("tab", name="Overview").click()
        overview_visible = page.locator('[data-mobile-screen="overview"]').is_visible()
        return action_passed and activity_visible and overview_visible
    if category == "web_saas" and domain == "personal_finance":
        return _exercise_personal_finance_web(page)
    if category == "web_saas" and domain == "service_booking":
        page.get_by_role("button", name="Choose service").click()
        page.get_by_role("link", name="Go to availability").click()
        page.get_by_role("button", name="Today · 14:00").click()
        page.get_by_role("button", name="Confirm demo booking").click()
        return str(page.get_by_role("status").inner_text()) == "Demo booking confirmed"
    if category == "web_saas" and domain == "education":
        page.get_by_role("button", name="Explore catalog").click()
        page.get_by_role("button", name="View course").click()
        page.get_by_role("button", name="Start course").click()
        return str(page.get_by_role("status").inner_text()).startswith("Enrollment complete")
    if category == "web_saas" and domain == "marketplace":
        page.get_by_role("button", name="Shop by collection").click()
        page.get_by_role("button", name="View details").first.click()
        page.get_by_role("button", name="Add to basket").click()
        page.get_by_role("button", name="Complete sample checkout").click()
        return str(page.get_by_role("status").inner_text()).startswith("Sample checkout complete")
    action_by_category = {
        "cli_developer_tool": ("run-scenario", "Predefined scenario completed"),
        "web_saas": ("move-task", "Task moved to In progress"),
        "desktop_app": ("save-settings", "Settings saved"),
        "api_backend": ("send-request", "Request validated locally"),
        "data_ai_pipeline": ("run-pipeline", "Pipeline simulation complete"),
        "iot_hardware": ("change-reading", "Simulated sensor reading updated"),
        "hybrid": ("capture-reading", "Mobile record and operations dashboard updated"),
        "other": ("advance-flow", "Inferred outcome shown"),
    }
    action, expected = action_by_category.get(category, action_by_category["other"])
    page.locator(f'[data-action="{action}"]').first.click()
    return expected in page.get_by_role("status").inner_text()


def _exercise_personal_finance_web(page: Any) -> bool:
    """Exercise local expense and savings-goal updates in a finance web demo."""
    if not page.locator('[data-action="open-expense-form"]').count():
        return False

    total_before = page.locator("#total-spent").inner_text()
    count_before = int(page.locator("#expense-count").inner_text())
    page.locator('[data-action="open-expense-form"]').first.click()
    page.locator("#expense-description").fill("Browser validation expense")
    page.locator("#expense-amount").fill("12.50")
    category = page.locator("#expense-category")
    category_index = category.locator("option").evaluate_all(
        "(options) => options.findIndex((option) => option.value.trim() !== '')"
    )
    if category_index < 0:
        return False
    category.select_option(index=category_index)
    page.locator("#expense-date").fill("2026-11-08")
    page.locator('[data-action="save-expense"]').click()

    expense_passed = (
        int(page.locator("#expense-count").inner_text()) == count_before + 1
        and page.locator("#total-spent").inner_text() != total_before
        and "Browser validation expense" in page.locator("#transaction-list").inner_text()
    )
    if not expense_passed:
        return False

    goal_name = "Browser validation goal"
    page.locator('[data-action="create-goal"]').first.click()
    page.locator("#goal-name").fill(goal_name)
    page.locator("#goal-target").fill("1000")
    page.locator("#goal-saved").fill("200")
    page.locator('[data-action="save-goal"]').click()
    goal_card = page.locator("#goal-list article").filter(has_text=goal_name)
    if not goal_card.count():
        return False

    goal_card.locator('[data-action="open-contribution-form"]').click()
    page.locator("#contribution-amount").fill("50")
    page.locator('[data-action="save-contribution"]').click()
    return (
        "250" in goal_card.inner_text()
        and "Progresso da meta atualizado." in page.get_by_role("status").inner_text()
    )
