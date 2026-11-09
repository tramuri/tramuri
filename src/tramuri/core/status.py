"""Read-only status projection for terminal, JSON and provider handoff."""

from __future__ import annotations

from typing import Any


def _stage_label(stage: dict[str, Any]) -> str:
    return str(stage.get("stage_key", "unknown")).replace("_", " ").title()


def build_status(
    details: dict[str, Any] | None,
    *,
    checkpoint: dict[str, Any] | None = None,
    knowledge: list[dict[str, Any]] | tuple[dict[str, Any], ...] = (),
    sensors: list[dict[str, Any]] | tuple[dict[str, Any], ...] = (),
) -> dict[str, Any]:
    """Project persisted facts into a compact actionable status object."""

    if details is None:
        return {
            "schema_version": "tramuri.status.v1",
            "semantic_status": "ready",
            "run_id": None,
            "project": {"state": "ready"},
            "progress": {"completed": 0, "total": 0, "percent": 0},
            "current_stage": None,
            "last_completed_stage": None,
            "next_action": "Run tramuri product demo --mode mvp or tramuri product run",
            "gates": {"blocking_failures": 0, "pending_validation": 0},
            "counts": {"completed_stages": 0, "artifacts": 0, "knowledge": 0, "sensors": 0},
            "research": {"status": "not_started", "source_count": 0},
            "experience": {"surfaces": [], "browser": "not_run", "limitations": []},
            "stages": [],
        }

    run = details.get("run", {})
    stages = [dict(item) for item in details.get("stages", [])]
    completed = [item for item in stages if item.get("status") == "completed"]
    active = next(
        (item for item in stages if item.get("status") in {"running", "blocked", "failed"}),
        next((item for item in stages if item.get("status") == "pending"), None),
    )
    stage_sensors = list(sensors) or list(details.get("sensor_results", []))
    blocking_failures = sum(
        1
        for item in stage_sensors
        if item.get("severity") == "blocking" and item.get("status") in {"failed", "blocked"}
    )
    knowledge_items = list(knowledge) or list(details.get("knowledge", []))
    pending_validation = sum(
        1 for item in knowledge_items if item.get("classification") == "pending_validation"
    )
    stale_artifacts = sum(
        1 for item in details.get("artifacts", []) if item.get("status") == "stale"
    )
    research_artifacts = [
        item
        for item in details.get("artifacts", [])
        if item.get("kind") == "pm.market_research" and item.get("status") != "superseded"
    ]
    research_stage = next((item for item in stages if item.get("stage_key") == "research"), None)
    delegated: dict[str, dict[str, Any]] = {}
    for event in details.get("events", []):
        if event.get("event_type") != "stage.delegated":
            continue
        payload = event.get("payload", {})
        if isinstance(payload, dict) and payload.get("stage"):
            delegated[str(payload["stage"])] = payload
    if research_artifacts:
        research_payload = research_artifacts[-1].get("payload", {})
        research_view = {
            "status": str(research_payload.get("status", "imported")),
            "source_count": int(research_payload.get("source_count", 0)),
        }
    elif research_stage and research_stage.get("status") == "skipped":
        research_view = {"status": "not_run", "source_count": 0}
    elif research_stage and research_stage.get("status") == "blocked":
        research_view = {"status": "required", "source_count": 0}
    else:
        research_view = {"status": "not_started", "source_count": 0}
    navigation_artifacts = [
        item
        for item in details.get("artifacts", [])
        if item.get("kind") == "pm.navigation" and item.get("status") != "superseded"
    ]
    lifecycle: dict[str, dict[str, Any]] = {}
    for event in details.get("events", []):
        event_type = str(event.get("event_type", ""))
        if event_type.startswith("experience."):
            lifecycle[event_type.removeprefix("experience.")] = dict(event.get("payload", {}))
    experience_view = {
        "surfaces": list(
            (navigation_artifacts[-1].get("payload", {}) if navigation_artifacts else {}).get(
                "surfaces", []
            )
        ),
        "browser": str(lifecycle.get("browser_validation", {}).get("status", "not_run")),
        "limitations": [
            "research:not_run" if research_view["status"] in {"not_run", "required"} else None,
            "browser:not_run"
            if str(lifecycle.get("browser_validation", {}).get("status", "not_run")) == "not_run"
            else None,
        ],
    }
    experience_view["limitations"] = [
        item for item in experience_view["limitations"] if item is not None
    ]
    raw_status = str(run.get("status", "unknown"))
    if blocking_failures:
        semantic = "blocked"
    elif raw_status == "failed":
        semantic = "failed"
    elif raw_status == "completed" and stale_artifacts:
        semantic = "stale"
    elif raw_status == "completed":
        semantic = "completed"
    elif pending_validation and not active:
        semantic = "waiting"
    else:
        semantic = raw_status
    total = len(stages)
    completed_count = len(completed)
    next_action = (
        str(checkpoint.get("next_action")) if checkpoint and checkpoint.get("next_action") else None
    )
    if not next_action:
        if semantic == "blocked":
            next_action = "Repair the blocking gate, then run tramuri resume"
        elif active:
            next_action = f"Continue {_stage_label(active)}"
        elif semantic == "completed":
            next_action = "Review the report and product demo"
        else:
            next_action = "Inspect tramuri runs show <run-id>"
    return {
        "schema_version": "tramuri.status.v1",
        "semantic_status": semantic,
        "run_id": run.get("id"),
        "project": {
            "mode": run.get("mode"),
            "provider": run.get("provider"),
            "updated_at": run.get("updated_at"),
        },
        "progress": {
            "completed": completed_count,
            "total": total,
            "percent": round(completed_count / total * 100) if total else 0,
        },
        "current_stage": active.get("stage_key") if active else None,
        "last_completed_stage": completed[-1].get("stage_key") if completed else None,
        "next_action": next_action,
        "gates": {"blocking_failures": blocking_failures, "pending_validation": pending_validation},
        "counts": {
            "completed_stages": completed_count,
            "artifacts": len(details.get("artifacts", [])),
            "knowledge": len(knowledge_items),
            "sensors": len(stage_sensors),
            "stale_artifacts": stale_artifacts,
        },
        "research": research_view,
        "experience": experience_view,
        "agents": {
            "delegated": len(delegated),
            "execution_modes": sorted(
                {
                    str(item.get("execution_mode"))
                    for item in delegated.values()
                    if item.get("execution_mode")
                }
            ),
            "invocation_statuses": sorted(
                {
                    str(item.get("agent_status"))
                    for item in delegated.values()
                    if item.get("agent_status")
                }
            ),
            "limitations": sorted(
                {
                    str(item.get("limitation"))
                    for item in delegated.values()
                    if item.get("limitation")
                }
            ),
        },
        "stages": [
            {
                "key": item.get("stage_key"),
                "status": item.get("status"),
                "attempts": item.get("attempts", 0),
                "position": item.get("position", index),
                "owner": item.get("owner"),
                "agent_id": delegated.get(str(item.get("stage_key")), {}).get("agent_id"),
                "execution_mode": delegated.get(str(item.get("stage_key")), {}).get(
                    "execution_mode"
                ),
            }
            for index, item in enumerate(stages)
        ],
    }
