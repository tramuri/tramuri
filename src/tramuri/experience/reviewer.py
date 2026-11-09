"""Read-only review of Product Demo specification, rendering and traceability."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from tramuri.experience.models import DemoReview, ReviewFinding
from tramuri.experience.spec import SPEC_VERSION


class ProductLeadReviewer:
    """Review the generated artifact against the persisted product contract."""

    def review(self, details: dict[str, Any], demo_root: Path) -> DemoReview:
        findings: list[ReviewFinding] = []
        index = demo_root / "index.html"
        spec_path = demo_root / "demo_spec.json"
        trace_path = demo_root / "traceability.json"
        html = index.read_text(encoding="utf-8") if index.is_file() else ""
        spec = self._read(spec_path)
        trace = self._read(trace_path)
        if not index.is_file():
            findings.append(
                ReviewFinding("error", "demo-file", "Demo index is missing", (str(index),))
            )
        if not spec:
            findings.append(
                ReviewFinding(
                    "error",
                    "demo-spec",
                    "Versioned demo specification is missing or invalid",
                    (str(spec_path),),
                )
            )
        else:
            if spec.get("version") != SPEC_VERSION:
                findings.append(
                    ReviewFinding(
                        "error",
                        "demo-version",
                        "Unsupported demo specification version",
                        (str(spec_path),),
                    )
                )
            if str(spec.get("run_id")) != str(details.get("run", {}).get("id")):
                findings.append(
                    ReviewFinding(
                        "error",
                        "demo-run",
                        "Demo specification belongs to another run",
                        (str(spec_path),),
                    )
                )
            if str(spec.get("product_name", "")).casefold() not in html.casefold():
                findings.append(
                    ReviewFinding(
                        "warning",
                        "product-name",
                        "Rendered demo does not expose its product name",
                        (str(spec_path),),
                    )
                )
            classification = spec.get("classification", {})
            if isinstance(classification, dict) and classification.get("ambiguous"):
                findings.append(
                    ReviewFinding(
                        "warning",
                        "classification",
                        "Product experience category is ambiguous; the generated composition is marked as a concept.",
                        tuple(
                            str(item.get("signal", ""))
                            for item in classification.get("evidence", [])
                            if isinstance(item, dict)
                        ),
                    )
                )
            if (
                spec.get("visual_direction", {}).get("provenance") == "design_proposal"
                and "design proposal" not in html.casefold()
            ):
                findings.append(
                    ReviewFinding(
                        "warning",
                        "visual-direction",
                        "Proposed identity is not identified in the demo",
                        (str(spec_path),),
                    )
                )
        if not trace:
            findings.append(
                ReviewFinding(
                    "error",
                    "traceability",
                    "Demo traceability is missing or invalid",
                    (str(trace_path),),
                )
            )
        else:
            if str(trace.get("run_id")) != str(details.get("run", {}).get("id")):
                findings.append(
                    ReviewFinding(
                        "error",
                        "traceability",
                        "Demo belongs to another source run",
                        (str(trace_path),),
                    )
                )
            for screen in trace.get("screens", []):
                if not trace.get("screen_sources", {}).get(screen):
                    findings.append(
                        ReviewFinding(
                            "error",
                            "screen-trace",
                            f"Screen is missing source links: {screen}",
                            (str(trace_path),),
                        )
                    )
            for action in trace.get("interactions", []):
                if not trace.get("interaction_sources", {}).get(action):
                    findings.append(
                        ReviewFinding(
                            "error",
                            "interaction-trace",
                            f"Interaction is missing source links: {action}",
                            (str(trace_path),),
                        )
                    )
            if trace.get("simulation_level") != "interactive_simulation":
                findings.append(
                    ReviewFinding(
                        "error",
                        "simulation-boundary",
                        "Simulation level is missing or not explicit",
                        (str(trace_path),),
                    )
                )
            if f'data-demo-category="{trace.get("category")}"' not in html:
                findings.append(
                    ReviewFinding(
                        "error",
                        "category-representation",
                        "Rendered environment does not match its recorded category",
                        (str(index), str(trace_path)),
                    )
                )

        source_kinds = {str(item.get("kind", "")) for item in details.get("artifacts", [])}
        for feature in self._in_scope(details):
            if (
                source_kinds
                and "pm.shaping" not in source_kinds
                and feature.casefold() not in html.casefold()
            ):
                findings.append(
                    ReviewFinding(
                        "warning",
                        "scope",
                        f"In-scope item has no rendered evidence: {feature}",
                        ("pm.shaping",),
                    )
                )
        status = "passed" if not any(item.severity == "error" for item in findings) else "failed"
        return DemoReview(
            status,
            tuple(findings),
            tuple(str(item.get("kind", "")) for item in details.get("artifacts", [])),
        )

    @staticmethod
    def _read(path: Path) -> dict[str, Any]:
        try:
            value = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            return {}
        return value if isinstance(value, dict) else {}

    @staticmethod
    def _in_scope(details: dict[str, Any]) -> list[str]:
        for artifact in details.get("artifacts", []):
            if artifact.get("kind") == "pm.shaping":
                values = artifact.get("payload", {}).get("in_scope", [])
                return [str(item) for item in values]
        return []
