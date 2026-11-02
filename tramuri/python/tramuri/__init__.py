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
    _execution_transition_json,
    _knowledge_add_json,
    _knowledge_list_json,
    _knowledge_withdraw_json,
    _mission_contract_revise_json,
    _mission_close_json,
    _mission_create_json,
    _mission_start_json,
    _define_start_json,
    _define_status_json,
    _define_next_json,
    _define_submit_json,
    _define_review_json,
    _define_history_json,
    _define_brief_assess_json,
    _define_brief_assessment_get_json,
    _define_question_request_json,
    _define_question_answer_json,
    _profile_get_json,
    _profile_set_json,
    _mission_workflow_get_json,
    _mission_workflow_set_json,
    _mission_workflow_activity_set_json,
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
    _readiness_assess_json,
    _readiness_get_json,
    _readiness_history_json,
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
    criteria: list[dict[str, Any]] | None = None,
    context: str | None = None,
) -> dict[str, Any]:
    """Create a Mission; omit criteria to begin discovery with an exploratory contract."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(
        _mission_create_json(encoded_path, objective, context, json.dumps(criteria or []))
    )


def mission_start(
    path: PathLike = ".", *, objective: str, perspective: str | None = None,
    context: str | None = None,
) -> dict[str, Any]:
    """Start an exploratory Mission; new Missions require an explicit perspective.

    An exact, unique open Mission match is resumed with its recorded perspective. Otherwise pass
    ``product_engineer``, ``fde``, or ``mixed``; intent inference and saved defaults never select
    the Mission perspective on the caller's behalf.
    """
    return json.loads(_mission_start_json(os.fsdecode(os.fspath(path)), objective, perspective, context))


def mission_define_start(path: PathLike = ".", *, mission_id: str) -> dict[str, Any]:
    """Start the fixed five-step Product Engineering DEFINE workflow for a Mission."""
    return json.loads(_define_start_json(os.fsdecode(os.fspath(path)), mission_id))


def mission_define_status(path: PathLike = ".", *, mission_id: str) -> dict[str, Any]:
    """Read persisted DEFINE progress without advancing its current step."""
    return json.loads(_define_status_json(os.fsdecode(os.fspath(path)), mission_id))


def mission_define_next(path: PathLike = ".", *, mission_id: str) -> dict[str, Any]:
    """Return the currently authorized DEFINE step contract and accepted inputs."""
    return json.loads(_define_next_json(os.fsdecode(os.fspath(path)), mission_id))


def mission_define_submit(
    path: PathLike = ".", *, mission_id: str, step: str,
    payload: dict[str, Any], expected_revision: int | None = None,
    host_reported_skill_id: str | None = None,
) -> dict[str, Any]:
    """Submit a structured step result; the Rust Core validates and controls progression."""
    return json.loads(_define_submit_json(
        os.fsdecode(os.fspath(path)), mission_id, step, json.dumps(payload),
        expected_revision, host_reported_skill_id,
    ))


def mission_define_review(
    path: PathLike = ".", *, mission_id: str, step: str,
    payload: dict[str, Any], reason: str, declared_author: str,
    origin: str = "user_request", expected_revision: int | None = None,
    authorize_decision_changes: bool = False,
    host_reported_skill_id: str | None = None,
) -> dict[str, Any]:
    """Review a completed DEFINE artifact, preserving its history and invalidating dependents."""
    return json.loads(_define_review_json(
        os.fsdecode(os.fspath(path)), mission_id, step, json.dumps(payload),
        reason, declared_author, origin, expected_revision,
        authorize_decision_changes, host_reported_skill_id,
    ))


def mission_define_history(
    path: PathLike = ".", *, mission_id: str, step: str | None = None,
) -> list[dict[str, Any]]:
    """Return all accepted DEFINE artifact revisions, including stale history."""
    return json.loads(_define_history_json(
        os.fsdecode(os.fspath(path)), mission_id, step,
    ))


def mission_define_assess_brief(
    path: PathLike = ".", *, mission_id: str, assessment: dict[str, Any],
) -> dict[str, Any]:
    """Persist a versioned host semantic review; the Core validates shape, not truth."""
    return json.loads(_define_brief_assess_json(
        os.fsdecode(os.fspath(path)), mission_id, json.dumps(assessment),
    ))


def mission_define_brief_assessment(
    path: PathLike = ".", *, mission_id: str,
) -> dict[str, Any] | None:
    """Read the latest host-produced semantic review without treating it as Core verification."""
    return json.loads(_define_brief_assessment_get_json(os.fsdecode(os.fspath(path)), mission_id))


def mission_define_question_request(
    path: PathLike = ".", *, mission_id: str, step: str, rationale: str,
    alternatives: list[str] | tuple[str, ...] = (), material_decision: bool = False,
) -> dict[str, Any]:
    """Record one explicit clarification request against the persisted two-question budget."""
    return json.loads(_define_question_request_json(
        os.fsdecode(os.fspath(path)), mission_id, step, rationale,
        list(alternatives), material_decision,
    ))


def mission_define_question_answer(
    path: PathLike = ".", *, mission_id: str, question_id: str,
    answer: str, explicit_user_answer: bool = True,
) -> dict[str, Any]:
    """Persist a declared answer; the flag records its source, not authenticated identity."""
    return json.loads(_define_question_answer_json(
        os.fsdecode(os.fspath(path)), mission_id, question_id, answer,
        explicit_user_answer,
    ))


def profile_get(path: PathLike = ".") -> dict[str, Any]:
    """Read local Work Profile defaults without authenticating a person."""
    return json.loads(_profile_get_json(os.fsdecode(os.fspath(path))))


def profile_set(
    path: PathLike = ".", *, perspective: str = "auto", ask: str = "ambiguous"
) -> dict[str, Any]:
    """Set the default work perspective and ambiguity prompt policy."""
    return json.loads(_profile_set_json(os.fsdecode(os.fspath(path)), perspective, ask))


def mission_workflow_get(path: PathLike = ".", *, mission_id: str) -> dict[str, Any] | None:
    """Get Mission perspective and planned next activity, if recorded."""
    return json.loads(_mission_workflow_get_json(os.fsdecode(os.fspath(path)), mission_id))


def mission_workflow_set(
    path: PathLike = ".", *, mission_id: str, perspective: str,
    source: str = "explicit", next_activity: str | None = None,
) -> dict[str, Any]:
    """Update explicit workflow metadata without assessing Context Readiness."""
    return json.loads(_mission_workflow_set_json(os.fsdecode(os.fspath(path)), mission_id, perspective, source, next_activity))


def mission_workflow_activity_set(
    path: PathLike = ".", *, mission_id: str, next_activity: str,
) -> dict[str, Any]:
    """Update a Mission's current focus without changing its work perspective or provenance."""
    return json.loads(
        _mission_workflow_activity_set_json(
            os.fsdecode(os.fspath(path)), mission_id, next_activity,
        )
    )


def readiness_assess(
    path: PathLike = ".",
    *,
    mission_id: str,
    activity: str,
    state: str,
    criteria_applied: list[str] | tuple[str, ...] = (),
    gaps: list[str] | tuple[str, ...] = (),
    blockers: list[str] | tuple[str, ...] = (),
    accepted_uncertainties: list[str] | tuple[str, ...] = (),
    actor_reference: str | None = None,
) -> dict[str, Any]:
    """Record consultative context readiness for an explicitly selected activity."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(
        _readiness_assess_json(
            encoded_path,
            json.dumps(
                {
                    "mission_id": mission_id,
                    "activity": activity,
                    "decision": state,
                    "criteria_applied": list(criteria_applied),
                    "gaps": list(gaps),
                    "blockers": list(blockers),
                    "accepted_uncertainties": list(accepted_uncertainties),
                    "actor_reference": actor_reference,
                }
            ),
        )
    )


def readiness_get(path: PathLike = ".", *, mission_id: str) -> dict[str, Any]:
    """Return the current activity readiness, including stale detection."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_readiness_get_json(encoded_path, mission_id))


def readiness_history(path: PathLike = ".", *, mission_id: str) -> list[dict[str, Any]]:
    """Return append-only readiness assessments, newest first."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(_readiness_history_json(encoded_path, mission_id))


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
    work_perspective: str | None = None,
) -> dict[str, Any]:
    """Record an attempt and optional work perspective; completion does not verify the outcome."""
    encoded_path = os.fsdecode(os.fspath(path))
    return json.loads(
        _execution_record_json(
            encoded_path, mission_id, contract_revision, actor_kind, state, actor_reference, notes,
            work_perspective,
        )
    )


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


def execution_transition(
    path: PathLike = ".", *, execution_id: str, state: str
) -> dict[str, Any]:
    """Explicitly finish an in-progress Execution; terminal records cannot be rewritten."""
    return json.loads(_execution_transition_json(os.fsdecode(os.fspath(path)), execution_id, state))


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
    "mission_start",
    "profile_get",
    "profile_set",
    "mission_workflow_get",
    "mission_workflow_set",
    "mission_workflow_activity_set",
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
    "execution_transition",
    "execution_list",
    "execution_get",
    "evidence_add",
    "evidence_list",
    "evidence_get",
    "outcome_assess",
    "outcome_get",
    "readiness_assess",
    "readiness_get",
    "readiness_history",
]
