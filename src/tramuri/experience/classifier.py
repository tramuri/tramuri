"""Explainable product-experience classification from persisted PDLC context."""

from __future__ import annotations

import re
import unicodedata
from collections.abc import Mapping
from dataclasses import asdict, dataclass
from enum import StrEnum
from typing import Any


class ExperienceCategory(StrEnum):
    MOBILE_APP = "mobile_app"
    WEB_SAAS = "web_saas"
    CLI = "cli_developer_tool"
    DESKTOP = "desktop_app"
    API = "api_backend"
    DATA_PIPELINE = "data_ai_pipeline"
    IOT = "iot_hardware"
    HYBRID = "hybrid"
    OTHER = "other"


@dataclass(frozen=True, slots=True)
class ContextFragment:
    source: str
    role: str
    text: str


@dataclass(frozen=True, slots=True)
class ClassificationResult:
    category: str
    confidence: float
    ambiguous: bool
    evidence: tuple[dict[str, str], ...]
    candidate_scores: dict[str, float]
    components: tuple[str, ...] = ()
    override: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self) | {
            "evidence": [dict(item) for item in self.evidence],
            "components": list(self.components),
        }


_ROLE_WEIGHT = {
    "platform": 4.0,
    "intent": 2.8,
    "description": 2.3,
    "workflow": 1.8,
    "feature": 1.4,
    "constraint": 1.3,
    "audience": 1.0,
    "decision": 0.8,
    "evidence": 0.6,
}

_CUES: dict[str, tuple[tuple[str, float], ...]] = {
    ExperienceCategory.MOBILE_APP: (
        ("mobile app", 2.6),
        ("mobile application", 2.6),
        ("aplicativo movel", 2.6),
        ("app movel", 2.5),
        ("ios", 2.5),
        ("android", 2.5),
        ("smartphone", 2.5),
        ("tablet", 1.8),
        ("mobile", 1.9),
        ("touch-first", 1.4),
        ("field app", 1.8),
        ("on the go", 1.3),
    ),
    ExperienceCategory.WEB_SAAS: (
        ("online marketplace", 2.7),
        ("marketplace", 2.4),
        ("ecommerce", 2.5),
        ("e-commerce", 2.5),
        ("seller platform", 2.4),
        ("product listings", 2.2),
        ("web application", 2.6),
        ("web app", 2.5),
        ("web saas", 2.6),
        ("saas", 2.2),
        ("browser", 2.1),
        ("website", 1.8),
        ("web", 1.4),
        ("painel web", 2.6),
        ("plataforma web", 2.6),
        ("web dashboard", 2.5),
    ),
    ExperienceCategory.CLI: (
        ("command-line", 2.7),
        ("command line", 2.7),
        ("cli", 2.6),
        ("terminal", 2.4),
        ("shell", 2.2),
        ("developer tool", 2.0),
        ("linha de comando", 2.7),
        ("ferramenta de linha de comando", 2.8),
    ),
    ExperienceCategory.DESKTOP: (
        ("desktop application", 2.8),
        ("desktop app", 2.7),
        ("native desktop", 2.8),
        ("windows application", 2.7),
        ("macos desktop", 2.7),
        ("desktop", 2.0),
        ("aplicativo desktop", 2.7),
        ("aplicacao desktop", 2.7),
        ("electron app", 2.4),
    ),
    ExperienceCategory.API: (
        ("rest api", 2.8),
        ("graphql api", 2.8),
        ("api", 2.4),
        ("backend", 2.1),
        ("endpoint", 2.2),
        ("request and response", 2.1),
        ("request/response", 2.1),
        ("webhook", 2.0),
        ("http route", 1.7),
    ),
    ExperienceCategory.DATA_PIPELINE: (
        ("data pipeline", 2.8),
        ("ai pipeline", 2.8),
        ("etl", 2.6),
        ("data ingestion", 2.3),
        ("data transformation", 2.3),
        ("batch processing", 2.4),
        ("model training", 2.3),
        ("machine learning pipeline", 2.8),
        ("pipeline", 2.0),
    ),
    ExperienceCategory.IOT: (
        ("iot", 2.8),
        ("internet of things", 2.8),
        ("hardware", 2.3),
        ("firmware", 2.4),
        ("device telemetry", 2.7),
        ("sensor telemetry", 2.7),
        ("sensor reading", 2.3),
        ("telemetry", 2.0),
        ("device", 1.8),
    ),
}

_FIELD_ROLES: dict[str, str] = {
    "platform": "platform",
    "platforms": "platform",
    "surface": "platform",
    "surfaces": "platform",
    "interfaces": "platform",
    "channels": "platform",
    "intent": "intent",
    "idea": "intent",
    "description": "description",
    "summary": "description",
    "problem": "description",
    "problem_statement": "description",
    "value": "description",
    "value_proposition": "description",
    "target_users": "audience",
    "audience": "audience",
    "users": "audience",
    "actors": "audience",
    "features": "feature",
    "capabilities": "feature",
    "in_scope": "feature",
    "core_journey": "workflow",
    "journeys": "workflow",
    "flows": "workflow",
    "workflows": "workflow",
    "user_journey": "workflow",
    "steps": "workflow",
    "constraints": "constraint",
    "technical_constraints": "constraint",
    "technology": "constraint",
    "decision_text": "decision",
    "rationale": "decision",
    "claim": "evidence",
    "statement": "evidence",
}


def _normalize(value: str) -> str:
    return unicodedata.normalize("NFKD", value).encode("ascii", "ignore").decode("ascii").casefold()


def _flatten(value: Any) -> str:
    if isinstance(value, Mapping):
        return " ".join(_flatten(item) for item in value.values())
    if isinstance(value, (list, tuple, set)):
        return " ".join(_flatten(item) for item in value)
    if value is None:
        return ""
    return str(value)


def _field_fragments(payload: Mapping[str, Any], source: str) -> list[ContextFragment]:
    fragments: list[ContextFragment] = []
    for key, value in payload.items():
        role = _FIELD_ROLES.get(str(key).casefold())
        if role is None or str(key).casefold() == "out_of_scope":
            continue
        text = _flatten(value).strip()
        if text:
            fragments.append(ContextFragment(f"{source}.{key}", role, text))
    return fragments


class ProductExperienceClassifier:
    """Classify a product's dominant experience from its PDLC structure and meaning."""

    def __init__(self) -> None:
        self._cues: dict[str | ExperienceCategory, tuple[tuple[str, float], ...]] = dict(_CUES)
        self._category_values = [category.value for category in ExperienceCategory]

    @property
    def categories(self) -> tuple[str, ...]:
        return tuple(self._category_values)

    def register_category(self, category: str, cues: tuple[tuple[str, float], ...]) -> None:
        """Extend deterministic classification with a domain-specific category."""
        category = category.strip().casefold()
        if not category or category in self._category_values:
            raise ValueError(
                f"Product experience category {category!r} is empty or already registered"
            )
        if category in {ExperienceCategory.HYBRID.value, ExperienceCategory.OTHER.value}:
            raise ValueError(f"{category!r} is a reserved Product Experience category")
        if not cues or any(not phrase.strip() or weight <= 0 for phrase, weight in cues):
            raise ValueError("A custom category needs non-empty phrases with positive weights")
        self._cues[category] = tuple((_normalize(phrase), float(weight)) for phrase, weight in cues)
        self._category_values.insert(-2, category)

    def classify(
        self, details: Mapping[str, Any], override: str | None = None
    ) -> ClassificationResult:
        """Score product intent, platforms, audience, workflows, artifacts and decisions."""
        fragments = self._fragments(details)
        scores = {category: 0.0 for category in self.categories if category != "hybrid"}
        evidence: list[dict[str, str]] = []
        category_sources: dict[str, set[str]] = {category: set() for category in scores}

        for fragment in fragments:
            normalized = _normalize(fragment.text)
            role_weight = _ROLE_WEIGHT.get(fragment.role, 0.7)
            for registered_category, cues in self._cues.items():
                category = (
                    registered_category.value
                    if isinstance(registered_category, ExperienceCategory)
                    else registered_category
                )
                for cue, signal_weight in cues:
                    if self._contains(normalized, cue):
                        score = role_weight * signal_weight
                        scores[category] += score
                        category_sources[category].add(fragment.source)
                        evidence.append(
                            {
                                "category": category,
                                "signal": cue,
                                "source": fragment.source,
                                "context": fragment.role,
                            }
                        )

        ordered = sorted(scores.items(), key=lambda item: (-item[1], item[0]))
        best_category, best_score = ordered[0]
        next_score = ordered[1][1]
        components = tuple(
            category
            for category, score in ordered
            if score >= 8.0 and len(components_source(category_sources, category)) > 0
        )[:3]

        explicit_hybrid = any(
            self._contains(_normalize(fragment.text), "hybrid product")
            or self._contains(_normalize(fragment.text), "produto hibrido")
            for fragment in fragments
            if fragment.role in {"intent", "description", "platform"}
        )
        hybrid_evidence = len(components) >= 2
        if hybrid_evidence or explicit_hybrid:
            category = ExperienceCategory.HYBRID.value
            if explicit_hybrid and not components:
                evidence.append(
                    {
                        "category": category,
                        "signal": "explicit hybrid-product context",
                        "source": "product intent",
                        "context": "intent",
                    }
                )
            confidence = min(0.97, min((scores[item] for item in components), default=8.0) / 16.0)
            ambiguous = confidence < 0.45
        elif best_score < 5.0:
            category = ExperienceCategory.OTHER.value
            confidence = 0.25 if best_score == 0 else min(0.45, best_score / 14.0)
            ambiguous = True
            components = ()
            evidence.append(
                {
                    "category": category,
                    "signal": "no experience category is strongly supported by the available context",
                    "source": "classifier inference",
                    "context": "ambiguity",
                }
            )
        else:
            category = best_category
            separation = max(0.35, 1 - next_score / max(best_score * 1.8, 1.0))
            confidence = round(min(0.98, (best_score / 18.0) * separation + 0.35), 3)
            ambiguous = confidence < 0.58 or best_score - next_score < 3.0
            components = (category,)

        selected_override = override or self._override_from_details(details)
        if selected_override:
            selected_override = selected_override.strip().casefold()
            if selected_override not in self.categories:
                choices = ", ".join(self.categories)
                raise ValueError(
                    f"Unsupported experience category {selected_override!r}; choose {choices}"
                )
            category = selected_override
            confidence = 1.0
            ambiguous = False
            if category != ExperienceCategory.HYBRID.value:
                components = (category,)
            evidence.append(
                {
                    "category": category,
                    "signal": f"manual classification correction: {category}",
                    "source": "user override",
                    "context": "decision",
                }
            )

        candidate_scores = {key: round(value, 2) for key, value in ordered}
        candidate_scores[ExperienceCategory.HYBRID.value] = (
            round(min((scores[item] for item in components), default=0.0), 2)
            if category == ExperienceCategory.HYBRID.value
            else 0.0
        )
        return ClassificationResult(
            category,
            round(confidence, 3),
            ambiguous,
            tuple(evidence),
            candidate_scores,
            components,
            selected_override,
        )

    def _fragments(self, details: Mapping[str, Any]) -> list[ContextFragment]:
        fragments: list[ContextFragment] = []
        run = details.get("run", {})
        if isinstance(run, Mapping):
            idea = str(run.get("idea", "")).strip()
            if idea:
                fragments.append(ContextFragment("run.idea", "intent", idea))
        artifacts = details.get("artifacts", [])
        if isinstance(artifacts, (list, tuple)):
            for index, artifact in enumerate(artifacts):
                if not isinstance(artifact, Mapping):
                    continue
                kind = str(artifact.get("kind", f"artifact[{index}]"))
                payload = artifact.get("payload", {})
                if isinstance(payload, Mapping):
                    fragments.extend(_field_fragments(payload, kind))
        for name, role in (
            ("decisions", "decision"),
            ("evidence", "evidence"),
            ("hypotheses", "workflow"),
        ):
            values = details.get(name, [])
            if isinstance(values, (list, tuple)):
                for index, item in enumerate(values):
                    fragments.append(ContextFragment(f"{name}[{index}]", role, _flatten(item)))
        return [fragment for fragment in fragments if fragment.text]

    def _override_from_details(self, details: Mapping[str, Any]) -> str | None:
        direct = details.get("experience_category_override")
        if direct:
            return str(direct)
        events = details.get("events", [])
        if isinstance(events, (list, tuple)):
            for event in reversed(events):
                if not isinstance(event, Mapping):
                    continue
                if event.get("event_type") == "experience.classification_override":
                    payload = event.get("payload", {})
                    if isinstance(payload, Mapping) and payload.get("category"):
                        return str(payload["category"])
        return None

    @staticmethod
    def _contains(text: str, cue: str) -> bool:
        return re.search(rf"(?<![a-z0-9]){re.escape(cue)}(?![a-z0-9])", text) is not None


def components_source(category_sources: dict[str, set[str]], category: str) -> set[str]:
    """Return context sources for a category, keeping the hybrid check explicit."""
    return category_sources.get(category, set())
