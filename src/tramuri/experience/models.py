"""Typed contracts for the Product Experience extension."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any


@dataclass(frozen=True, slots=True)
class ExperienceStage:
    key: str
    label: str
    status: str
    summary: str
    href: str | None = None

    @staticmethod
    def default_map() -> list[ExperienceStage]:
        return [
            ExperienceStage(
                "product-definition",
                "Product Definition",
                "completed",
                "Define the problem, users, scope and success criteria.",
            ),
            ExperienceStage(
                "product-demo",
                "Product Demo",
                "pending",
                "Explore the primary product journey in an offline prototype.",
            ),
            ExperienceStage(
                "product-ux",
                "Product UX",
                "future",
                "A future stage for deeper experience design.",
            ),
            ExperienceStage(
                "product-engineering",
                "Product Engineering",
                "future",
                "A future stage for building the production application.",
            ),
        ]

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True, slots=True)
class SensorResult:
    key: str
    status: str
    message: str
    evidence: tuple[str, ...] = ()
    path: str | None = None
    next_step: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True, slots=True)
class ReviewFinding:
    severity: str
    key: str
    message: str
    evidence: tuple[str, ...] = ()

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True, slots=True)
class DemoReview:
    status: str
    findings: tuple[ReviewFinding, ...] = ()
    checked_artifacts: tuple[str, ...] = ()
    reviewer: str = "tramuri-product-review"

    def to_dict(self) -> dict[str, Any]:
        return {
            "status": self.status,
            "findings": [item.to_dict() for item in self.findings],
            "checked_artifacts": list(self.checked_artifacts),
            "reviewer": self.reviewer,
        }


@dataclass(frozen=True, slots=True)
class ProductSurface:
    key: str
    label: str
    screens: tuple[str, ...] = ()
    interactions: tuple[str, ...] = ()

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True, slots=True)
class DemoResult:
    root: Path
    index: Path
    files: tuple[Path, ...]
    screens: tuple[str, ...]
    source_artifacts: tuple[str, ...]
    synthetic: bool = True
    surfaces: tuple[ProductSurface, ...] = ()
    category: str = "other"
    template_id: str = "other"
    specification: Path | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "root": str(self.root),
            "index": str(self.index),
            "files": [str(item) for item in self.files],
            "screens": list(self.screens),
            "source_artifacts": list(self.source_artifacts),
            "synthetic": self.synthetic,
            "surfaces": [item.to_dict() for item in self.surfaces],
            "category": self.category,
            "template_id": self.template_id,
            "specification": str(self.specification) if self.specification else None,
        }


@dataclass(frozen=True, slots=True)
class ExperienceCheck:
    review: DemoReview
    sensors: tuple[SensorResult, ...] = field(default_factory=tuple)
    browser: SensorResult | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "review": self.review.to_dict(),
            "sensors": [item.to_dict() for item in self.sensors],
            "browser": self.browser.to_dict() if self.browser else None,
        }
