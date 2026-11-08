"""Deterministic explanations backed by persisted Tramuri evidence."""

from __future__ import annotations

import json
import re
from typing import Any

SCHEMA_VERSION = "tramuri.explain.v1"
_ALIASES = {
    "proposta de valor": ("value proposition", "value_proposition"),
    "valor": ("value proposition", "value_proposition"),
    "usuário": ("target users", "target_users"),
    "usuario": ("target users", "target_users"),
    "problema": ("problem",),
    "escopo": ("scope", "in_scope", "out_of_scope", "shaping"),
    "hipótese": ("hypothesis", "hypotheses"),
    "hipotese": ("hypothesis", "hypotheses"),
    "métrica": ("metric", "metrics", "success"),
    "metrica": ("metric", "metrics", "success"),
}


def _haystack(item: dict[str, Any]) -> str:
    return json.dumps(item, ensure_ascii=False, default=str).casefold()


def _query_terms(subject: str) -> tuple[str, ...]:
    query = subject.casefold().strip()
    aliases = _ALIASES.get(query, ())
    terms = (query, *aliases)
    return tuple(term for term in terms if term)


def _matches(item: dict[str, Any], subject: str) -> bool:
    if not subject.strip():
        return True
    haystack = _haystack(item)
    terms = _query_terms(subject)
    if any(term in haystack for term in terms):
        return True
    words = [word for word in re.split(r"\W+", subject.casefold()) if len(word) > 2]
    return bool(words) and all(word in haystack for word in words)


def _artifact_source(artifact: dict[str, Any]) -> dict[str, Any]:
    kind = str(artifact.get("kind", "unknown"))
    payload = artifact.get("payload", {})
    return {
        "artifact": kind,
        "stage": kind.removeprefix("pm."),
        "revision": artifact.get("revision", 1),
        "status": artifact.get("status", "unknown"),
        "classification": payload.get("classification", "unknown")
        if isinstance(payload, dict)
        else "unknown",
        "source": f"artifact:{kind}:v{artifact.get('revision', 1)}",
    }


def _connections(
    artifacts: list[dict[str, Any]], matched: list[dict[str, Any]]
) -> list[dict[str, str]]:
    by_stage = {
        str(item.get("kind", "")).removeprefix("pm."): str(item.get("kind", ""))
        for item in artifacts
    }
    matched_kinds = {str(item.get("kind", "")) for item in matched}
    connections: list[dict[str, str]] = []
    seen: set[tuple[str, str, str]] = set()
    for artifact in matched:
        kind = str(artifact.get("kind", "unknown"))
        dependencies = artifact.get("dependencies", [])
        if not isinstance(dependencies, list):
            dependencies = []
        for dependency in dependencies:
            target = by_stage.get(str(dependency), f"pm.{dependency}")
            key = (kind, target, "depends_on")
            if target and key not in seen:
                seen.add(key)
                connections.append(
                    {
                        "from": kind,
                        "to": target,
                        "relation": "depends_on",
                        "evidence": "stored dependency",
                    }
                )
    for artifact in artifacts:
        kind = str(artifact.get("kind", "unknown"))
        if kind in matched_kinds:
            continue
        dependencies = artifact.get("dependencies", [])
        if not isinstance(dependencies, list):
            dependencies = []
        for dependency in dependencies:
            target = by_stage.get(str(dependency), f"pm.{dependency}")
            if target in matched_kinds:
                key = (kind, target, "depends_on")
                if key not in seen:
                    seen.add(key)
                    connections.append(
                        {
                            "from": kind,
                            "to": target,
                            "relation": "depends_on",
                            "evidence": "stored dependency",
                        }
                    )
    return connections


def _matching_records(records: list[dict[str, Any]], subject: str) -> list[dict[str, Any]]:
    return [item for item in records if _matches(item, subject)] if subject.strip() else []


def explain_run(details: dict[str, Any], subject: str | None = None) -> dict[str, Any]:
    """Explain only facts and relationships that are persisted for one run."""
    requested = (subject or "").strip()
    artifacts = [dict(item) for item in details.get("artifacts", [])]
    matched = [item for item in artifacts if _matches(item, requested)]
    run = details.get("run", {})
    unknowns: list[str] = []
    if not matched:
        unknowns.append(
            f"No persisted Tramuri evidence was found for '{requested or 'the current product'}'."
        )
    else:
        classifications = {
            str(item.get("payload", {}).get("classification", "unknown"))
            for item in matched
            if isinstance(item.get("payload"), dict)
        }
        if classifications & {"hypothesis", "assumption", "pending_validation"}:
            unknowns.append("Part of this explanation is an assumption or pending validation.")
        if not _matching_records(list(details.get("evidence", [])), requested):
            unknowns.append("No external evidence is attached to the matched records.")

    sources = [_artifact_source(item) for item in matched]
    evidence = _matching_records(list(details.get("evidence", [])), requested)
    decisions = _matching_records(list(details.get("decisions", [])), requested)
    hypotheses = _matching_records(list(details.get("hypotheses", [])), requested)
    knowledge = _matching_records(list(details.get("knowledge", [])), requested)
    return {
        "schema_version": SCHEMA_VERSION,
        "found": bool(matched),
        "subject": requested or "current product",
        "run_id": run.get("id"),
        "run_status": run.get("status", "unknown"),
        "mode": run.get("mode"),
        "summary": (
            f"Persisted product records related to '{requested}'."
            if requested
            else "Persisted explanation of the current product run."
        ),
        "sources": sources,
        "connections": _connections(artifacts, matched),
        "evidence": [
            {
                "source": item.get("source_ref") or item.get("source_kind", "unknown"),
                "classification": item.get("classification", "unknown"),
            }
            for item in evidence
        ],
        "records": {
            "decisions": len(decisions),
            "hypotheses": len(hypotheses),
            "knowledge": len(knowledge),
        },
        "unknowns": unknowns,
    }
