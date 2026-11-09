"""Versioned intermediate specification for reproducible offline product demos."""

from __future__ import annotations

import re
import unicodedata
from collections.abc import Mapping
from dataclasses import asdict, dataclass
from typing import Any

from tramuri.experience.classifier import ClassificationResult, ProductExperienceClassifier
from tramuri.experience.registry import DemoTemplateRegistry
from tramuri.modules.pm.intent import parse_intent

SPEC_VERSION = "tramuri.demo.v1"

_PALETTES: dict[str, dict[str, str]] = {
    "personal_finance": {
        "background": "#f4f7f2",
        "surface": "#ffffff",
        "ink": "#17271d",
        "muted": "#506257",
        "accent": "#26724e",
        "accent_soft": "#dcefe2",
        "line": "#d8e2d9",
        "highlight": "#d8ec80",
    },
    "developer_tools": {
        "background": "#101619",
        "surface": "#182226",
        "ink": "#e9f0ee",
        "muted": "#b0c0ba",
        "accent": "#a5e071",
        "accent_soft": "#2a3b2d",
        "line": "#3a4b47",
        "highlight": "#d8f189",
    },
    "project_management": {
        "background": "#f6f5f1",
        "surface": "#ffffff",
        "ink": "#242532",
        "muted": "#5e6070",
        "accent": "#6253bd",
        "accent_soft": "#e9e5fb",
        "line": "#dfddea",
        "highlight": "#e4eea6",
    },
    "api_infrastructure": {
        "background": "#f2f6f8",
        "surface": "#ffffff",
        "ink": "#172d3a",
        "muted": "#506876",
        "accent": "#176c74",
        "accent_soft": "#d8ecea",
        "line": "#d5e0e4",
        "highlight": "#d9ebc2",
    },
    "data_pipeline": {
        "background": "#f5f3fb",
        "surface": "#ffffff",
        "ink": "#271f42",
        "muted": "#655b7b",
        "accent": "#6954ae",
        "accent_soft": "#e5def5",
        "line": "#dcd5eb",
        "highlight": "#e8c99c",
    },
    "device_operations": {
        "background": "#f2f6f6",
        "surface": "#ffffff",
        "ink": "#18313c",
        "muted": "#536b72",
        "accent": "#a0581a",
        "accent_soft": "#f4e4d1",
        "line": "#d7e1e1",
        "highlight": "#a8d6ca",
    },
    "education": {
        "background": "#f5f4fa",
        "surface": "#ffffff",
        "ink": "#28233d",
        "muted": "#615c75",
        "accent": "#6555a0",
        "accent_soft": "#e9e4f5",
        "line": "#ded9eb",
        "highlight": "#efcb83",
    },
    "service_booking": {
        "background": "#f7f5ef",
        "surface": "#ffffff",
        "ink": "#292b2a",
        "muted": "#62655f",
        "accent": "#356f68",
        "accent_soft": "#dcece6",
        "line": "#e2dfd5",
        "highlight": "#efc277",
    },
    "marketplace": {
        "background": "#f7f5ef",
        "surface": "#ffffff",
        "ink": "#242820",
        "muted": "#5f6559",
        "accent": "#895c33",
        "accent_soft": "#f0e5d7",
        "line": "#e2ddd4",
        "highlight": "#e1c77e",
    },
    "general": {
        "background": "#f4f6f5",
        "surface": "#ffffff",
        "ink": "#1c2927",
        "muted": "#576965",
        "accent": "#376c62",
        "accent_soft": "#dcebe6",
        "line": "#d9e2df",
        "highlight": "#d9e6a7",
    },
}


@dataclass(frozen=True, slots=True)
class DemoScreen:
    id: str
    title: str
    purpose: str
    source_artifacts: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class DemoInteraction:
    id: str
    label: str
    effect: str
    source_artifacts: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class DemoSpecification:
    version: str
    run_id: str
    product_name: str
    description: str
    category: str
    template_id: str
    classification: dict[str, Any]
    domain: str
    platform: str
    audience: tuple[str, ...]
    features: tuple[str, ...]
    primary_journey: str
    constraints: tuple[str, ...]
    source_artifacts: tuple[str, ...]
    screens: tuple[DemoScreen, ...]
    interactions: tuple[DemoInteraction, ...]
    visual_direction: dict[str, Any]
    simulation_level: str = "interactive_simulation"

    def to_dict(self) -> dict[str, Any]:
        payload = asdict(self)
        for key in ("audience", "features", "constraints", "source_artifacts"):
            payload[key] = list(payload[key])
        payload["screens"] = [
            asdict(item) | {"source_artifacts": list(item.source_artifacts)}
            for item in self.screens
        ]
        payload["interactions"] = [
            asdict(item) | {"source_artifacts": list(item.source_artifacts)}
            for item in self.interactions
        ]
        payload["classification"] = dict(self.classification)
        payload["visual_direction"] = dict(self.visual_direction)
        return payload

    @classmethod
    def from_dict(cls, value: Mapping[str, Any]) -> DemoSpecification:
        """Validate and load a persisted spec without rerunning discovery."""
        if value.get("version") != SPEC_VERSION:
            raise ValueError(f"Unsupported Product Demo spec version: {value.get('version')!r}")
        required = (
            "run_id",
            "product_name",
            "category",
            "template_id",
            "classification",
            "source_artifacts",
            "screens",
            "interactions",
            "visual_direction",
        )
        missing = [key for key in required if key not in value]
        if missing:
            raise ValueError(f"Product Demo spec is missing fields: {', '.join(missing)}")
        return cls(
            version=SPEC_VERSION,
            run_id=str(value["run_id"]),
            product_name=str(value["product_name"]),
            description=str(value.get("description", "")),
            category=str(value["category"]),
            template_id=str(value["template_id"]),
            classification=dict(value["classification"]),
            domain=str(value.get("domain", "general")),
            platform=str(value.get("platform", "context-appropriate")),
            audience=tuple(_strings(value.get("audience", []))),
            features=tuple(_strings(value.get("features", []))),
            primary_journey=str(value.get("primary_journey", "")),
            constraints=tuple(_strings(value.get("constraints", []))),
            source_artifacts=tuple(_strings(value["source_artifacts"])),
            screens=tuple(
                DemoScreen(
                    str(item["id"]),
                    str(item["title"]),
                    str(item.get("purpose", "")),
                    tuple(_strings(item.get("source_artifacts", []))),
                )
                for item in _mapping_list(value["screens"])
            ),
            interactions=tuple(
                DemoInteraction(
                    str(item["id"]),
                    str(item["label"]),
                    str(item.get("effect", "")),
                    tuple(_strings(item.get("source_artifacts", []))),
                )
                for item in _mapping_list(value["interactions"])
            ),
            visual_direction=dict(value["visual_direction"]),
            simulation_level=str(value.get("simulation_level", "interactive_simulation")),
        )


class DemoSpecificationBuilder:
    """Convert existing product artifacts into a versioned, replayable demo contract."""

    def __init__(self, registry: DemoTemplateRegistry) -> None:
        self.registry = registry

    def build(
        self,
        details: Mapping[str, Any],
        classification: ClassificationResult | None = None,
    ) -> DemoSpecification:
        run = _mapping(details.get("run", {}))
        idea = str(run.get("idea", ""))
        profile = parse_intent(idea)
        artifacts = _artifact_map(details)
        framing = artifacts.get("pm.framing", {})
        shaping = artifacts.get("pm.shaping", {})
        ux = artifacts.get("pm.ux", {})
        navigation = artifacts.get("pm.navigation", {})
        title = str(framing.get("product_name") or profile.title or "Product concept")
        description = str(
            framing.get("description")
            or framing.get("value")
            or framing.get("problem")
            or profile.intent
            or idea
        )
        classification = classification or ProductExperienceClassifier().classify(details)
        category = classification.category
        template = self.registry.for_category(category)
        source_artifacts = tuple(artifacts) or ("pm.intent",)
        features = _strings(shaping.get("in_scope", [])) or _strings(framing.get("features", []))
        audience = (
            _strings(framing.get("target_users", []))
            or _strings(framing.get("audience", []))
            or tuple(profile.audience)
        )
        journey_values = _strings(
            ux.get("journeys")
            or ux.get("flows")
            or navigation.get("journeys")
            or framing.get("core_journey")
        )
        primary_journey = journey_values[0] if journey_values else profile.desired_outcome
        constraints = _strings(framing.get("constraints", [])) or tuple(profile.constraints)
        domain = self._domain(
            " ".join((idea, description, " ".join(features), " ".join(journey_values))),
            profile.archetype,
        )
        platform = self._platform(category, _platforms(framing, ux), classification.components)
        visual_direction = self._visual_direction(category, domain, artifacts)
        screen_definitions = (
            self._screen_definitions(template.template_id, domain) or template.screens
        )
        interaction_definitions = self._interaction_definitions(template.template_id, domain)
        if not interaction_definitions:
            interaction_definitions = template.interactions
        screens = tuple(
            DemoScreen(identifier, label, purpose, source_artifacts)
            for identifier, label, purpose in screen_definitions
        )
        interactions = tuple(
            DemoInteraction(identifier, label, effect, source_artifacts)
            for identifier, label, effect in interaction_definitions
        )
        return DemoSpecification(
            SPEC_VERSION,
            str(run.get("id", "unsaved-product")),
            title,
            description,
            category,
            template.template_id,
            classification.to_dict(),
            domain,
            platform,
            tuple(audience),
            tuple(features),
            primary_journey,
            tuple(constraints),
            source_artifacts,
            screens,
            interactions,
            visual_direction,
        )

    def _screen_definitions(
        self, template_id: str, domain: str
    ) -> tuple[tuple[str, str, str], ...]:
        if template_id == "web" and domain == "service_booking":
            return (
                ("home", "Booking overview", "Begin the customer booking journey"),
                ("service", "Service", "Choose a service"),
                ("availability", "Availability", "Choose a sample time"),
                ("confirmation", "Confirmation", "Confirm the simulated booking"),
                ("complete", "Complete", "Review the local booking state"),
            )
        if template_id == "web" and domain == "marketplace":
            return (
                ("home", "Storefront", "Find the relevant product collection"),
                ("catalog", "Catalog", "Browse synthetic product listings"),
                ("product", "Product detail", "Review a selected sample listing"),
                ("cart", "Basket", "Inspect products added to local basket state"),
                ("complete", "Order complete", "Review a simulated checkout result"),
                ("seller", "Seller workspace", "Review the separate seller-facing surface"),
            )
        if template_id == "web" and domain == "education":
            return (
                ("home", "Learning overview", "Begin a learning journey"),
                ("catalog", "Catalog", "Explore available courses"),
                ("checkout", "Enrollment", "Confirm simulated enrollment"),
                ("progress", "Progress", "Update lesson completion"),
                ("complete", "Complete", "Review local learning progress"),
                ("course", "Course", "Inspect the selected course"),
                ("course-editor", "Course editor", "Edit the simulated course state"),
                ("admin", "Creator signals", "Review sample creator state"),
            )
        return ()

    def _domain(self, text: str, intent_domain: str) -> str:
        normalized = _normalize(text)
        if intent_domain == "learning" or any(
            term in normalized
            for term in ("course", "learner", "lesson", "student", "curso", "aluno")
        ):
            return "education"
        if any(
            term in normalized
            for term in (
                "marketplace",
                "ecommerce",
                "e-commerce",
                "seller",
                "shoppers",
                "product listings",
            )
        ):
            return "marketplace"
        if intent_domain == "appointment" or any(
            term in normalized
            for term in ("appointment", "booking", "service selection", "agend", "consulta")
        ):
            return "service_booking"
        if any(
            term in normalized
            for term in (
                "expense",
                "budget",
                "finance",
                "spending",
                "despesa",
                "orcamento",
                "financas",
            )
        ):
            return "personal_finance"
        if any(
            term in normalized
            for term in ("project board", "project management", "task", "kanban", "project team")
        ):
            return "project_management"
        if any(
            term in normalized
            for term in ("pipeline", "etl", "data ingestion", "batch processing", "model training")
        ):
            return "data_pipeline"
        if any(
            term in normalized
            for term in ("sensor", "telemetry", "hardware", "device", "equipment reading")
        ):
            return "device_operations"
        if any(
            term in normalized
            for term in ("rest api", "graphql", "endpoint", "backend api", "request and response")
        ):
            return "api_infrastructure"
        if any(
            term in normalized for term in ("developer", "terminal", "command-line", "cli tool")
        ):
            return "developer_tools"
        return "general"

    def _platform(
        self, category: str, supplied: tuple[str, ...], components: tuple[str, ...]
    ) -> str:
        platform_text = " ".join(supplied).casefold()
        if category == "mobile_app":
            if "ios" in platform_text and "android" in platform_text:
                return "cross-platform iOS and Android"
            if "ios" in platform_text:
                return "iOS"
            if "android" in platform_text:
                return "Android"
            return "cross-platform mobile proposal"
        if category == "hybrid":
            return (
                " + ".join(_platform_label(item) for item in components[:3])
                or "multiple product surfaces"
            )
        return {
            "cli_developer_tool": "interactive terminal",
            "web_saas": "web browser",
            "desktop_app": "desktop application window",
            "api_backend": "request and response console",
            "data_ai_pipeline": "pipeline execution canvas",
            "iot_hardware": "device and telemetry panel",
            "other": "contextual concept composition",
        }.get(category, "context-appropriate product surface")

    def _visual_direction(
        self, category: str, domain: str, artifacts: dict[str, dict[str, Any]]
    ) -> dict[str, Any]:
        identity: Any = None
        for payload in artifacts.values():
            identity = (
                payload.get("brand_identity")
                or payload.get("visual_direction")
                or payload.get("brand")
            )
            if identity:
                break
        palette_domain = domain if domain in _PALETTES else _category_palette(category)
        palette = dict(_PALETTES.get(palette_domain, _PALETTES["general"]))
        supplied = _brand_colors(identity)
        palette.update(supplied)
        label = "product identity from supplied brand"
        provenance = "user_supplied"
        if not supplied:
            label = f"proposed {domain.replace('_', ' ')} direction"
            provenance = "design_proposal"
        return {
            "label": label,
            "provenance": provenance,
            "palette_domain": palette_domain,
            "colors": palette,
            "type": "offline system sans with a monospace face for code and measurements",
            "principles": [
                "Product-specific information hierarchy",
                "Semantic HTML and visible keyboard focus",
                "Responsive layout with touch targets",
                "Reduced motion and local system fonts",
            ],
        }

    def _interaction_definitions(
        self, template_id: str, domain: str
    ) -> tuple[tuple[str, str, str], ...]:
        if template_id == "web" and domain == "service_booking":
            return (
                ("choose-service", "Choose service", "Advance to a local availability state"),
                ("choose-time", "Choose time", "Select a simulated slot"),
                ("confirm", "Confirm demo booking", "Show a local confirmation"),
            )
        if template_id == "web" and domain == "marketplace":
            return (
                ("browse-catalog", "Browse product catalog", "Open the storefront collection"),
                ("add-to-cart", "Add product to basket", "Update the local basket count"),
                ("checkout", "Complete sample checkout", "Show a simulated order confirmation"),
            )
        if template_id == "web" and domain == "education":
            return (
                ("browse-catalog", "Explore catalog", "Show available learning paths"),
                ("enroll", "View course", "Open a simulated course selection"),
                ("confirm-enrollment", "Start course", "Update simulated enrollment progress"),
                ("mark-progress", "Mark lesson complete", "Update local learning progress"),
                ("publish-course", "Publish simulated course", "Update the local creator state"),
            )
        return ()


def _artifact_map(details: Mapping[str, Any]) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for artifact in details.get("artifacts", []):
        if not isinstance(artifact, Mapping):
            continue
        kind = str(artifact.get("kind", ""))
        payload = artifact.get("payload", {})
        if kind and isinstance(payload, Mapping):
            result[kind] = dict(payload)
    return result


def _mapping(value: Any) -> dict[str, Any]:
    return dict(value) if isinstance(value, Mapping) else {}


def _mapping_list(value: Any) -> list[Mapping[str, Any]]:
    if not isinstance(value, list) or any(not isinstance(item, Mapping) for item in value):
        raise ValueError("Product Demo spec lists must contain objects")
    return value


def _strings(value: Any) -> list[str]:
    if value is None:
        return []
    values = value if isinstance(value, (list, tuple)) else [value]
    return [str(item).strip() for item in values if str(item).strip()]


def _platforms(*payloads: dict[str, Any]) -> tuple[str, ...]:
    values: list[str] = []
    for payload in payloads:
        for key in ("platform", "platforms", "surfaces", "interfaces", "channels"):
            values.extend(_strings(payload.get(key)))
    return tuple(dict.fromkeys(values))


def _platform_label(category: str) -> str:
    return {
        "mobile_app": "mobile app",
        "web_saas": "web dashboard",
        "cli_developer_tool": "CLI",
        "desktop_app": "desktop app",
        "api_backend": "API console",
        "data_ai_pipeline": "data pipeline",
        "iot_hardware": "IoT panel",
        "other": "concept surface",
    }.get(category, category)


def _category_palette(category: str) -> str:
    return {
        "mobile_app": "general",
        "web_saas": "general",
        "cli_developer_tool": "developer_tools",
        "desktop_app": "general",
        "api_backend": "api_infrastructure",
        "data_ai_pipeline": "data_pipeline",
        "iot_hardware": "device_operations",
        "hybrid": "device_operations",
        "other": "general",
    }.get(category, "general")


def _normalize(value: str) -> str:
    return unicodedata.normalize("NFKD", value).encode("ascii", "ignore").decode("ascii").casefold()


def _brand_colors(value: Any) -> dict[str, str]:
    if not isinstance(value, Mapping):
        return {}
    source = value.get("colors", value)
    if not isinstance(source, Mapping):
        return {}
    aliases = {
        "background": ("background", "base", "canvas"),
        "surface": ("surface", "card"),
        "ink": ("ink", "foreground", "text", "primary"),
        "muted": ("muted", "secondary"),
        "accent": ("accent", "brand", "cta"),
        "accent_soft": ("accent_soft", "soft", "secondary_surface"),
        "line": ("line", "border"),
        "highlight": ("highlight", "signal"),
    }
    result: dict[str, str] = {}
    for name, candidates in aliases.items():
        for candidate in candidates:
            color = str(source.get(candidate, ""))
            if re.fullmatch(r"#[0-9a-fA-F]{6}", color):
                result[name] = color.lower()
                break
    return result
