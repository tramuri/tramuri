"""Run the Product Demo Engine inside the existing Product Experience flow."""

from __future__ import annotations

import json
from collections.abc import Mapping
from pathlib import Path
from typing import Any

from tramuri.core.paths import output_root
from tramuri.experience.classifier import ProductExperienceClassifier
from tramuri.experience.models import DemoResult, ProductSurface
from tramuri.experience.registry import DemoTemplateRegistry
from tramuri.experience.renderer import DemoRenderer, default_template_registry
from tramuri.experience.spec import DemoSpecification, DemoSpecificationBuilder


class ProductDemoBuilder:
    """Classify PDLC context, build a versioned spec and publish offline demo files."""

    def __init__(
        self,
        project_dir: Path,
        *,
        classifier: ProductExperienceClassifier | None = None,
        registry: DemoTemplateRegistry | None = None,
    ):
        self.project_dir = project_dir.resolve()
        self.registry = registry or default_template_registry()
        self.classifier = classifier or ProductExperienceClassifier()
        self.specifications = DemoSpecificationBuilder(self.registry)
        self.renderer = DemoRenderer(self.registry)

    def classify(self, details: Mapping[str, Any], override: str | None = None) -> dict[str, Any]:
        """Expose the explainable category decision for preview and correction."""
        return self.classifier.classify(details, override).to_dict()

    def build(self, details: dict[str, Any]) -> DemoResult:
        """Build from current persisted artifacts; this does not run product discovery."""
        classification = self.classifier.classify(details)
        spec = self.specifications.build(details, classification)
        return self._publish(spec)

    def render_spec(self, spec_path: Path) -> DemoResult:
        """Regenerate a demo from a stored spec without loading PDLC context."""
        try:
            payload = json.loads(spec_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            raise ValueError(f"Product Demo spec is not valid JSON: {spec_path}") from exc
        if not isinstance(payload, Mapping):
            raise ValueError("Product Demo spec must contain a JSON object")
        spec = DemoSpecification.from_dict(payload)
        template = self.registry.by_id(spec.template_id)
        if spec.category not in template.categories:
            raise ValueError(
                f"Template {spec.template_id!r} does not support category {spec.category!r}"
            )
        return self._publish(spec)

    def _publish(self, spec: DemoSpecification) -> DemoResult:
        root = output_root(self.project_dir) / "demo"
        root.mkdir(parents=True, exist_ok=True)
        rendered = self.renderer.render(spec.to_dict())
        for name, content in rendered.items():
            self._write(root / name, content)

        spec_path = root / "demo_spec.json"
        self._write(
            spec_path,
            json.dumps(spec.to_dict(), ensure_ascii=False, indent=2) + "\n",
        )
        traceability = self._traceability(spec)
        traceability_path = root / "traceability.json"
        self._write(
            traceability_path,
            json.dumps(traceability, ensure_ascii=False, indent=2) + "\n",
        )
        files = tuple(root / name for name in sorted(rendered)) + (spec_path, traceability_path)
        return DemoResult(
            root=root,
            index=root / "index.html",
            files=files,
            screens=tuple(item.id for item in spec.screens),
            source_artifacts=spec.source_artifacts,
            synthetic=True,
            surfaces=self._surfaces(spec),
            category=spec.category,
            template_id=spec.template_id,
            specification=spec_path,
        )

    def _traceability(self, spec: DemoSpecification) -> dict[str, Any]:
        screen_sources = {item.id: list(item.source_artifacts) for item in spec.screens}
        interaction_sources = {item.id: list(item.source_artifacts) for item in spec.interactions}
        return {
            "schema_version": "tramuri.demo-traceability.v1",
            "run_id": spec.run_id,
            "synthetic": True,
            "simulation_level": spec.simulation_level,
            "product_name": spec.product_name,
            "category": spec.category,
            "template_id": spec.template_id,
            "domain": spec.domain,
            "platform": spec.platform,
            "classification": spec.classification,
            "visual_direction": spec.visual_direction,
            "source_artifacts": list(spec.source_artifacts),
            "surfaces": [surface.key for surface in self._surfaces(spec)],
            "screens": [item.id for item in spec.screens],
            "interactions": [item.id for item in spec.interactions],
            "screen_sources": screen_sources,
            "interaction_sources": interaction_sources,
            "evidence_note": (
                "All interactive outcomes and values are local simulations. "
                "No integration, real user validation, or production behavior is claimed."
            ),
        }

    def _surfaces(self, spec: DemoSpecification) -> tuple[ProductSurface, ...]:
        groups: dict[str, tuple[str, str]] = {
            "mobile": ("mobile-app", "Mobile app"),
            "cli": ("terminal", "Interactive terminal"),
            "desktop": ("desktop-app", "Desktop application"),
            "api": ("api-console", "API console"),
            "pipeline": ("pipeline-canvas", "Data pipeline"),
            "iot": ("device-panel", "Device telemetry"),
            "hybrid": ("hybrid-product", "Connected product surfaces"),
            "other": ("concept-composition", "Contextual concept"),
        }
        if spec.template_id == "web":
            groups["web"] = (
                (
                    "booking",
                    "Customer booking" if spec.domain == "service_booking" else "Course catalog",
                )
                if spec.domain in {"service_booking", "education"}
                else ("workspace", "Project workspace")
            )
            if spec.domain in {"service_booking", "education"}:
                return (
                    ProductSurface(
                        "marketing",
                        "Marketing",
                        ("home", "catalog") if spec.domain == "education" else ("home", "service"),
                    ),
                    ProductSurface(
                        "customer",
                        "Customer",
                        tuple(item.id for item in spec.screens if item.id != "admin"),
                        tuple(item.id for item in spec.interactions),
                    ),
                    ProductSurface(
                        "admin",
                        "Creator admin" if spec.domain == "education" else "Service admin",
                        ("course-editor", "admin") if spec.domain == "education" else ("admin",),
                        ("publish-course",) if spec.domain == "education" else (),
                    ),
                )
        if spec.template_id == "web" and spec.domain == "marketplace":
            return (
                ProductSurface(
                    "storefront",
                    "Marketplace storefront",
                    ("home", "catalog", "product", "cart", "complete"),
                    tuple(item.id for item in spec.interactions),
                ),
                ProductSurface("seller", "Seller workspace", ("seller",)),
            )
        surface_id, surface_label = groups.get(
            spec.template_id, (spec.template_id, "Product surface")
        )
        return (
            ProductSurface(
                surface_id,
                surface_label,
                tuple(item.id for item in spec.screens),
                tuple(item.id for item in spec.interactions),
            ),
        )

    @staticmethod
    def _write(path: Path, content: str) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
