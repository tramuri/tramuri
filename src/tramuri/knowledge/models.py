"""Typed knowledge contracts kept separate from provider projections."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from tramuri.core.models import (
    KnowledgeClassification,
    KnowledgeFreshness,
    KnowledgeStatus,
    KnowledgeTier,
)


@dataclass(frozen=True, slots=True)
class KnowledgeEntry:
    title: str
    content: str
    classification: KnowledgeClassification
    source_kind: str
    tier: KnowledgeTier = KnowledgeTier.PROJECT
    source_ref: str | None = None
    source_hash: str | None = None
    freshness: KnowledgeFreshness = KnowledgeFreshness.UNKNOWN
    status: KnowledgeStatus = KnowledgeStatus.ACTIVE

    def to_dict(self) -> dict[str, Any]:
        return {
            "title": self.title,
            "content": self.content,
            "classification": self.classification.value,
            "source_kind": self.source_kind,
            "tier": self.tier.value,
            "source_ref": self.source_ref,
            "source_hash": self.source_hash,
            "freshness": self.freshness.value,
            "status": self.status.value,
        }
