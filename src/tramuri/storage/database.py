"""SQLite persistence for runs, artifacts, events and provider manifests."""

from __future__ import annotations

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
from tramuri.core.workflow import stage_specs, validate_transition

SCHEMA_VERSION = 1


class Database:
    """Small repository around one project-local SQLite database."""

    def __init__(self, project_dir: Path):
        self.project_dir = project_dir.resolve()
        self.path = self.project_dir / ".tramuri" / "database" / "tramuri.sqlite3"
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
                    PRAGMA user_version = 1;
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
    ) -> None:
        db.execute(
            "INSERT INTO events(id, run_id, stage_id, event_type, payload_json, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            (new_id(), run_id, stage_id, event_type, json_dumps(payload), now_iso()),
        )

    def event(
        self, run_id: str, event_type: str, payload: Any, stage_id: str | None = None
    ) -> None:
        with self.connection() as db:
            self._insert_event(db, run_id, stage_id, event_type, payload)

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
        self, run_id: str, key: str, status: StageStatus, error: str | None = None
    ) -> None:
        with self.connection() as db:
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
                db, run_id, row["id"], f"stage.{status.value}", {"stage": key, "error": error}
            )

    def add_artifact(
        self,
        run_id: str,
        stage_key: str,
        kind: str,
        payload: dict[str, Any],
        dependencies: list[str] | None = None,
    ) -> str:
        import hashlib

        artifact_id = new_id()
        stage = self.stage(run_id, stage_key)
        dependencies = dependencies or []
        serialized = json_dumps(payload)
        content_hash = hashlib.sha256(serialized.encode("utf-8")).hexdigest()
        with self.connection() as db:
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
                "INSERT INTO artifacts(id, run_id, stage_id, kind, revision, status, payload_json, dependencies_json, content_hash, created_at, supersedes_id) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
                    now_iso(),
                    previous[0] if previous else None,
                ),
            )
            db.execute(
                "UPDATE stages SET current_artifact_id = ? WHERE id = ?", (artifact_id, stage["id"])
            )
            self._insert_event(
                db, run_id, stage["id"], "artifact.created", {"kind": kind, "id": artifact_id}
            )
        return artifact_id

    def invalidate_from_stage(self, run_id: str, changed_stage: str) -> int:
        """Mark only artifacts that declare the changed stage as stale."""
        with self.connection() as db:
            rows = db.execute(
                "SELECT id, dependencies_json FROM artifacts WHERE run_id = ? AND status = ?",
                (run_id, ArtifactStatus.VALID.value),
            ).fetchall()
            stale: list[str] = []
            for row in rows:
                dependencies = json.loads(row[1])
                if changed_stage in dependencies:
                    stale.append(str(row[0]))
            for artifact_id in stale:
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
    ) -> None:
        with self.connection() as db:
            db.execute(
                "INSERT INTO validations(id, run_id, stage_id, artifact_id, check_key, result, details, checked_at) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (new_id(), run_id, stage_id, artifact_id, key, result, details, now_iso()),
            )
            self._insert_event(
                db, run_id, stage_id, "validation.recorded", {"check": key, "result": result}
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
                "SELECT * FROM events WHERE run_id = ? ORDER BY created_at", (run_id,)
            ).fetchall()
            validations = db.execute(
                "SELECT * FROM validations WHERE run_id = ? ORDER BY checked_at", (run_id,)
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

    def resume_run(self, run_id: str) -> None:
        with self.connection() as db:
            row = db.execute("SELECT id FROM runs WHERE id = ?", (run_id,)).fetchone()
            if not row:
                raise ValidationError(f"Run not found: {run_id}")
            db.execute(
                "UPDATE runs SET status = ?, updated_at = ? WHERE id = ?",
                (RunStatus.RUNNING.value, now_iso(), run_id),
            )
            self._insert_event(db, run_id, None, "run.resumed", {})
