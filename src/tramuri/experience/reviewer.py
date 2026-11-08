"""Read-only Product Lead review for demo coherence and traceability."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from tramuri.experience.models import DemoReview, ReviewFinding


class ProductLeadReviewer:
    def review(self, details: dict[str, Any], demo_root: Path) -> DemoReview:
        findings: list[ReviewFinding] = []
        index = demo_root / "index.html"
        traceability = demo_root / "traceability.json"
        html = index.read_text(encoding="utf-8") if index.is_file() else ""
        if not index.is_file():
            findings.append(
                ReviewFinding("error", "demo-file", "Demo index is missing", (str(index),))
            )
        if not traceability.is_file():
            findings.append(
                ReviewFinding(
                    "error", "traceability", "Demo traceability is missing", (str(traceability),)
                )
            )
        else:
            try:
                payload = json.loads(traceability.read_text(encoding="utf-8"))
                if str(payload.get("run_id")) != str(details["run"]["id"]):
                    findings.append(
                        ReviewFinding(
                            "error",
                            "traceability",
                            "Demo run does not match the source run",
                            (str(traceability),),
                        )
                    )
            except json.JSONDecodeError:
                findings.append(
                    ReviewFinding(
                        "error",
                        "traceability",
                        "Traceability is not valid JSON",
                        (str(traceability),),
                    )
                )
        by_kind = {
            item.get("kind"): item.get("payload", {}) for item in details.get("artifacts", [])
        }
        product_name = str(by_kind.get("pm.framing", {}).get("product_name", ""))
        if product_name and product_name.lower() not in html.lower():
            findings.append(
                ReviewFinding(
                    "warning",
                    "product-name",
                    "Demo does not expose the product name from framing",
                    ("pm.framing",),
                )
            )
        for item in by_kind.get("pm.shaping", {}).get("in_scope", []):
            if str(item).lower() not in html.lower():
                findings.append(
                    ReviewFinding(
                        "warning",
                        "scope",
                        f"In-scope item is not traceable in the demo: {item}",
                        ("pm.shaping",),
                    )
                )
        required_ids = ("home", "service", "availability", "confirmation")
        missing = [item for item in required_ids if f'id="{item}"' not in html]
        if missing:
            findings.append(
                ReviewFinding(
                    "error", "journey", "Required journey screens are missing", tuple(missing)
                )
            )
        status = "passed" if not any(item.severity == "error" for item in findings) else "failed"
        return DemoReview(
            status,
            tuple(findings),
            tuple(item.get("kind", "") for item in details.get("artifacts", [])),
        )
