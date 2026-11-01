"""Python SDK for Tramuri's shared Rust core."""

from __future__ import annotations

from importlib.metadata import version
import json
import os
from typing import Any

from ._native import (
    TramuriError,
    _evidence_get_json,
    _evidence_list_json,
    _init_json,
    _evidence_add_json,
    _execution_get_json,
    _execution_list_json,
    _execution_record_json,
    _knowledge_add_json,
    _knowledge_list_json,
    _knowledge_withdraw_json,
    _mission_contract_revise_json,
    _mission_close_json,
    _mission_create_json,
    _mission_get_json,
    _mission_list_json,
    _mission_knowledge_link_json,
    _mission_knowledge_unlink_json,
    _mission_capability_link_json,
    _mission_capability_unlink_json,
    _mission_relationships_json,
    _mission_resume_json,
    _outcome_assess_json,
    _outcome_get_json,
    _scan_json,
    _capability_snapshot_json,
    _status_json,
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


def capabilities(path: PathLike = ".", *, type: str | None = None) -> dict[str, Any]:
    """Return the latest declarative capability snapshot created by ``scan``.

    ``type`` may be ``agent``, ``skill``, ``rule``, ``hook``, or ``mcp_server``.
    This reads declarations only; it never executes or connects to a capability.
    """
    encoded_path = os.fsdecode(os.fspath(path))
    normalized = type.replace("-", "_") if type is not None else None
    return json.loads(_capability_snapshot_json(encoded_path, normalized))


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


def mission_list(path: PathLike = ".") -> list[dict[str, Any]]:
    """List Missions without requiring their IDs in advance."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_mission_list_json(encoded_path))


def mission_knowledge_link(
    path: PathLike = ".", *, mission_id: str, knowledge_id: str,
    actor_kind: str = "human", actor_reference: str | None = None,
) -> dict[str, Any]:
    """Explicitly link a Knowledge record to a Mission, recording caller-supplied attribution."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(
        _mission_knowledge_link_json(
            encoded_path, mission_id, knowledge_id, actor_kind, actor_reference
        )
    )


def mission_knowledge_unlink(
    path: PathLike = ".", *, mission_id: str, knowledge_id: str
) -> bool:
    """Remove a Mission–Knowledge link without deleting the Knowledge record."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_mission_knowledge_unlink_json(encoded_path, mission_id, knowledge_id))


def mission_capability_link(
    path: PathLike = ".", *, mission_id: str, capability_id: str,
    actor_kind: str = "human", actor_reference: str | None = None,
) -> dict[str, Any]:
    """Explicitly link a scanned Capability; its observed source hash is retained."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(
        _mission_capability_link_json(
            encoded_path, mission_id, capability_id, actor_kind, actor_reference
        )
    )


def mission_capability_unlink(
    path: PathLike = ".", *, mission_id: str, capability_id: str
) -> bool:
    """Remove a Mission–Capability link without deleting the discovered declaration."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_mission_capability_unlink_json(encoded_path, mission_id, capability_id))


def mission_relationships(path: PathLike = ".", *, mission_id: str) -> dict[str, Any]:
    """Read only the Knowledge and Capabilities explicitly associated with a Mission."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_mission_relationships_json(encoded_path, mission_id))


def mission_resume(path: PathLike = ".", *, mission_id: str) -> dict[str, Any]:
    """Resume a Mission with its recorded activity and explicitly linked context."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_mission_resume_json(encoded_path, mission_id))


def status(path: PathLike = ".") -> dict[str, Any]:
    """Return a privacy-minimized, versioned workspace status snapshot."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_status_json(encoded_path))


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


def execution_list(path: PathLike = ".", *, mission_id: str) -> list[dict[str, Any]]:
    """List a Mission's executions, newest first, through the shared Rust core.

    Opening older local state may apply its supported schema migration before the read.
    Execution, evidence, and outcome records are not changed by this query.
    """
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_execution_list_json(encoded_path, mission_id))


def execution_get(
    path: PathLike = ".", *, execution_id: str
) -> dict[str, Any]:
    """Retrieve one execution by ID from the selected local Tramuri database.

    Opening older local state may apply its supported schema migration before the read.
    Execution, evidence, and outcome records are not changed by this query.
    """
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_execution_get_json(encoded_path, execution_id))


def evidence_add(
    path: PathLike = ".", *, mission_id: str, execution_id: str, kind: str, locator: str,
) -> dict[str, Any]:
    """Attach a reference without fetching external resources or storing file contents."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_evidence_add_json(encoded_path, mission_id, execution_id, kind, locator))


def evidence_list(path: PathLike = ".", *, mission_id: str) -> list[dict[str, Any]]:
    """List stored evidence references for a Mission without following their locators.

    Opening older local state may apply its supported schema migration before the read.
    Evidence records are not changed, followed, or revalidated by this query.
    """
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_evidence_list_json(encoded_path, mission_id))


def evidence_get(path: PathLike = ".", *, evidence_id: str) -> dict[str, Any]:
    """Retrieve stored evidence by ID without following or revalidating its locator.

    Opening older local state may apply its supported schema migration before the read.
    Evidence records are not changed, followed, or revalidated by this query.
    """
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_evidence_get_json(encoded_path, evidence_id))


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
    "capabilities",
    "knowledge_add",
    "knowledge_list",
    "knowledge_withdraw",
    "mission_create",
    "mission_get",
    "mission_list",
    "mission_knowledge_link",
    "mission_knowledge_unlink",
    "mission_capability_link",
    "mission_capability_unlink",
    "mission_relationships",
    "mission_resume",
    "status",
    "mission_close",
    "mission_contract_revise",
    "execution_record",
    "execution_list",
    "execution_get",
    "evidence_add",
    "evidence_list",
    "evidence_get",
    "outcome_assess",
    "outcome_get",
]
