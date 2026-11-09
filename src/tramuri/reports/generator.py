"""Offline HTML report and product-navigation generation."""

from __future__ import annotations

import base64
import shutil
from html import escape
from pathlib import Path
from typing import Any

from jinja2 import Environment, FileSystemLoader, select_autoescape

from tramuri.core.paths import output_root
from tramuri.core.status import build_status
from tramuri.experience.demo import ProductDemoBuilder
from tramuri.experience.portal import ProductPortalBuilder
from tramuri.experience.reviewer import ProductLeadReviewer
from tramuri.experience.sensors import DemoQualitySensors
from tramuri.modules.pm.intent import parse_intent
from tramuri.reports.presentation import (
    build_relationship_graph,
    redact_report_value,
    safe_http_url,
)


class ReportGenerator:
    def __init__(self, project_dir: Path):
        self.project_dir = project_dir.resolve()
        template_dir = Path(__file__).parent / "templates"
        self.environment = Environment(
            loader=FileSystemLoader(template_dir),
            autoescape=select_autoescape(("html", "xml")),
        )

    def generate(self, details: dict[str, Any]) -> dict[str, Any]:
        details = redact_report_value(details)
        run_id = str(details["run"]["id"])
        output_dir = output_root(self.project_dir) / "reports" / run_id
        output_dir.mkdir(parents=True, exist_ok=True)
        context = self._context(details)
        report_path = output_dir / "report.html"
        prototype_path = output_dir / "prototype.html"
        self._archive_previous(report_path, "report")
        self._archive_previous(prototype_path, "prototype")
        report_path.write_text(
            self.environment.get_template("report.html").render(**context), encoding="utf-8"
        )
        prototype_path.write_text(
            self.environment.get_template("prototype.html").render(**context), encoding="utf-8"
        )
        demo = ProductDemoBuilder(self.project_dir).build(details)
        review = ProductLeadReviewer().review(details, demo.root)
        sensors = DemoQualitySensors().run(demo.root, details)
        portal = ProductPortalBuilder(self.project_dir).build(
            details, report_path, demo.index, review, sensors
        )
        return {
            "report": str(report_path),
            "prototype": str(prototype_path),
            "portal": str(portal),
            "demo": str(demo.index),
            "demo_specification": str(demo.specification) if demo.specification else None,
            "demo_category": demo.category,
            "demo_template": demo.template_id,
            "experience_surfaces": [item.key for item in demo.surfaces],
            "review": review.to_dict(),
            "experience_sensors": [item.to_dict() for item in sensors],
        }

    def _archive_previous(self, path: Path, stem: str) -> None:
        if not path.is_file():
            return
        revision = 1
        archived = path.with_name(f"{stem}-v{revision}.html")
        while archived.exists():
            revision += 1
            archived = path.with_name(f"{stem}-v{revision}.html")
        shutil.copy2(path, archived)

    def _context(self, details: dict[str, Any]) -> dict[str, Any]:
        artifacts = details.get("artifacts", [])
        by_kind = {item.get("kind"): item.get("payload", {}) for item in artifacts}
        logo_path = Path(__file__).parent.parent / "assets" / "logo.png"
        logo_data = ""
        if logo_path.is_file():
            logo_data = "data:image/png;base64," + base64.b64encode(logo_path.read_bytes()).decode(
                "ascii"
            )
        run = details["run"]
        status_view = build_status(details)
        intent_profile = parse_intent(str(run.get("idea", ""))).to_dict()
        research = by_kind.get("pm.market_research", {})
        navigation = by_kind.get("pm.navigation", {})
        report_graph = build_relationship_graph(details)
        report_graph["node_by_id"] = {item["id"]: item for item in report_graph["nodes"]}
        stage_name_by_id = {
            str(item.get("id")): str(item.get("stage_key", "")).replace("_", " ").title()
            for item in details.get("stages", [])
            if item.get("id")
        }
        return {
            "run": run,
            "stages": details.get("stages", []),
            "artifacts": artifacts,
            "decisions": details.get("decisions", []),
            "hypotheses": details.get("hypotheses", []),
            "evidence": details.get("evidence", []),
            "validations": details.get("validations", []),
            "checkpoints": details.get("checkpoints", []),
            "knowledge": details.get("knowledge", []),
            "sensor_results": details.get("sensor_results", []),
            "events": details.get("events", []),
            "status_view": status_view,
            "framing": by_kind.get("pm.framing", {}),
            "discovery": by_kind.get("pm.discovery", {}),
            "shaping": by_kind.get("pm.shaping", {}),
            "success": by_kind.get("pm.success", {}),
            "delivery": by_kind.get("pm.delivery", {}),
            "review": by_kind.get("pm.critical_review", {}),
            "report_graph": report_graph,
            "stage_name_by_id": stage_name_by_id,
            "intent_profile": intent_profile,
            "research": research,
            "navigation": navigation,
            "experience_surfaces": ["marketing", "customer", "admin"],
            "logo_data": logo_data,
            "safe_http_url": safe_http_url,
            "escape": escape,
        }
