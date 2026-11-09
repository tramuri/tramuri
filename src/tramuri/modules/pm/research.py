"""Source-backed market research packs for the local product workflow."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

from tramuri.core.errors import ValidationError
from tramuri.core.models import StageStatus
from tramuri.storage.database import Database

MAX_RESEARCH_BYTES = 2_000_000
RESEARCH_VERSION = "tramuri.research.v1"
CLAIM_COLLECTIONS = (
    "competitors",
    "market_signals",
    "buying_motivations",
    "build_buy_partner",
    "positioning",
)


@dataclass(frozen=True, slots=True)
class ResearchPack:
    payload: dict[str, Any]


def _require_text(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ValidationError(f"Research pack field '{field}' is required")
    return value.strip()


def _validate_source(source: Any, index: int) -> dict[str, Any]:
    if not isinstance(source, dict):
        raise ValidationError(f"Research source {index} must be an object")
    result = dict(source)
    _require_text(result.get("id"), f"sources[{index}].id")
    url = _require_text(result.get("url"), f"sources[{index}].url")
    parsed = urlparse(url)
    if parsed.scheme not in {"http", "https"} or not parsed.netloc:
        raise ValidationError(f"Research source URL is invalid: {url}")
    _require_text(result.get("title"), f"sources[{index}].title")
    _require_text(result.get("publisher"), f"sources[{index}].publisher")
    _require_text(result.get("accessed_at"), f"sources[{index}].accessed_at")
    confidence = result.get("confidence", "medium")
    if confidence not in {"low", "medium", "high"}:
        raise ValidationError("Research source confidence must be low, medium or high")
    return result


def _validate_claims(payload: dict[str, Any], source_ids: set[str]) -> None:
    for collection in CLAIM_COLLECTIONS:
        value = payload.get(collection, [])
        if not isinstance(value, list):
            raise ValidationError(f"Research field '{collection}' must be a list")
        for index, item in enumerate(value):
            if not isinstance(item, dict):
                raise ValidationError(f"Research claim {collection}[{index}] must be an object")
            references = item.get("source_ids")
            if not isinstance(references, list) or not references:
                raise ValidationError(f"Research claim {collection}[{index}] requires source_ids")
            unknown = [str(ref) for ref in references if str(ref) not in source_ids]
            if unknown:
                raise ValidationError(
                    f"Research claim {collection}[{index}] references unknown source(s): {', '.join(unknown)}"
                )
    sizing = payload.get("market_sizing", {})
    if not isinstance(sizing, dict):
        raise ValidationError("Research field 'market_sizing' must be an object")
    for index, claim in enumerate(sizing.get("claims", [])):
        if not isinstance(claim, dict) or not claim.get("source_ids"):
            raise ValidationError(f"Research market_sizing claim {index} requires source_ids")
        unknown = [str(ref) for ref in claim["source_ids"] if str(ref) not in source_ids]
        if unknown:
            raise ValidationError(
                f"Research market_sizing claim {index} references unknown source(s): {', '.join(unknown)}"
            )


def load_research_pack(path: Path) -> ResearchPack:
    """Load and validate a provider-produced JSON pack before persistence."""
    try:
        if path.stat().st_size > MAX_RESEARCH_BYTES:
            raise ValidationError("Research pack exceeds the safe 2 MB limit")
        payload = json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise ValidationError(f"Could not read research pack: {path}") from exc
    except json.JSONDecodeError as exc:
        raise ValidationError(f"Research pack is not valid JSON: {exc.msg}") from exc
    if not isinstance(payload, dict):
        raise ValidationError("Research pack must be a JSON object")
    if payload.get("version") != RESEARCH_VERSION:
        raise ValidationError(f"Research pack version must be {RESEARCH_VERSION}")
    _require_text(payload.get("researched_at"), "researched_at")
    _require_text(payload.get("query"), "query")
    sources = payload.get("sources")
    if not isinstance(sources, list) or not sources:
        raise ValidationError("Research pack requires at least one source")
    normalized_sources = [_validate_source(source, index) for index, source in enumerate(sources)]
    source_ids = {str(source["id"]) for source in normalized_sources}
    if len(source_ids) != len(normalized_sources):
        raise ValidationError("Research source ids must be unique")
    _validate_claims(payload, source_ids)
    normalized = dict(payload)
    normalized["sources"] = normalized_sources
    normalized["status"] = "imported"
    normalized["source_count"] = len(normalized_sources)
    return ResearchPack(normalized)


def not_run_payload(reason: str = "no-research-pack") -> dict[str, Any]:
    return {
        "version": RESEARCH_VERSION,
        "status": "not_run",
        "reason": reason,
        "source_count": 0,
        "competitors": [],
        "market_signals": [],
        "buying_motivations": [],
        "build_buy_partner": [],
        "positioning": [],
        "market_sizing": {"status": "pending_validation", "claims": []},
        "unknowns": ["No provider-backed research pack was supplied."],
        "classification": "pending_validation",
    }


def import_research_pack(database: Database, run_id: str, path: Path) -> str:
    """Persist a validated pack and its source-backed claims as evidence."""
    pack = load_research_pack(path)
    details = database.run_details(run_id)
    stage = next((item for item in details["stages"] if item["stage_key"] == "research"), None)
    if stage is None:
        raise ValidationError("This run mode does not include the research stage")
    if stage["status"] not in {StageStatus.RUNNING.value, StageStatus.COMPLETED.value}:
        database.transition_stage(run_id, "research", StageStatus.RUNNING)
    artifact_id = database.add_artifact(
        run_id,
        "research",
        "pm.market_research",
        pack.payload,
        ["discovery"],
        idempotency_key=f"{run_id}:research:import:{pack.payload['source_count']}:{pack.payload['researched_at']}",
    )
    for collection in CLAIM_COLLECTIONS:
        for item in pack.payload.get(collection, []):
            statement = str(
                item.get("statement")
                or item.get("signal")
                or item.get("motivation")
                or item.get("positioning")
                or item.get("option")
                or item.get("name")
                or "Research claim"
            )
            database.add_record(
                "evidence",
                {
                    "run_id": run_id,
                    "claim": statement,
                    "source_kind": "market_research",
                    "source_ref": ",".join(str(item_id) for item_id in item["source_ids"]),
                    "classification": "evidence",
                    "freshness": str(pack.payload["researched_at"]),
                    "verified_by": "provider-research-pack",
                },
            )
    database.event(
        run_id,
        "research.imported",
        {"artifact_id": artifact_id, "source_count": pack.payload["source_count"]},
        idempotency_key=f"{run_id}:research:imported:{artifact_id}",
    )
    database.validation(
        run_id,
        str(stage["id"]),
        artifact_id,
        "pm.market_research.integrity",
        "passed",
        "Research claims reference validated source records.",
        idempotency_key=f"{run_id}:research:imported:validation:{artifact_id}",
    )
    if stage["status"] != StageStatus.COMPLETED.value:
        database.transition_stage(run_id, "research", StageStatus.COMPLETED)
    return artifact_id
