"""Deterministic offline, accessibility and visual-quality checks for demos."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from tramuri.experience.models import SensorResult


class VisualQualityValidator:
    """Validate rendered demo structure and report measurable visual signals."""

    required_files = ("index.html", "styles.css", "app.js", "demo_spec.json", "traceability.json")

    def run(self, demo_root: Path, details: dict[str, Any]) -> list[SensorResult]:
        index = demo_root / "index.html"
        html = index.read_text(encoding="utf-8") if index.is_file() else ""
        spec = self._read_json(demo_root / "demo_spec.json")
        traceability = self._read_json(demo_root / "traceability.json")
        all_html = " ".join(
            path.read_text(encoding="utf-8") for path in demo_root.glob("*.html") if path.is_file()
        )
        css_path = demo_root / "styles.css"
        css = css_path.read_text(encoding="utf-8") if css_path.is_file() else ""
        script_path = demo_root / "app.js"
        script = script_path.read_text(encoding="utf-8") if script_path.is_file() else ""
        results = [
            self._files(demo_root),
            self._links(all_html, demo_root, index),
            self._screens(all_html, spec, index),
            self._interactions(all_html, script, spec, script_path),
            self._traceability(traceability, details, demo_root / "traceability.json"),
            self._accessibility(html, css, index),
            self._responsive(html, css, index),
            self._contrast(css, css_path),
            self._offline(demo_root),
            self._representation(html, spec, index),
            self._placeholders(demo_root),
        ]
        return results

    def _files(self, root: Path) -> SensorResult:
        found = [name for name in self.required_files if (root / name).is_file()]
        return SensorResult(
            "files",
            "passed" if len(found) == len(self.required_files) else "failed",
            "Demo files and versioned specification are present"
            if len(found) == len(self.required_files)
            else "Required demo files are missing",
            tuple(found),
            str(root),
            None if len(found) == len(self.required_files) else "Create the missing demo files.",
        )

    def _links(self, html: str, root: Path, index: Path) -> SensorResult:
        ids = set(re.findall(r'id=["\']([^"\']+)', html))
        missing = [
            target for target in re.findall(r'href=["\']#([^"\']+)', html) if target not in ids
        ]
        missing.extend(
            target
            for target in re.findall(r'href=["\']([^"\']+)', html)
            if not target.startswith(("#", "http://", "https://", "mailto:"))
            and not (root / target.split("#", 1)[0]).is_file()
        )
        return SensorResult(
            "links",
            "failed" if missing else "passed",
            f"Missing local targets: {', '.join(missing)}"
            if missing
            else "Local navigation targets resolve",
            tuple(missing),
            str(index),
            "Repair the local navigation target before delivery." if missing else None,
        )

    def _screens(self, html: str, spec: dict[str, Any], index: Path) -> SensorResult:
        screens = spec.get("screens", [])
        required = [str(item.get("id", "")) for item in screens if isinstance(item, dict)]
        missing = [identifier for identifier in required if f'id="{identifier}"' not in html]
        return SensorResult(
            "screens",
            "failed" if missing else "passed",
            f"Missing screens: {', '.join(missing)}"
            if missing
            else "Specified journey screens exist",
            tuple(required),
            str(index),
            "Render each screen recorded in the demo specification." if missing else None,
        )

    def _interactions(
        self, html: str, script: str, spec: dict[str, Any], script_path: Path
    ) -> SensorResult:
        interactions = spec.get("interactions", [])
        missing: list[str] = []
        submitted_forms = set(
            re.findall(
                r"document\.querySelector\(['\"]#([^'\"]+)['\"]\)\?\.addEventListener\(['\"]submit",
                script,
            )
        )
        form_actions: dict[str, str] = {}
        for match in re.finditer(r"<form\b([^>]*)>(.*?)</form>", html, re.IGNORECASE | re.DOTALL):
            form_id = re.search(r"\bid=['\"]([^'\"]+)", match.group(1), re.IGNORECASE)
            action_ids = re.findall(r"data-action=['\"]([^'\"]+)", match.group(2), re.IGNORECASE)
            if form_id:
                form_actions.update({action: form_id.group(1) for action in action_ids})
        for item in interactions:
            if not isinstance(item, dict):
                missing.append("invalid interaction record")
                continue
            action = str(item.get("id", ""))
            handler_is_bound = action in script or form_actions.get(action) in submitted_forms
            if f'data-action="{action}"' not in html or not handler_is_bound:
                missing.append(action)
        return SensorResult(
            "interactions",
            "failed" if missing else "passed",
            f"Unwired product actions: {', '.join(missing)}"
            if missing
            else "Product interactions have local state handlers",
            tuple(str(item.get("id", "")) for item in interactions if isinstance(item, dict)),
            str(script_path),
            "Wire every specified action to an explicit local simulation." if missing else None,
        )

    def _traceability(
        self, payload: dict[str, Any], details: dict[str, Any], path: Path
    ) -> SensorResult:
        source_artifacts = payload.get("source_artifacts", [])
        valid = (
            str(payload.get("run_id")) == str(details.get("run", {}).get("id"))
            and payload.get("schema_version") == "tramuri.demo-traceability.v1"
            and bool(source_artifacts)
            and bool(payload.get("category"))
            and all(
                payload.get("screen_sources", {}).get(screen)
                for screen in payload.get("screens", [])
            )
            and all(
                payload.get("interaction_sources", {}).get(action)
                for action in payload.get("interactions", [])
            )
        )
        return SensorResult(
            "traceability",
            "passed" if valid else "failed",
            "Demo spec and rendered screens map to persisted source artifacts"
            if valid
            else "Demo traceability is invalid",
            tuple(str(item) for item in source_artifacts),
            str(path),
            "Regenerate the demo from the persisted PDLC context." if not valid else None,
        )

    def _accessibility(self, html: str, css: str, index: Path) -> SensorResult:
        required = ("<html lang=", "skip-link", "aria-live", ":focus-visible", "min-height:44px")
        missing = [item for item in required if item not in html and item not in css]
        if re.search(r"<(input|select|textarea)\b", html) and "<label" not in html:
            missing.append("label for form controls")
        return SensorResult(
            "accessibility",
            "failed" if missing else "passed",
            f"Missing accessibility signals: {', '.join(missing)}"
            if missing
            else "Semantic HTML, keyboard feedback, live status and touch targets are present",
            tuple(required),
            str(index),
            "Add the missing semantic, focus or form-label signal." if missing else None,
        )

    def _responsive(self, html: str, css: str, index: Path) -> SensorResult:
        checks = ('name="viewport"', "@media", "overflow-wrap:anywhere")
        missing = [item for item in checks if item not in html and item not in css]
        return SensorResult(
            "responsive",
            "failed" if missing else "passed",
            f"Missing responsive signals: {', '.join(missing)}"
            if missing
            else "Viewport, breakpoint and long-content handling are defined",
            tuple(checks),
            str(index),
            "Add viewport-aware layout and overflow handling." if missing else None,
        )

    def _contrast(self, css: str, path: Path) -> SensorResult:
        colors = dict(re.findall(r"--(bg|surface|ink|muted|accent|soft):\s*(#[0-9a-fA-F]{6})", css))
        button_text = "#101619" if ".terminal-window button{color:#101619}" in css else "#ffffff"
        ratios = {
            name: self._contrast_ratio(colors[foreground], colors["bg"])
            for name, foreground in (
                ("body text", "ink"),
                ("secondary text", "muted"),
                ("accent labels", "accent"),
            )
            if foreground in colors and "bg" in colors
        }
        if "accent" in colors:
            ratios["primary button text"] = self._contrast_ratio(button_text, colors["accent"])
        if "ink" in colors and "soft" in colors:
            ratios["quiet button text"] = self._contrast_ratio(colors["ink"], colors["soft"])
        failures = [f"{name} {ratio:.2f}:1" for name, ratio in ratios.items() if ratio < 4.5]
        if len(ratios) != 5:
            failures.append("missing semantic background, text, accent or button tokens")
        return SensorResult(
            "contrast",
            "failed" if failures else "passed",
            f"Contrast below 4.5:1: {', '.join(failures)}"
            if failures
            else "Text, accent labels and primary/quiet controls meet the 4.5:1 target",
            tuple(f"{name}: {ratio:.2f}:1" for name, ratio in ratios.items()),
            str(path),
            "Adjust semantic palette tokens and rerun contrast checks." if failures else None,
        )

    def _offline(self, root: Path) -> SensorResult:
        external: list[str] = []
        for path in root.glob("*"):
            if path.suffix not in {".html", ".css", ".js"}:
                continue
            text = path.read_text(encoding="utf-8")
            if re.search(r"https?://", text):
                external.append(path.name)
        return SensorResult(
            "offline",
            "failed" if external else "passed",
            f"External resources found: {', '.join(external)}"
            if external
            else "Demo runs from local HTML, CSS and JavaScript",
            tuple(external),
            str(root),
            "Remove external resources from the offline demo." if external else None,
        )

    def _representation(self, html: str, spec: dict[str, Any], index: Path) -> SensorResult:
        template_id = str(spec.get("template_id", ""))
        representation = {
            "mobile": "device-shell",
            "cli": "terminal-window",
            "web": "surface-nav",
            "desktop": "window-frame",
            "api": "api-layout",
            "pipeline": "pipeline",
            "iot": "device-panel",
            "hybrid": "hybrid-surfaces",
            "other": "concept-flow",
        }.get(template_id, "")
        valid = (
            bool(representation)
            and representation in html
            and f'data-demo-category="{spec.get("category")}"' in html
            and str(spec.get("simulation_level", "")) in html
            and "simulation" in html.casefold()
        )
        return SensorResult(
            "representation",
            "passed" if valid else "failed",
            "Category-specific representation and simulation boundary are visible"
            if valid
            else "Representation does not match the classified product category",
            (template_id, representation),
            str(index),
            "Render the category's registered visual environment and label simulated states."
            if not valid
            else None,
        )

    def _placeholders(self, root: Path) -> SensorResult:
        unresolved: list[str] = []
        for path in root.glob("*.html"):
            if re.search(r"\{\{[^}]+\}\}|\{stage\.key\}", path.read_text(encoding="utf-8")):
                unresolved.append(path.name)
        return SensorResult(
            "placeholders",
            "failed" if unresolved else "passed",
            f"Unresolved placeholders: {', '.join(unresolved)}"
            if unresolved
            else "No unresolved template placeholders",
            tuple(unresolved),
            str(root),
            "Render every template value before publishing." if unresolved else None,
        )

    @staticmethod
    def _read_json(path: Path) -> dict[str, Any]:
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            return {}
        return payload if isinstance(payload, dict) else {}

    @staticmethod
    def _contrast_ratio(foreground: str, background: str) -> float:
        def luminance(color: str) -> float:
            channels = [int(color[index : index + 2], 16) / 255 for index in (1, 3, 5)]
            linear = [
                channel / 12.92 if channel <= 0.04045 else ((channel + 0.055) / 1.055) ** 2.4
                for channel in channels
            ]
            return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2]

        high, low = sorted((luminance(foreground), luminance(background)), reverse=True)
        return (high + 0.05) / (low + 0.05)


class DemoQualitySensors(VisualQualityValidator):
    """Backward-compatible name for the Product Demo quality validator."""
