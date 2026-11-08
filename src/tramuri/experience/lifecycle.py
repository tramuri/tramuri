"""Persist the Product Experience lifecycle as idempotent run events."""

from __future__ import annotations

import hashlib
import json
from typing import Any

from tramuri.storage.database import Database


class ExperienceLifecycle:
    steps = (
        "product_definition",
        "product_review",
        "demo_generation",
        "browser_validation",
        "critical_review",
        "correction",
        "delivery",
    )

    def __init__(self, database: Database, run_id: str):
        self.database = database
        self.run_id = run_id

    def record(self, step: str, payload: dict[str, Any]) -> None:
        fingerprint = hashlib.sha256(
            json.dumps(payload, ensure_ascii=False, sort_keys=True).encode("utf-8")
        ).hexdigest()[:16]
        self.database.event(
            self.run_id,
            f"experience.{step}",
            payload,
            idempotency_key=f"{self.run_id}:experience:{step}:{fingerprint}",
        )

    def status(self) -> dict[str, dict[str, Any]]:
        """Return the latest persisted event for each experience step."""
        details = self.database.run_details(self.run_id)
        result: dict[str, dict[str, Any]] = {
            step: {"step": step, "status": "pending"} for step in self.steps
        }
        for event in details.get("events", []):
            event_type = str(event.get("event_type", ""))
            prefix = "experience."
            if not event_type.startswith(prefix):
                continue
            step = event_type.removeprefix(prefix)
            if step in result:
                result[step] = {
                    "step": step,
                    "status": "recorded",
                    "payload": event.get("payload", {}),
                    "sequence": event.get("sequence"),
                }
        return result
