"""SQLite persistence for runs, artifacts, events and provider manifests."""

from __future__ import annotations

import hashlib
import json
import sqlite3
from collections.abc import Iterator
from contextlib import contextmanager, suppress
from pathlib import Path
from typing import Any, cast

from tramuri.core.errors import StorageError, ValidationError
from tramuri.core.models import (
    ArtifactStatus,
    Mode,
    RunStatus,
    StageStatus,
    json_dumps,
    new_id,
    now_iso,
)
from tramuri.core.paths import output_root
from tramuri.core.workflow import stage_specs, validate_transition

SCHEMA_VERSION = 6


class Database:
    """Small repository around one project-local SQLite database."""

    def __init__(self, project_dir: Path):
        self.project_dir = project_dir.resolve()
        self.path = output_root(self.project_dir) / "database" / "tramuri.sqlite3"
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self._migrate()

    @contextmanager
    def connection(self) -> Iterator[sqlite3.Connection]:
        connection = sqlite3.connect(self.path, timeout=10)
        connection.row_factory = sqlite3.Row
        connection.execute("PRAGMA foreign_keys = ON")
        try:
            yield connection
            connection.commit()
        except sqlite3.Error as exc:
            connection.rollback()
            raise StorageError(f"SQLite operation failed: {exc}") from exc
        finally:
            connection.close()

    def _migrate(self) -> None:
        with self.connection() as db:
            current = db.execute("PRAGMA user_version").fetchone()[0]
            if current > SCHEMA_VERSION:
                raise StorageError(
                    f"Database schema {current} is newer than supported {SCHEMA_VERSION}"
                )
            if current == 0:
                db.executescript(
                    """
                    CREATE TABLE IF NOT EXISTS projects (
                        id TEXT PRIMARY KEY,
                        root_path TEXT NOT NULL UNIQUE,
                        metadata_json TEXT NOT NULL DEFAULT '{}',
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS runs (
                        id TEXT PRIMARY KEY,
                        project_id TEXT NOT NULL REFERENCES projects(id),
                        idea TEXT NOT NULL,
                        mode TEXT NOT NULL,
                        provider TEXT NOT NULL,
                        status TEXT NOT NULL,
                        attempt_limit INTEGER NOT NULL DEFAULT 3,
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL,
                        completed_at TEXT
                    );
                    CREATE TABLE IF NOT EXISTS stages (
                        id TEXT PRIMARY KEY,
                        run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                        stage_key TEXT NOT NULL,
                        position INTEGER NOT NULL,
                        owner TEXT NOT NULL,
                        dependencies_json TEXT NOT NULL DEFAULT '[]',
                        status TEXT NOT NULL,
                        attempts INTEGER NOT NULL DEFAULT 0,
                        started_at TEXT,
                        completed_at TEXT,
                        current_artifact_id TEXT,
                        error_message TEXT,
                        UNIQUE(run_id, stage_key)
                    );
                    CREATE TABLE IF NOT EXISTS artifacts (
                        id TEXT PRIMARY KEY,
                        run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                        stage_id TEXT REFERENCES stages(id),
                        kind TEXT NOT NULL,
                        revision INTEGER NOT NULL,
                        status TEXT NOT NULL,
                        payload_json TEXT NOT NULL,
                        dependencies_json TEXT NOT NULL DEFAULT '[]',
                        content_hash TEXT NOT NULL,
                        lineage_hash TEXT NOT NULL DEFAULT '',
                        created_at TEXT NOT NULL,
                        supersedes_id TEXT
                    );
                    CREATE TABLE IF NOT EXISTS decisions (
                        id TEXT PRIMARY KEY,
                        run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                        title TEXT NOT NULL,
                        decision_text TEXT NOT NULL,
                        classification TEXT NOT NULL,
                        rationale TEXT NOT NULL,
                        source_kind TEXT NOT NULL,
                        source_ref TEXT,
                        created_at TEXT NOT NULL,
                        supersedes_id TEXT
                    );
                    CREATE TABLE IF NOT EXISTS hypotheses (
                        id TEXT PRIMARY KEY,
                        run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                        statement TEXT NOT NULL,
                        risk_level TEXT NOT NULL,
                        evidence_state TEXT NOT NULL,
                        test_method TEXT NOT NULL,
                        success_signal TEXT NOT NULL,
                        created_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS evidence (
                        id TEXT PRIMARY KEY,
                        run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                        claim TEXT NOT NULL,
                        source_kind TEXT NOT NULL,
                        source_ref TEXT,
                        classification TEXT NOT NULL,
                        freshness TEXT NOT NULL,
                        verified_by TEXT,
                        created_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS events (
                        id TEXT PRIMARY KEY,
                        run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                        stage_id TEXT,
                        sequence INTEGER NOT NULL DEFAULT 0,
                        event_type TEXT NOT NULL,
                        payload_json TEXT NOT NULL,
                        created_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS validations (
                        id TEXT PRIMARY KEY,
                        run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                        stage_id TEXT,
                        artifact_id TEXT,
                        check_key TEXT NOT NULL,
                        result TEXT NOT NULL,
                        details TEXT NOT NULL,
                        checked_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS capabilities (
                        id TEXT PRIMARY KEY,
                        project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                        provider TEXT NOT NULL,
                        capability_key TEXT NOT NULL,
                        status TEXT NOT NULL,
                        source_path TEXT,
                        source_hash TEXT,
                        observed_version TEXT,
                        details TEXT NOT NULL,
                        observed_at TEXT NOT NULL,
                        UNIQUE(project_id, provider, capability_key)
                    );
                    CREATE TABLE IF NOT EXISTS installation_manifests (
                        id TEXT PRIMARY KEY,
                        project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                        provider TEXT NOT NULL,
                        tramuri_version TEXT NOT NULL,
                        manifest_path TEXT NOT NULL,
                        components_json TEXT NOT NULL,
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL,
                        UNIQUE(project_id, provider)
                    );
                    CREATE TABLE IF NOT EXISTS hook_events (
                        id TEXT PRIMARY KEY,
                        project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                        provider TEXT NOT NULL,
                        session_id TEXT,
                        event_name TEXT NOT NULL,
                        tool_name TEXT,
                        prompt_class TEXT,
                        decision TEXT NOT NULL,
                        payload_json TEXT NOT NULL,
                        created_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS event_idempotency (
                        idempotency_key TEXT PRIMARY KEY,
                        event_id TEXT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
                        created_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS sessions (
                        id TEXT PRIMARY KEY,
                        project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                        run_id TEXT REFERENCES runs(id) ON DELETE SET NULL,
                        provider TEXT NOT NULL,
                        external_session_id TEXT NOT NULL,
                        status TEXT NOT NULL,
                        metadata_json TEXT NOT NULL DEFAULT '{}',
                        started_at TEXT NOT NULL,
                        last_event_at TEXT NOT NULL,
                        ended_at TEXT,
                        UNIQUE(project_id, provider, external_session_id)
                    );
                    CREATE TABLE IF NOT EXISTS session_events (
                        id TEXT PRIMARY KEY,
                        session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                        event_name TEXT NOT NULL,
                        payload_json TEXT NOT NULL DEFAULT '{}',
                        idempotency_key TEXT UNIQUE,
                        created_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS checkpoints (
                        id TEXT PRIMARY KEY,
                        run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                        stage_id TEXT REFERENCES stages(id),
                        session_id TEXT REFERENCES sessions(id),
                        status TEXT NOT NULL,
                        next_action TEXT NOT NULL,
                        state_json TEXT NOT NULL,
                        pending_json TEXT NOT NULL,
                        state_digest TEXT NOT NULL,
                        context_digest TEXT NOT NULL DEFAULT '',
                        idempotency_key TEXT UNIQUE,
                        created_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS knowledge_entries (
                        id TEXT PRIMARY KEY,
                        project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                        run_id TEXT REFERENCES runs(id) ON DELETE CASCADE,
                        tier TEXT NOT NULL,
                        title TEXT NOT NULL,
                        content TEXT NOT NULL,
                        classification TEXT NOT NULL,
                        source_kind TEXT NOT NULL,
                        source_ref TEXT,
                        content_hash TEXT NOT NULL,
                        source_hash TEXT,
                        freshness TEXT NOT NULL,
                        status TEXT NOT NULL,
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL,
                        supersedes_id TEXT REFERENCES knowledge_entries(id)
                    );
                    CREATE TABLE IF NOT EXISTS sensor_results (
                        id TEXT PRIMARY KEY,
                        run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                        stage_id TEXT REFERENCES stages(id),
                        sensor_key TEXT NOT NULL,
                        severity TEXT NOT NULL,
                        status TEXT NOT NULL,
                        evidence_json TEXT NOT NULL DEFAULT '{}',
                        checked_at TEXT NOT NULL,
                        idempotency_key TEXT UNIQUE
                    );
                    PRAGMA user_version = 6;
                    """
                )
            elif current == 1:
                db.execute(
                    """
                    CREATE TABLE IF NOT EXISTS hook_events (
                        id TEXT PRIMARY KEY,
                        project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                        provider TEXT NOT NULL,
                        session_id TEXT,
                        event_name TEXT NOT NULL,
                        tool_name TEXT,
                        prompt_class TEXT,
                        decision TEXT NOT NULL,
                        payload_json TEXT NOT NULL,
                        created_at TEXT NOT NULL
                    )
                    """
                )
                db.execute(
                    "CREATE TABLE IF NOT EXISTS event_idempotency ("
                    "idempotency_key TEXT PRIMARY KEY, event_id TEXT NOT NULL REFERENCES events(id) "
                    "ON DELETE CASCADE, created_at TEXT NOT NULL)"
                )
                artifact_table = db.execute(
                    "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'artifacts'"
                ).fetchone()
                if artifact_table:
                    columns = {
                        str(row[1]) for row in db.execute("PRAGMA table_info(artifacts)").fetchall()
                    }
                    if "lineage_hash" not in columns:
                        db.execute(
                            "ALTER TABLE artifacts ADD COLUMN lineage_hash TEXT NOT NULL DEFAULT ''"
                        )
                events_table = db.execute(
                    "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'events'"
                ).fetchone()
                if events_table:
                    event_columns = {
                        str(row[1]) for row in db.execute("PRAGMA table_info(events)").fetchall()
                    }
                    if "sequence" not in event_columns:
                        db.execute(
                            "ALTER TABLE events ADD COLUMN sequence INTEGER NOT NULL DEFAULT 0"
                        )
                db.execute("PRAGMA user_version = 5")
            elif current == 2:
                db.execute(
                    "CREATE TABLE IF NOT EXISTS event_idempotency ("
                    "idempotency_key TEXT PRIMARY KEY, event_id TEXT NOT NULL REFERENCES events(id) "
                    "ON DELETE CASCADE, created_at TEXT NOT NULL)"
                )
                columns = {
                    str(row[1]) for row in db.execute("PRAGMA table_info(artifacts)").fetchall()
                }
                if "lineage_hash" not in columns:
                    db.execute(
                        "ALTER TABLE artifacts ADD COLUMN lineage_hash TEXT NOT NULL DEFAULT ''"
                    )
                event_columns = {
                    str(row[1]) for row in db.execute("PRAGMA table_info(events)").fetchall()
                }
                if "sequence" not in event_columns:
                    db.execute("ALTER TABLE events ADD COLUMN sequence INTEGER NOT NULL DEFAULT 0")
                db.execute("PRAGMA user_version = 5")
            elif current == 3:
                columns = {
                    str(row[1]) for row in db.execute("PRAGMA table_info(artifacts)").fetchall()
                }
                if "lineage_hash" not in columns:
                    db.execute(
                        "ALTER TABLE artifacts ADD COLUMN lineage_hash TEXT NOT NULL DEFAULT ''"
                    )
                event_columns = {
                    str(row[1]) for row in db.execute("PRAGMA table_info(events)").fetchall()
                }
                if "sequence" not in event_columns:
                    db.execute("ALTER TABLE events ADD COLUMN sequence INTEGER NOT NULL DEFAULT 0")
                db.execute("PRAGMA user_version = 5")
            elif current == 4:
                event_columns = {
                    str(row[1]) for row in db.execute("PRAGMA table_info(events)").fetchall()
                }
                if "sequence" not in event_columns:
                    db.execute("ALTER TABLE events ADD COLUMN sequence INTEGER NOT NULL DEFAULT 0")
                db.execute("PRAGMA user_version = 5")
            if current < SCHEMA_VERSION:
                self._create_resumable_tables(db)
                db.execute(f"PRAGMA user_version = {SCHEMA_VERSION}")

    @staticmethod
    def _create_resumable_tables(db: sqlite3.Connection) -> None:
        """Create additive v6 tables for sessions, checkpoints, knowledge and sensors."""
        db.executescript(
            """
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                run_id TEXT REFERENCES runs(id) ON DELETE SET NULL,
                provider TEXT NOT NULL,
                external_session_id TEXT NOT NULL,
                status TEXT NOT NULL,
                metadata_json TEXT NOT NULL DEFAULT '{}',
                started_at TEXT NOT NULL,
                last_event_at TEXT NOT NULL,
                ended_at TEXT,
                UNIQUE(project_id, provider, external_session_id)
            );
            CREATE TABLE IF NOT EXISTS session_events (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                event_name TEXT NOT NULL,
                payload_json TEXT NOT NULL DEFAULT '{}',
                idempotency_key TEXT UNIQUE,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                stage_id TEXT REFERENCES stages(id),
                session_id TEXT REFERENCES sessions(id),
                status TEXT NOT NULL,
                next_action TEXT NOT NULL,
                state_json TEXT NOT NULL,
                pending_json TEXT NOT NULL,
                state_digest TEXT NOT NULL,
                context_digest TEXT NOT NULL DEFAULT '',
                idempotency_key TEXT UNIQUE,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS knowledge_entries (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                run_id TEXT REFERENCES runs(id) ON DELETE CASCADE,
                tier TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                classification TEXT NOT NULL,
                source_kind TEXT NOT NULL,
                source_ref TEXT,
                content_hash TEXT NOT NULL,
                source_hash TEXT,
                freshness TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                supersedes_id TEXT REFERENCES knowledge_entries(id)
            );
            CREATE TABLE IF NOT EXISTS sensor_results (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                stage_id TEXT REFERENCES stages(id),
                sensor_key TEXT NOT NULL,
                severity TEXT NOT NULL,
                status TEXT NOT NULL,
                evidence_json TEXT NOT NULL DEFAULT '{}',
                checked_at TEXT NOT NULL,
                idempotency_key TEXT UNIQUE
            );
            """
        )

    def ensure_project(self) -> str:
        project_id = new_id()
        timestamp = now_iso()
        with self.connection() as db:
            row = db.execute(
                "SELECT id FROM projects WHERE root_path = ?", (str(self.project_dir),)
            ).fetchone()
            if row:
                db.execute("UPDATE projects SET updated_at = ? WHERE id = ?", (timestamp, row[0]))
                return str(row[0])
            db.execute(
                "INSERT INTO projects(id, root_path, created_at, updated_at) VALUES (?, ?, ?, ?)",
                (project_id, str(self.project_dir), timestamp, timestamp),
            )
        return project_id

    def project_id(self) -> str:
        self.ensure_project()
        with self.connection() as db:
            row = db.execute(
                "SELECT id FROM projects WHERE root_path = ?", (str(self.project_dir),)
            ).fetchone()
        if not row:
            raise StorageError("Project record could not be created")
        return str(row[0])

    def create_run(self, idea: str, mode: Mode, provider: str = "local-demo") -> str:
        run_id = new_id()
        timestamp = now_iso()
        project_id = self.project_id()
        with self.connection() as db:
            db.execute(
                "INSERT INTO runs(id, project_id, idea, mode, provider, status, created_at, updated_at) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    run_id,
                    project_id,
                    idea,
                    mode.value,
                    provider,
                    RunStatus.RUNNING.value,
                    timestamp,
                    timestamp,
                ),
            )
            for position, spec in enumerate(stage_specs(mode)):
                db.execute(
                    "INSERT INTO stages(id, run_id, stage_key, position, owner, dependencies_json, status) "
                    "VALUES (?, ?, ?, ?, ?, ?, ?)",
                    (
                        new_id(),
                        run_id,
                        spec.key,
                        position,
                        spec.owner,
                        json_dumps(list(spec.dependencies)),
                        StageStatus.PENDING.value,
                    ),
                )
            self._insert_event(
                db, run_id, None, "run.created", {"mode": mode.value, "provider": provider}
            )
        return run_id

    def _insert_event(
        self,
        db: sqlite3.Connection,
        run_id: str,
        stage_id: str | None,
        event_type: str,
        payload: Any,
        idempotency_key: str | None = None,
    ) -> str:
        if idempotency_key:
            existing = db.execute(
                "SELECT event_id FROM event_idempotency WHERE idempotency_key = ?",
                (idempotency_key,),
            ).fetchone()
            if existing:
                return str(existing[0])
        event_id = new_id()
        sequence = int(
            db.execute(
                "SELECT COALESCE(MAX(sequence), 0) + 1 FROM events WHERE run_id = ?", (run_id,)
            ).fetchone()[0]
        )
        db.execute(
            "INSERT INTO events(id, run_id, stage_id, sequence, event_type, payload_json, created_at) "
            "VALUES (?, ?, ?, ?, ?, ?, ?)",
            (event_id, run_id, stage_id, sequence, event_type, json_dumps(payload), now_iso()),
        )
        if idempotency_key:
            db.execute(
                "INSERT INTO event_idempotency(idempotency_key, event_id, created_at) VALUES (?, ?, ?)",
                (idempotency_key, event_id, now_iso()),
            )
        return event_id

    def event(
        self,
        run_id: str,
        event_type: str,
        payload: Any,
        stage_id: str | None = None,
        idempotency_key: str | None = None,
    ) -> None:
        with self.connection() as db:
            self._insert_event(db, run_id, stage_id, event_type, payload, idempotency_key)

    def hook_event(
        self,
        provider: str,
        event_name: str,
        session_id: str | None,
        tool_name: str | None,
        prompt_class: str | None,
        decision: str,
        payload: dict[str, Any],
    ) -> None:
        """Persist hook metadata without storing prompts, commands or tool arguments."""
        project_id = self.project_id()
        with self.connection() as db:
            db.execute(
                "INSERT INTO hook_events(id, project_id, provider, session_id, event_name, "
                "tool_name, prompt_class, decision, payload_json, created_at) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    new_id(),
                    project_id,
                    provider,
                    session_id,
                    event_name,
                    tool_name,
                    prompt_class,
                    decision,
                    json_dumps(payload),
                    now_iso(),
                ),
            )

    def record_session_event(
        self,
        provider: str,
        event_name: str,
        external_session_id: str,
        *,
        run_id: str | None = None,
        metadata: dict[str, Any] | None = None,
        idempotency_key: str | None = None,
    ) -> str:
        """Persist provider lifecycle metadata without prompts or tool arguments."""
        project_id = self.project_id()
        safe_metadata = {
            key: value
            for key, value in (metadata or {}).items()
            if key in {"source", "reason", "phase", "status", "event_kind"}
            and isinstance(value, (str, int, float, bool, type(None)))
        }
        event_lower = event_name.casefold()
        status = (
            "compacted"
            if "compact" in event_lower
            else "ended"
            if event_lower in {"sessionend", "session_end", "stop"}
            else "started"
            if event_lower in {"sessionstart", "session_start", "startup"}
            else "active"
        )
        timestamp = now_iso()
        with self.connection() as db:
            row = db.execute(
                "SELECT id FROM sessions WHERE project_id = ? AND provider = ? "
                "AND external_session_id = ?",
                (project_id, provider, external_session_id),
            ).fetchone()
            if row:
                session_id = str(row[0])
                db.execute(
                    "UPDATE sessions SET run_id = COALESCE(?, run_id), status = ?, "
                    "metadata_json = ?, last_event_at = ?, ended_at = ? WHERE id = ?",
                    (
                        run_id,
                        status,
                        json_dumps(safe_metadata),
                        timestamp,
                        timestamp if status == "ended" else None,
                        session_id,
                    ),
                )
            else:
                session_id = new_id()
                db.execute(
                    "INSERT INTO sessions(id, project_id, run_id, provider, external_session_id, "
                    "status, metadata_json, started_at, last_event_at, ended_at) "
                    "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    (
                        session_id,
                        project_id,
                        run_id,
                        provider,
                        external_session_id,
                        status,
                        json_dumps(safe_metadata),
                        timestamp,
                        timestamp,
                        timestamp if status == "ended" else None,
                    ),
                )
            existing_event = (
                db.execute(
                    "SELECT 1 FROM session_events WHERE idempotency_key = ?", (idempotency_key,)
                ).fetchone()
                if idempotency_key
                else None
            )
            if not existing_event:
                db.execute(
                    "INSERT INTO session_events(id, session_id, event_name, payload_json, "
                    "idempotency_key, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                    (
                        new_id(),
                        session_id,
                        event_name,
                        json_dumps(safe_metadata),
                        idempotency_key,
                        timestamp,
                    ),
                )
        return session_id

    def save_checkpoint(
        self,
        run_id: str,
        stage_key: str | None,
        session_id: str | None,
        *,
        next_action: str,
        state: dict[str, Any],
        pending: dict[str, Any] | list[Any] | None = None,
        context_digest: str = "",
        idempotency_key: str | None = None,
    ) -> str:
        """Save a resumable handoff and supersede the prior valid checkpoint."""
        pending_value: dict[str, Any] | list[Any] = pending if pending is not None else {}
        state_digest = hashlib.sha256(json_dumps(state).encode("utf-8")).hexdigest()
        with self.connection() as db:
            if idempotency_key:
                existing = db.execute(
                    "SELECT id FROM checkpoints WHERE idempotency_key = ?", (idempotency_key,)
                ).fetchone()
                if existing:
                    return str(existing[0])
            stage_id: str | None = None
            if stage_key:
                stage = db.execute(
                    "SELECT id FROM stages WHERE run_id = ? AND stage_key = ?", (run_id, stage_key)
                ).fetchone()
                stage_id = str(stage[0]) if stage else None
            db.execute(
                "UPDATE checkpoints SET status = 'superseded' WHERE run_id = ? AND status = 'valid'",
                (run_id,),
            )
            checkpoint_id = new_id()
            db.execute(
                "INSERT INTO checkpoints(id, run_id, stage_id, session_id, status, next_action, "
                "state_json, pending_json, state_digest, context_digest, idempotency_key, created_at) "
                "VALUES (?, ?, ?, ?, 'valid', ?, ?, ?, ?, ?, ?, ?)",
                (
                    checkpoint_id,
                    run_id,
                    stage_id,
                    session_id,
                    next_action,
                    json_dumps(state),
                    json_dumps(pending_value),
                    state_digest,
                    context_digest,
                    idempotency_key,
                    now_iso(),
                ),
            )
            self._insert_event(
                db,
                run_id,
                stage_id,
                "checkpoint.saved",
                {"id": checkpoint_id, "stage": stage_key, "next_action": next_action},
                idempotency_key=f"checkpoint-event:{checkpoint_id}",
            )
        return checkpoint_id

    def latest_checkpoint(self, run_id: str) -> dict[str, Any] | None:
        with self.connection() as db:
            row = db.execute(
                "SELECT * FROM checkpoints WHERE run_id = ? AND status = 'valid' "
                "ORDER BY created_at DESC LIMIT 1",
                (run_id,),
            ).fetchone()
        if not row:
            return None
        result = dict(row)
        for key in ("state_json", "pending_json"):
            result[key.removesuffix("_json")] = json.loads(str(result.pop(key)))
        return result

    def add_knowledge(
        self,
        *,
        title: str,
        content: str,
        classification: str,
        source_kind: str,
        tier: str = "project",
        run_id: str | None = None,
        source_ref: str | None = None,
        source_hash: str | None = None,
        freshness: str = "unknown",
        status: str = "active",
    ) -> str:
        """Store a provenance-labelled knowledge entry, never executable instructions."""
        project_id = self.project_id()
        content_hash = hashlib.sha256(content.encode("utf-8")).hexdigest()
        entry_id = new_id()
        timestamp = now_iso()
        with self.connection() as db:
            db.execute(
                "INSERT INTO knowledge_entries(id, project_id, run_id, tier, title, content, "
                "classification, source_kind, source_ref, content_hash, source_hash, freshness, "
                "status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    entry_id,
                    project_id,
                    run_id,
                    tier,
                    title,
                    content,
                    classification,
                    source_kind,
                    source_ref,
                    content_hash,
                    source_hash,
                    freshness,
                    status,
                    timestamp,
                    timestamp,
                ),
            )
            if run_id:
                self._insert_event(
                    db,
                    run_id,
                    None,
                    "knowledge.recorded",
                    {"entry_id": entry_id, "classification": classification},
                    idempotency_key=f"knowledge-recorded:{entry_id}",
                )
        return entry_id

    def list_knowledge(
        self, *, run_id: str | None = None, status: str = "active"
    ) -> list[dict[str, Any]]:
        project_id = self.project_id()
        with self.connection() as db:
            if run_id:
                rows = db.execute(
                    "SELECT * FROM knowledge_entries WHERE project_id = ? AND run_id = ? "
                    "AND status = ? ORDER BY updated_at DESC",
                    (project_id, run_id, status),
                ).fetchall()
            else:
                rows = db.execute(
                    "SELECT * FROM knowledge_entries WHERE project_id = ? AND status = ? "
                    "ORDER BY updated_at DESC",
                    (project_id, status),
                ).fetchall()
        return [dict(row) for row in rows]

    def knowledge_entry(self, entry_id: str) -> dict[str, Any] | None:
        with self.connection() as db:
            row = db.execute("SELECT * FROM knowledge_entries WHERE id = ?", (entry_id,)).fetchone()
        return dict(row) if row else None

    def update_knowledge_status(
        self, entry_id: str, *, status: str, freshness: str, reason: str
    ) -> None:
        with self.connection() as db:
            row = db.execute(
                "SELECT run_id FROM knowledge_entries WHERE id = ?", (entry_id,)
            ).fetchone()
            if not row:
                raise ValidationError(f"Knowledge entry not found: {entry_id}")
            db.execute(
                "UPDATE knowledge_entries SET status = ?, freshness = ?, updated_at = ? WHERE id = ?",
                (status, freshness, now_iso(), entry_id),
            )
            if row[0]:
                self._insert_event(
                    db,
                    str(row[0]),
                    None,
                    "knowledge.status_changed",
                    {"entry_id": entry_id, "status": status, "reason": reason},
                    idempotency_key=f"knowledge-status:{entry_id}:{status}:{freshness}",
                )

    def record_sensor_result(
        self,
        run_id: str,
        stage_key: str | None,
        sensor_key: str,
        severity: str,
        status: str,
        evidence: dict[str, Any] | None = None,
        *,
        idempotency_key: str | None = None,
    ) -> str:
        """Persist a deterministic sensor receipt; failure is never coerced to success."""
        with self.connection() as db:
            if idempotency_key:
                existing = db.execute(
                    "SELECT id FROM sensor_results WHERE idempotency_key = ?", (idempotency_key,)
                ).fetchone()
                if existing:
                    return str(existing[0])
            stage_id: str | None = None
            if stage_key:
                stage = db.execute(
                    "SELECT id FROM stages WHERE run_id = ? AND stage_key = ?", (run_id, stage_key)
                ).fetchone()
                stage_id = str(stage[0]) if stage else None
            result_id = new_id()
            db.execute(
                "INSERT INTO sensor_results(id, run_id, stage_id, sensor_key, severity, status, "
                "evidence_json, checked_at, idempotency_key) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    result_id,
                    run_id,
                    stage_id,
                    sensor_key,
                    severity,
                    status,
                    json_dumps(evidence or {}),
                    now_iso(),
                    idempotency_key,
                ),
            )
            self._insert_event(
                db,
                run_id,
                stage_id,
                "sensor.recorded",
                {"sensor_key": sensor_key, "status": status, "id": result_id},
                idempotency_key=f"sensor-event:{result_id}",
            )
        return result_id

    def sensor_results(self, run_id: str, stage_key: str | None = None) -> list[dict[str, Any]]:
        with self.connection() as db:
            query = (
                "SELECT sr.*, s.stage_key FROM sensor_results sr "
                "LEFT JOIN stages s ON s.id = sr.stage_id WHERE sr.run_id = ?"
            )
            parameters: list[Any] = [run_id]
            if stage_key:
                query += " AND s.stage_key = ?"
                parameters.append(stage_key)
            query += " ORDER BY checked_at"
            rows = db.execute(query, parameters).fetchall()
        results: list[dict[str, Any]] = []
        for row in rows:
            item = dict(row)
            item["evidence"] = json.loads(str(item.pop("evidence_json")))
            results.append(item)
        return results

    def latest_hook_context(self, provider: str, session_id: str | None) -> sqlite3.Row | None:
        """Return the latest prompt policy for a provider session."""
        project_id = self.project_id()
        with self.connection() as db:
            row = db.execute(
                "SELECT * FROM hook_events WHERE project_id = ? AND provider = ? "
                "AND session_id IS ? AND event_name IN ('UserPromptSubmit', 'BeforeAgent') "
                "ORDER BY created_at DESC LIMIT 1",
                (project_id, provider, session_id),
            ).fetchone()
        return cast(sqlite3.Row | None, row)

    def has_completed_run(self) -> bool:
        """Return whether this project has a verified completed product execution."""
        project_id = self.project_id()
        with self.connection() as db:
            row = db.execute(
                "SELECT 1 FROM runs WHERE project_id = ? AND status = 'completed' LIMIT 1",
                (project_id,),
            ).fetchone()
        return row is not None

    def stage(self, run_id: str, key: str) -> sqlite3.Row:
        with self.connection() as db:
            row = db.execute(
                "SELECT * FROM stages WHERE run_id = ? AND stage_key = ?", (run_id, key)
            ).fetchone()
        if not row:
            raise ValidationError(f"Unknown stage: {key}")
        return cast(sqlite3.Row, row)

    def completed_stages(self, run_id: str) -> set[str]:
        with self.connection() as db:
            rows = db.execute(
                "SELECT stage_key FROM stages WHERE run_id = ? AND status = 'completed'", (run_id,)
            ).fetchall()
        return {str(row[0]) for row in rows}

    def transition_stage(
        self,
        run_id: str,
        key: str,
        status: StageStatus,
        error: str | None = None,
        idempotency_key: str | None = None,
    ) -> None:
        with self.connection() as db:
            if idempotency_key:
                already_applied = db.execute(
                    "SELECT 1 FROM event_idempotency WHERE idempotency_key = ?",
                    (idempotency_key,),
                ).fetchone()
                if already_applied:
                    return
            row = db.execute(
                "SELECT * FROM stages WHERE run_id = ? AND stage_key = ?", (run_id, key)
            ).fetchone()
            if not row:
                raise ValidationError(f"Unknown stage: {key}")
            old = StageStatus(row["status"])
            validate_transition(old, status)
            now = now_iso()
            attempts = int(row["attempts"]) + (1 if status == StageStatus.RUNNING else 0)
            db.execute(
                "UPDATE stages SET status = ?, attempts = ?, started_at = COALESCE(started_at, ?), "
                "completed_at = ?, error_message = ? WHERE id = ?",
                (
                    status.value,
                    attempts,
                    now if status == StageStatus.RUNNING else row["started_at"],
                    now
                    if status in {StageStatus.COMPLETED, StageStatus.BLOCKED, StageStatus.FAILED}
                    else None,
                    error,
                    row["id"],
                ),
            )
            self._insert_event(
                db,
                run_id,
                row["id"],
                f"stage.{status.value}",
                {"stage": key, "error": error},
                idempotency_key,
            )

    def add_artifact(
        self,
        run_id: str,
        stage_key: str,
        kind: str,
        payload: dict[str, Any],
        dependencies: list[str] | None = None,
        idempotency_key: str | None = None,
    ) -> str:
        import hashlib

        artifact_id = new_id()
        stage = self.stage(run_id, stage_key)
        dependencies = dependencies or []
        serialized = json_dumps(payload)
        content_hash = hashlib.sha256(serialized.encode("utf-8")).hexdigest()
        lineage_hash = hashlib.sha256(
            json_dumps({"content_hash": content_hash, "dependencies": sorted(dependencies)}).encode(
                "utf-8"
            )
        ).hexdigest()
        with self.connection() as db:
            if idempotency_key:
                existing = db.execute(
                    "SELECT e.payload_json FROM event_idempotency k "
                    "JOIN events e ON e.id = k.event_id "
                    "WHERE k.idempotency_key = ? AND e.event_type = 'artifact.created'",
                    (idempotency_key,),
                ).fetchone()
                if existing:
                    stored = json.loads(existing[0])
                    return str(stored["id"])
            revision = db.execute(
                "SELECT COALESCE(MAX(revision), 0) + 1 FROM artifacts WHERE run_id = ? AND kind = ?",
                (run_id, kind),
            ).fetchone()[0]
            previous = db.execute(
                "SELECT id FROM artifacts WHERE run_id = ? AND kind = ? ORDER BY revision DESC LIMIT 1",
                (run_id, kind),
            ).fetchone()
            if previous:
                db.execute(
                    "UPDATE artifacts SET status = ? WHERE id = ?",
                    (ArtifactStatus.SUPERSEDED.value, previous[0]),
                )
            db.execute(
                "INSERT INTO artifacts(id, run_id, stage_id, kind, revision, status, payload_json, dependencies_json, content_hash, lineage_hash, created_at, supersedes_id) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    artifact_id,
                    run_id,
                    stage["id"],
                    kind,
                    revision,
                    ArtifactStatus.VALID.value,
                    serialized,
                    json_dumps(dependencies),
                    content_hash,
                    lineage_hash,
                    now_iso(),
                    previous[0] if previous else None,
                ),
            )
            db.execute(
                "UPDATE stages SET current_artifact_id = ? WHERE id = ?", (artifact_id, stage["id"])
            )
            self._insert_event(
                db,
                run_id,
                stage["id"],
                "artifact.created",
                {"kind": kind, "id": artifact_id},
                idempotency_key,
            )
        return artifact_id

    def invalidate_from_stage(self, run_id: str, changed_stage: str) -> int:
        """Mark direct and transitive dependent artifacts as stale."""
        with self.connection() as db:
            rows = db.execute(
                "SELECT a.id, a.stage_id, a.dependencies_json, s.stage_key "
                "FROM artifacts a JOIN stages s ON s.id = a.stage_id "
                "WHERE a.run_id = ? AND a.status = ?",
                (run_id, ArtifactStatus.VALID.value),
            ).fetchall()
            stale: list[str] = []
            stale_stages = {changed_stage}
            remaining = list(rows)
            while remaining:
                progressed = False
                for row in remaining[:]:
                    dependencies = json.loads(row["dependencies_json"])
                    if any(
                        str(item) in stale_stages or str(item) in stale for item in dependencies
                    ):
                        stale.append(str(row["id"]))
                        stale_stages.add(str(row["stage_key"]))
                        remaining.remove(row)
                        progressed = True
                if not progressed:
                    break
            for artifact_id in set(stale):
                db.execute(
                    "UPDATE artifacts SET status = ? WHERE id = ?",
                    (ArtifactStatus.STALE.value, artifact_id),
                )
            if stale:
                self._insert_event(
                    db,
                    run_id,
                    None,
                    "artifacts.invalidated",
                    {"stage": changed_stage, "artifacts": stale},
                )
        return len(stale)

    def add_record(self, table: str, values: dict[str, Any]) -> str:
        allowed = {
            "decisions": (
                "title",
                "decision_text",
                "classification",
                "rationale",
                "source_kind",
                "source_ref",
            ),
            "hypotheses": (
                "statement",
                "risk_level",
                "evidence_state",
                "test_method",
                "success_signal",
            ),
            "evidence": (
                "claim",
                "source_kind",
                "source_ref",
                "classification",
                "freshness",
                "verified_by",
            ),
        }
        if table not in allowed:
            raise StorageError(f"Unsupported record table: {table}")
        record_id = new_id()
        columns = ["id", "run_id", *allowed[table], "created_at"]
        payload = [
            record_id,
            values["run_id"],
            *(values.get(column) for column in allowed[table]),
            now_iso(),
        ]
        placeholders = ", ".join("?" for _ in columns)
        with self.connection() as db:
            db.execute(
                f"INSERT INTO {table}({', '.join(columns)}) VALUES ({placeholders})", payload
            )
        return record_id

    def validation(
        self,
        run_id: str,
        stage_id: str | None,
        artifact_id: str | None,
        key: str,
        result: str,
        details: str,
        idempotency_key: str | None = None,
    ) -> None:
        with self.connection() as db:
            if (
                idempotency_key
                and db.execute(
                    "SELECT 1 FROM event_idempotency WHERE idempotency_key = ?",
                    (idempotency_key,),
                ).fetchone()
            ):
                return
            validation_id = new_id()
            db.execute(
                "INSERT INTO validations(id, run_id, stage_id, artifact_id, check_key, result, details, checked_at) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (validation_id, run_id, stage_id, artifact_id, key, result, details, now_iso()),
            )
            self._insert_event(
                db,
                run_id,
                stage_id,
                "validation.recorded",
                {"check": key, "result": result, "id": validation_id},
                idempotency_key,
            )

    def save_capabilities(self, capabilities: list[dict[str, Any]]) -> None:
        project_id = self.project_id()
        with self.connection() as db:
            for capability in capabilities:
                db.execute(
                    "INSERT INTO capabilities(id, project_id, provider, capability_key, status, source_path, source_hash, observed_version, details, observed_at) "
                    "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(project_id, provider, capability_key) DO UPDATE SET status=excluded.status, source_path=excluded.source_path, source_hash=excluded.source_hash, observed_version=excluded.observed_version, details=excluded.details, observed_at=excluded.observed_at",
                    (
                        new_id(),
                        project_id,
                        capability["provider"],
                        capability["key"],
                        capability["status"],
                        capability.get("source_path"),
                        capability.get("source_hash"),
                        capability.get("observed_version"),
                        capability.get("details", ""),
                        now_iso(),
                    ),
                )

    def list_capabilities(self) -> list[dict[str, Any]]:
        project_id = self.project_id()
        with self.connection() as db:
            rows = db.execute(
                "SELECT * FROM capabilities WHERE project_id = ? ORDER BY provider, capability_key",
                (project_id,),
            ).fetchall()
        return [dict(row) for row in rows]

    def save_manifest(
        self, provider: str, manifest_path: str, version: str, components: list[dict[str, Any]]
    ) -> None:
        project_id = self.project_id()
        with self.connection() as db:
            now = now_iso()
            db.execute(
                "INSERT INTO installation_manifests(id, project_id, provider, tramuri_version, manifest_path, components_json, created_at, updated_at) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(project_id, provider) DO UPDATE SET tramuri_version=excluded.tramuri_version, manifest_path=excluded.manifest_path, components_json=excluded.components_json, updated_at=excluded.updated_at",
                (
                    new_id(),
                    project_id,
                    provider,
                    version,
                    manifest_path,
                    json_dumps(components),
                    now,
                    now,
                ),
            )

    def manifest(self, provider: str) -> sqlite3.Row | None:
        project_id = self.project_id()
        with self.connection() as db:
            return cast(
                sqlite3.Row | None,
                db.execute(
                    "SELECT * FROM installation_manifests WHERE project_id = ? AND provider = ?",
                    (project_id, provider),
                ).fetchone(),
            )

    def list_manifests(self) -> list[dict[str, Any]]:
        project_id = self.project_id()
        with self.connection() as db:
            rows = db.execute(
                "SELECT * FROM installation_manifests WHERE project_id = ? ORDER BY provider",
                (project_id,),
            ).fetchall()
        return [dict(row) for row in rows]

    def list_runs(self) -> list[dict[str, Any]]:
        with self.connection() as db:
            rows = db.execute("SELECT * FROM runs ORDER BY created_at DESC").fetchall()
        return [dict(row) for row in rows]

    def run_details(self, run_id: str) -> dict[str, Any]:
        with self.connection() as db:
            run = db.execute("SELECT * FROM runs WHERE id = ?", (run_id,)).fetchone()
            if not run:
                raise ValidationError(f"Run not found: {run_id}")
            stages = db.execute(
                "SELECT * FROM stages WHERE run_id = ? ORDER BY position", (run_id,)
            ).fetchall()
            artifacts = db.execute(
                "SELECT * FROM artifacts WHERE run_id = ? ORDER BY created_at", (run_id,)
            ).fetchall()
            decisions = db.execute(
                "SELECT * FROM decisions WHERE run_id = ? ORDER BY created_at", (run_id,)
            ).fetchall()
            hypotheses = db.execute(
                "SELECT * FROM hypotheses WHERE run_id = ? ORDER BY created_at", (run_id,)
            ).fetchall()
            evidence = db.execute(
                "SELECT * FROM evidence WHERE run_id = ? ORDER BY created_at", (run_id,)
            ).fetchall()
            events = db.execute(
                "SELECT * FROM events WHERE run_id = ? ORDER BY sequence, created_at", (run_id,)
            ).fetchall()
            validations = db.execute(
                "SELECT * FROM validations WHERE run_id = ? ORDER BY checked_at", (run_id,)
            ).fetchall()
            checkpoints = db.execute(
                "SELECT * FROM checkpoints WHERE run_id = ? ORDER BY created_at", (run_id,)
            ).fetchall()
            sessions = db.execute(
                "SELECT * FROM sessions WHERE run_id = ? ORDER BY last_event_at", (run_id,)
            ).fetchall()
            knowledge = db.execute(
                "SELECT * FROM knowledge_entries WHERE run_id = ? ORDER BY updated_at", (run_id,)
            ).fetchall()
            sensor_results = db.execute(
                "SELECT sr.*, s.stage_key FROM sensor_results sr "
                "LEFT JOIN stages s ON s.id = sr.stage_id WHERE sr.run_id = ? ORDER BY checked_at",
                (run_id,),
            ).fetchall()

        def unpack(row: sqlite3.Row) -> dict[str, Any]:
            item = dict(row)
            for key in ("payload_json", "dependencies_json"):
                if key in item and isinstance(item[key], str):
                    with suppress(ValueError):
                        item[key.removesuffix("_json")] = json.loads(item[key])
            return item

        return {
            "run": dict(run),
            "stages": [unpack(row) for row in stages],
            "artifacts": [unpack(row) for row in artifacts],
            "decisions": [dict(row) for row in decisions],
            "hypotheses": [dict(row) for row in hypotheses],
            "evidence": [dict(row) for row in evidence],
            "events": [unpack(row) for row in events],
            "validations": [dict(row) for row in validations],
            "checkpoints": [
                {
                    **dict(row),
                    "state": json.loads(str(row["state_json"])),
                    "pending": json.loads(str(row["pending_json"])),
                }
                for row in checkpoints
            ],
            "sessions": [dict(row) for row in sessions],
            "knowledge": [dict(row) for row in knowledge],
            "sensor_results": [
                {**dict(row), "evidence": json.loads(str(row["evidence_json"]))}
                for row in sensor_results
            ],
        }

    def complete_run(self, run_id: str, status: RunStatus = RunStatus.COMPLETED) -> None:
        with self.connection() as db:
            db.execute(
                "UPDATE runs SET status = ?, updated_at = ?, completed_at = ? WHERE id = ?",
                (
                    status.value,
                    now_iso(),
                    now_iso() if status == RunStatus.COMPLETED else None,
                    run_id,
                ),
            )
            self._insert_event(db, run_id, None, f"run.{status.value}", {})

    def resume_run(self, run_id: str, idempotency_key: str | None = None) -> None:
        with self.connection() as db:
            row = db.execute("SELECT id FROM runs WHERE id = ?", (run_id,)).fetchone()
            if not row:
                raise ValidationError(f"Run not found: {run_id}")
            db.execute(
                "UPDATE runs SET status = ?, updated_at = ? WHERE id = ?",
                (RunStatus.RUNNING.value, now_iso(), run_id),
            )
            self._insert_event(db, run_id, None, "run.resumed", {}, idempotency_key)
