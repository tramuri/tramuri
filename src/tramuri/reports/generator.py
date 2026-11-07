"""Offline HTML report and product-navigation generation."""

from __future__ import annotations

import base64
import shutil
from html import escape
from pathlib import Path
from typing import Any

from jinja2 import Environment, FileSystemLoader, select_autoescape


class ReportGenerator:
    def __init__(self, project_dir: Path):
        self.project_dir = project_dir.resolve()
        template_dir = Path(__file__).parent / "templates"
        self.environment = Environment(
            loader=FileSystemLoader(template_dir),
            autoescape=select_autoescape(("html", "xml")),
        )

    def generate(self, details: dict[str, Any]) -> dict[str, str]:
        run_id = str(details["run"]["id"])
        output_dir = self.project_dir / ".tramuri" / "reports" / run_id
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
        return {"report": str(report_path), "prototype": str(prototype_path)}

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
        return {
            "run": run,
            "stages": details.get("stages", []),
            "artifacts": artifacts,
            "decisions": details.get("decisions", []),
            "hypotheses": details.get("hypotheses", []),
            "evidence": details.get("evidence", []),
            "validations": details.get("validations", []),
            "framing": by_kind.get("pm.framing", {}),
            "discovery": by_kind.get("pm.discovery", {}),
            "shaping": by_kind.get("pm.shaping", {}),
            "success": by_kind.get("pm.success", {}),
            "delivery": by_kind.get("pm.delivery", {}),
            "review": by_kind.get("pm.critical_review", {}),
            "logo_data": logo_data,
            "escape": escape,
        }
