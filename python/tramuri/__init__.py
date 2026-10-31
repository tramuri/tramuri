"""Python SDK for Tramuri's shared Rust core."""

from __future__ import annotations

from importlib.metadata import version
import json
import os
from typing import Any

from ._native import (
    TramuriError,
    _init_json,
    _evidence_add_json,
    _execution_record_json,
    _knowledge_add_json,
    _knowledge_list_json,
    _knowledge_withdraw_json,
    _mission_contract_revise_json,
    _mission_close_json,
    _mission_create_json,
    _mission_get_json,
    _outcome_assess_json,
    _outcome_get_json,
    _scan_json,
)

PathLike = str | os.PathLike[str]


def init(path: PathLike = ".") -> dict[str, Any]:
    """Initialize Tramuri's local database in *path* and return its report."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_init_json(encoded_path))


def scan(path: PathLike = ".") -> dict[str, Any]:
    """Inventory *path* using the Rust core and return a normalized report."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_scan_json(encoded_path))


def knowledge_add(
    path: PathLike = ".",
    *,
    type: str,
    statement: str,
    sources: tuple[str, ...] | list[str] = (),
    human_input: bool = False,
    supersedes: str | None = None,
) -> dict[str, Any]:
    """Store source-backed knowledge through the shared Rust core."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(
        _knowledge_add_json(encoded_path, type, statement, list(sources), human_input, supersedes)
    )


def knowledge_list(
    path: PathLike = ".", *, type: str | None = None, include_history: bool = False
) -> list[dict[str, Any]]:
    """List project knowledge, optionally including superseded history."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_knowledge_list_json(encoded_path, type, include_history))


def knowledge_withdraw(path: PathLike, id: str) -> dict[str, Any]:
    """Withdraw a current knowledge record while retaining its history."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_knowledge_withdraw_json(encoded_path, id))


def mission_create(
    path: PathLike = ".",
    *,
    objective: str,
    criteria: list[dict[str, Any]],
    context: str | None = None,
) -> dict[str, Any]:
    """Create a Mission and its first Success Contract through the Rust core."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(
        _mission_create_json(encoded_path, objective, context, json.dumps(criteria))
    )


def mission_get(path: PathLike = ".", *, mission_id: str) -> dict[str, Any]:
    """Retrieve a Mission with its latest Success Contract."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_mission_get_json(encoded_path, mission_id))


def mission_close(path: PathLike = ".", *, mission_id: str) -> dict[str, Any]:
    """Close a Mission without marking its outcome achieved or verified."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_mission_close_json(encoded_path, mission_id))


def mission_contract_revise(
    path: PathLike = ".",
    *,
    mission_id: str,
    criteria: list[dict[str, Any]],
) -> dict[str, Any]:
    """Create a new Success Contract revision without changing previous revisions."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(
        _mission_contract_revise_json(encoded_path, mission_id, json.dumps(criteria))
    )


def execution_record(
    path: PathLike = ".", *, mission_id: str, contract_revision: int,
    actor_kind: str, state: str, actor_reference: str | None = None, notes: str | None = None,
) -> dict[str, Any]:
    """Record an attempt; completion does not verify the Mission outcome."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_execution_record_json(encoded_path, mission_id, contract_revision, actor_kind, state, actor_reference, notes))


def evidence_add(
    path: PathLike = ".", *, mission_id: str, execution_id: str, kind: str, locator: str,
) -> dict[str, Any]:
    """Attach a reference without fetching external resources or storing file contents."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_evidence_add_json(encoded_path, mission_id, execution_id, kind, locator))


def outcome_assess(
    path: PathLike = ".",
    *,
    mission_id: str,
    contract_revision: int,
    criterion_id: str,
    result: str,
    verifier_kind: str,
    verifier_reference: str,
    evidence_ids: tuple[str, ...] | list[str] = (),
    explanation: str | None = None,
) -> dict[str, Any]:
    """Append a criterion assessment through the shared Rust core."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(
        _outcome_assess_json(
            encoded_path,
            json.dumps(
                {
                    "mission_id": mission_id,
                    "contract_revision": contract_revision,
                    "criterion_id": criterion_id,
                    "result": result,
                    "evidence_ids": list(evidence_ids),
                    "verifier_kind": verifier_kind,
                    "verifier_reference": verifier_reference,
                    "explanation": explanation,
                }
            ),
        )
    )


def outcome_get(
    path: PathLike = ".", *, mission_id: str, contract_revision: int | None = None
) -> dict[str, Any]:
    """Get criterion assessment history and the deterministic aggregate outcome."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_outcome_get_json(encoded_path, mission_id, contract_revision))


__version__ = version("tramuri")

__all__ = [
    "TramuriError",
    "__version__",
    "init",
    "scan",
    "knowledge_add",
    "knowledge_list",
    "knowledge_withdraw",
    "mission_create",
    "mission_get",
    "mission_close",
    "mission_contract_revise",
    "execution_record",
    "evidence_add",
    "outcome_assess",
    "outcome_get",
]
