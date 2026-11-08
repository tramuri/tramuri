"""Provider-neutral session and checkpoint facade."""

from __future__ import annotations

from typing import Any

from tramuri.storage.database import Database


class SessionStore:
    """Keep provider lifecycle details behind the storage contract."""

    def __init__(self, database: Database):
        self.database = database

    def record_event(
        self,
        provider: str,
        event_name: str,
        external_session_id: str,
        *,
        run_id: str | None = None,
        metadata: dict[str, Any] | None = None,
        idempotency_key: str | None = None,
    ) -> str:
        return self.database.record_session_event(
            provider,
            event_name,
            external_session_id,
            run_id=run_id,
            metadata=metadata,
            idempotency_key=idempotency_key,
        )

    def latest_checkpoint(self, run_id: str) -> dict[str, Any] | None:
        return self.database.latest_checkpoint(run_id)
