"""Deterministic sensors for generated Product Experience artifacts."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from tramuri.experience.models import SensorResult


class DemoQualitySensors:
    required_files = ("index.html", "styles.css", "app.js", "traceability.json")
    required_screens = ("home", "service", "availability", "confirmation")

    def run(self, demo_root: Path, details: dict[str, Any]) -> list[SensorResult]:
        index = demo_root / "index.html"
        html = index.read_text(encoding="utf-8") if index.is_file() else ""
        files = [name for name in self.required_files if (demo_root / name).is_file()]
        results = [
            SensorResult(
                "files",
                "passed" if len(files) == len(self.required_files) else "failed",
                "Required demo files are present"
                if len(files) == len(self.required_files)
                else "Required demo files are missing",
                tuple(files),
                next_step=(
                    "Create all required demo files before delivery."
                    if len(files) != len(self.required_files)
                    else None
                ),
            ),
            self._links(html, demo_root, index),
            self._screens(html, index),
            self._journey(html, demo_root / "app.js"),
            self._traceability(demo_root / "traceability.json", details),
            self._accessibility(html, demo_root / "styles.css", index),
            self._offline(demo_root),
        ]
        return results

    def _links(self, html: str, demo_root: Path, index: Path) -> SensorResult:
        ids = set(re.findall(r'id=["\']([^"\']+)', html))
        missing = [
            target for target in re.findall(r'href=["\']#([^"\']+)', html) if target not in ids
        ]
        missing.extend(
            target
            for target in re.findall(r'href=["\']([^"\']+)', html)
            if not target.startswith(("#", "http://", "https://", "mailto:"))
            and not (demo_root / target).is_file()
        )
        return SensorResult(
            "links",
            "failed" if missing else "passed",
            f"Missing local targets: {', '.join(missing)}"
            if missing
            else "Local links resolve to screens",
            tuple(missing),
            str(index),
            "Repair each missing local target before delivery." if missing else None,
        )

    def _screens(self, html: str, index: Path) -> SensorResult:
        missing = [item for item in self.required_screens if f'id="{item}"' not in html]
        return SensorResult(
            "screens",
            "failed" if missing else "passed",
            f"Missing screens: {', '.join(missing)}"
            if missing
            else "Required journey screens exist",
            tuple(self.required_screens),
            str(index),
            "Restore the missing journey screens before delivery." if missing else None,
        )

    def _journey(self, html: str, script: Path) -> SensorResult:
        text = html + (script.read_text(encoding="utf-8") if script.is_file() else "")
        actions = ("choose-service", "choose-time", "confirm")
        missing = [item for item in actions if item not in text]
        return SensorResult(
            "journey",
            "failed" if missing else "passed",
            f"Missing interactions: {', '.join(missing)}"
            if missing
            else "Primary journey actions are wired",
            actions,
            str(script),
            "Wire the missing primary interaction before delivery." if missing else None,
        )

    def _traceability(self, path: Path, details: dict[str, Any]) -> SensorResult:
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
            valid = str(payload.get("run_id")) == str(details["run"]["id"]) and bool(
                payload.get("source_artifacts")
            )
        except (OSError, json.JSONDecodeError):
            valid = False
            payload = {}
        return SensorResult(
            "traceability",
            "passed" if valid else "failed",
            "Demo maps to source run artifacts" if valid else "Demo traceability is invalid",
            tuple(payload.get("source_artifacts", [])),
            str(path),
            "Regenerate traceability from the source run before delivery." if not valid else None,
        )

    def _accessibility(self, html: str, stylesheet: Path, index: Path) -> SensorResult:
        checks = ("<html lang=", "skip-link", "aria-live", ":focus-visible", "<label for=")
        css = stylesheet.read_text(encoding="utf-8") if stylesheet.is_file() else ""
        missing = [item for item in checks if item not in html and item not in css]
        return SensorResult(
            "accessibility",
            "failed" if missing else "passed",
            f"Missing accessibility signals: {', '.join(missing)}"
            if missing
            else "Basic semantic and keyboard signals are present",
            tuple(checks),
            str(index),
            "Add the missing semantic or keyboard accessibility signal before delivery."
            if missing
            else None,
        )

    def _offline(self, demo_root: Path) -> SensorResult:
        external: list[str] = []
        for path in demo_root.glob("*"):
            if path.suffix not in {".html", ".css", ".js"}:
                continue
            text = path.read_text(encoding="utf-8")
            if re.search(r"https?://", text):
                external.append(path.name)
        return SensorResult(
            "offline",
            "failed" if external else "passed",
            f"External dependencies found: {', '.join(external)}"
            if external
            else "Demo uses only local assets",
            tuple(external),
            str(demo_root),
            "Remove external resources and keep the demo offline." if external else None,
        )
