"""Deterministic, presentation-only projections for generated reports."""

from __future__ import annotations

import json
import re
from typing import Any
from urllib.parse import parse_qsl, urlsplit

from tramuri.core.safety import redact

_NODE_WIDTH = 170
_NODE_HEIGHT = 66
_COLUMN_GAP = 16
_ROW_GAP = 30
_COLUMN_ORDER = ("stage", "artifact", "evidence", "hypothesis", "decision")
_SENSITIVE_KEY = re.compile(r"(?i)(?:api[_-]?key|password|passwd|token|secret|authorization)")
_BEARER = re.compile(r"(?i)\bbearer\s+[a-z0-9._~+/=-]+")


def safe_http_url(value: Any) -> str | None:
    """Return an absolute HTTP(S) URL suitable for an anchor, or ``None``."""

    if not isinstance(value, str) or not value or value != value.strip():
        return None
    if any(character.isspace() or ord(character) < 32 or character in "<>" for character in value):
        return None
    parsed = urlsplit(value)
    if (
        parsed.scheme.lower() not in {"http", "https"}
        or not parsed.hostname
        or parsed.username
        or parsed.password
    ):
        return None
    sensitive_query = {"api_key", "apikey", "token", "secret", "password", "authorization"}
    if any(
        key.casefold().replace("-", "_") in sensitive_query for key, _ in parse_qsl(parsed.query)
    ):
        return None
    return value


def redact_report_value(value: Any) -> Any:
    """Redact common secret values and sensitive-key fields before rendering."""

    if isinstance(value, dict):
        return {
            key: "[REDACTED]" if _SENSITIVE_KEY.fullmatch(str(key)) else redact_report_value(item)
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [redact_report_value(item) for item in value]
    if isinstance(value, tuple):
        return tuple(redact_report_value(item) for item in value)
    if isinstance(value, str):
        return _BEARER.sub("Bearer [REDACTED]", redact(value))
    return value


def _as_list(value: Any) -> list[Any]:
    if isinstance(value, list):
        return value
    if isinstance(value, tuple):
        return list(value)
    if isinstance(value, str):
        try:
            parsed = json.loads(value)
        except ValueError:
            return []
        return parsed if isinstance(parsed, list) else []
    return []


def _text(value: Any, fallback: str) -> str:
    return str(value).strip() if value is not None and str(value).strip() else fallback


def _short(value: str, length: int = 26) -> str:
    return value if len(value) <= length else value[: length - 1].rstrip() + "…"


def build_relationship_graph(details: dict[str, Any]) -> dict[str, Any]:
    """Project stored records into a graph without inferring semantic links.

    Edges come only from persisted stage dependencies, artifact stage/lineage
    references, or an exact evidence-to-decision ``source_ref`` match.
    """

    records: dict[str, list[dict[str, Any]]] = {
        "stage": [dict(item) for item in details.get("stages", [])],
        "artifact": [
            dict(item)
            for item in details.get("artifacts", [])
            if item.get("status") != "superseded"
        ],
        "evidence": [dict(item) for item in details.get("evidence", [])],
        "hypothesis": [dict(item) for item in details.get("hypotheses", [])],
        "decision": [dict(item) for item in details.get("decisions", [])],
    }
    stage_by_key = {
        str(item.get("stage_key")): item for item in records["stage"] if item.get("stage_key")
    }
    stage_by_id = {str(item.get("id")): item for item in records["stage"] if item.get("id")}
    stage_positions = {key: int(item.get("position", 0)) for key, item in stage_by_key.items()}
    x_positions = {
        kind: 26 + index * (_NODE_WIDTH + _COLUMN_GAP) for index, kind in enumerate(_COLUMN_ORDER)
    }
    nodes: list[dict[str, Any]] = []
    lookup: dict[tuple[str, str], dict[str, Any]] = {}

    for kind in _COLUMN_ORDER:

        def order(item: dict[str, Any], column_kind: str = kind) -> tuple[Any, ...]:
            if column_kind == "stage":
                return (int(item.get("position", 0)), str(item.get("stage_key", "")))
            if column_kind == "artifact":
                stage = stage_by_id.get(str(item.get("stage_id", "")))
                stage_key = str((stage or {}).get("stage_key") or item.get("stage_key") or "")
                return (
                    stage_positions.get(stage_key, len(stage_positions)),
                    int(item.get("revision", 0)),
                    str(item.get("kind", "")),
                    str(item.get("id", "")),
                )
            return (str(item.get("created_at", "")), str(item.get("id", "")))

        ordered = sorted(records[kind], key=order)
        for row_index, item in enumerate(ordered):
            record_id = str(item.get("id") or item.get("stage_key") or row_index)
            if kind == "stage":
                record_id = str(item.get("stage_key") or record_id)
                label = _text(item.get("stage_key"), "Workflow stage").replace("_", " ").title()
                description = _text(item.get("owner"), "Tramuri")
                href = f"#stage-{record_id}"
            elif kind == "artifact":
                label = (
                    _text(item.get("kind"), "Artifact")
                    .removeprefix("pm.")
                    .replace("_", " ")
                    .title()
                )
                description = _text(item.get("status"), "recorded artifact")
                href = f"#artifact-{record_id}"
            elif kind == "evidence":
                label = _text(item.get("claim"), "Evidence record")
                description = _text(item.get("classification"), "evidence")
                href = f"#evidence-{record_id}"
            elif kind == "hypothesis":
                label = _text(item.get("statement"), "Hypothesis")
                description = _text(item.get("evidence_state"), "hypothesis")
                href = f"#hypothesis-{record_id}"
            else:
                label = _text(item.get("title"), "Decision")
                description = _text(item.get("classification"), "decision")
                href = f"#decision-{record_id}"
            node = {
                "id": f"{kind}:{record_id}",
                "record_id": record_id,
                "kind": kind,
                "label": label,
                "short_label": _short(label),
                "description": description,
                "href": href,
                "x": x_positions[kind],
                "y": 52 + row_index * (_NODE_HEIGHT + _ROW_GAP),
                "width": _NODE_WIDTH,
                "height": _NODE_HEIGHT,
            }
            nodes.append(node)
            lookup[(kind, record_id)] = node

    edges: list[dict[str, Any]] = []

    def add_edge(source: dict[str, Any] | None, target: dict[str, Any] | None, kind: str) -> None:
        if not source or not target:
            return
        if any(
            edge["source"] == source["id"]
            and edge["target"] == target["id"]
            and edge["kind"] == kind
            for edge in edges
        ):
            return
        source_center = source["y"] + source["height"] / 2
        target_center = target["y"] + target["height"] / 2
        if source["x"] == target["x"]:
            start_x = source["x"] + source["width"] / 2
            end_x = target["x"] + target["width"] / 2
            start_y = source["y"] + source["height"]
            end_y = target["y"]
            path = f"M {start_x} {start_y} C {start_x} {start_y + 20}, {end_x} {end_y - 20}, {end_x} {end_y}"
        else:
            start_x = source["x"] + source["width"]
            end_x = target["x"]
            path = (
                f"M {start_x} {source_center} C {start_x + 22} {source_center}, "
                f"{end_x - 22} {target_center}, {end_x} {target_center}"
            )
        edges.append(
            {
                "source": source["id"],
                "target": target["id"],
                "kind": kind,
                "path": path,
                "label": kind.replace("_", " "),
            }
        )

    for item in records["stage"]:
        target_key = str(item.get("stage_key") or "")
        for dependency in _as_list(item.get("dependencies", item.get("dependencies_json"))):
            source_stage = stage_by_key.get(str(dependency)) or stage_by_id.get(str(dependency))
            add_edge(
                lookup.get(("stage", str(source_stage.get("stage_key")))) if source_stage else None,
                lookup.get(("stage", target_key)),
                "stage_dependency",
            )

    for item in records["artifact"]:
        artifact_node = lookup.get(("artifact", str(item.get("id", ""))))
        stage = stage_by_id.get(str(item.get("stage_id", "")))
        if not stage:
            stage = stage_by_key.get(str(item.get("stage_key", "")))
        if stage:
            add_edge(
                lookup.get(("stage", str(stage.get("stage_key")))),
                artifact_node,
                "produced",
            )
        for dependency in _as_list(item.get("dependencies", item.get("dependencies_json"))):
            dependency_text = str(dependency)
            source_artifact = lookup.get(("artifact", dependency_text))
            if source_artifact:
                add_edge(source_artifact, artifact_node, "artifact_dependency")
                continue
            source_stage = stage_by_key.get(dependency_text) or stage_by_id.get(dependency_text)
            if source_stage:
                add_edge(
                    lookup.get(("stage", str(source_stage.get("stage_key")))),
                    artifact_node,
                    "stage_dependency",
                )

    evidence_by_id = {str(item.get("id")): item for item in records["evidence"] if item.get("id")}
    evidence_by_ref: dict[str, list[dict[str, Any]]] = {}
    for item in records["evidence"]:
        if item.get("source_ref"):
            evidence_by_ref.setdefault(str(item["source_ref"]), []).append(item)
    for decision in records["decision"]:
        reference = decision.get("source_ref")
        if not reference:
            continue
        candidates = []
        if str(reference) in evidence_by_id:
            candidates.append(evidence_by_id[str(reference)])
        candidates.extend(evidence_by_ref.get(str(reference), []))
        for evidence in candidates:
            add_edge(
                lookup.get(("evidence", str(evidence.get("id", "")))),
                lookup.get(("decision", str(decision.get("id", "")))),
                "source_reference",
            )

    nodes.sort(key=lambda item: (_COLUMN_ORDER.index(item["kind"]), item["y"]))
    edges.sort(key=lambda item: (item["kind"], item["source"], item["target"]))
    max_row = max((node["y"] for node in nodes), default=0)
    return {
        "nodes": nodes,
        "edges": edges,
        "width": x_positions[_COLUMN_ORDER[-1]] + _NODE_WIDTH + 26,
        "height": max(180, int(max_row + _NODE_HEIGHT + 28)),
        "columns": [
            {"kind": kind, "x": x_positions[kind], "label": kind.title()} for kind in _COLUMN_ORDER
        ],
    }
