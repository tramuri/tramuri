use std::ffi::OsString;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::{HashStatus, Result, SCHEMA_VERSION, TramuriError};

pub(crate) const APP_DIR: &str = ".tramuri";
pub(crate) const DATABASE_FILE: &str = "tramuri.sqlite3";
const DATABASE_DIR: &str = "database";
const LEGACY_DATABASE_FILE: &str = DATABASE_FILE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoredFile {
    pub path: String,
    pub size_bytes: u64,
    pub modified_unix_ms: Option<i64>,
    pub sha256: Option<String>,
    pub hash_status: HashStatus,
    pub last_seen_scan_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScanObservation {
    pub id: i64,
    pub complete: bool,
}

pub(crate) fn database_path(root: &Path) -> PathBuf {
    root.join(APP_DIR).join(DATABASE_DIR).join(DATABASE_FILE)
}

fn legacy_database_path(root: &Path) -> PathBuf {
    root.join(APP_DIR).join(LEGACY_DATABASE_FILE)
}

pub(crate) fn open_database(root: &Path) -> Result<Connection> {
    let app_dir = root.join(APP_DIR);
    let app_metadata = match std::fs::symlink_metadata(&app_dir) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(TramuriError::NotInitialized(database_path(root)));
        }
        Err(source) => {
            return Err(TramuriError::Io {
                operation: "inspect Tramuri state directory",
                path: app_dir,
                source,
            });
        }
    };
    if app_metadata.file_type().is_symlink() || !app_metadata.is_dir() {
        return Err(TramuriError::InvalidStatePath(app_dir));
    }

    let database_dir = root.join(APP_DIR).join(DATABASE_DIR);
    ensure_directory(&database_dir, false, "inspect Tramuri database directory")?;
    migrate_legacy_database(root)?;
    ensure_directory(&database_dir, false, "inspect Tramuri database directory")?;

    let path = database_path(root);
    let metadata = match std::fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(TramuriError::NotInitialized(path));
        }
        Err(source) => {
            return Err(TramuriError::Io {
                operation: "inspect Tramuri database",
                path,
                source,
            });
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(TramuriError::InvalidStatePath(path));
    }
    let mut connection = Connection::open(&path)?;
    enable_foreign_keys(&connection)?;
    migrate_schema(&mut connection)?;
    validate_schema(&connection)?;
    Ok(connection)
}

pub(crate) fn initialize_database(root: &Path) -> Result<bool> {
    let app_dir = root.join(APP_DIR);
    match std::fs::symlink_metadata(&app_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(TramuriError::InvalidStatePath(app_dir));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir(&app_dir).map_err(|source| TramuriError::Io {
                operation: "create Tramuri state directory",
                path: app_dir.clone(),
                source,
            })?;
        }
        Err(source) => {
            return Err(TramuriError::Io {
                operation: "inspect Tramuri state directory",
                path: app_dir,
                source,
            });
        }
    }

    let database_dir = root.join(APP_DIR).join(DATABASE_DIR);
    ensure_directory(&database_dir, true, "create Tramuri database directory")?;
    migrate_legacy_database(root)?;

    let path = database_path(root);
    let already_initialized = match std::fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(TramuriError::InvalidStatePath(path));
        }
        Ok(_) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(source) => {
            return Err(TramuriError::Io {
                operation: "inspect Tramuri database",
                path,
                source,
            });
        }
    };
    let mut connection = Connection::open(&path)?;
    enable_foreign_keys(&connection)?;
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        0 => {
            if already_initialized {
                let user_tables: i64 = connection.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
                    [],
                    |row| row.get(0),
                )?;
                if user_tables != 0 {
                    return Err(TramuriError::UnsupportedSchema {
                        found: 0,
                        supported: SCHEMA_VERSION,
                    });
                }
            }
            create_v3_schema(&mut connection)?;
            migrate_v3_to_v4(&mut connection)?;
            migrate_v4_to_v5(&mut connection)?;
        }
        1 => {
            migrate_v1_to_v2(&mut connection)?;
            migrate_v2_to_v3(&mut connection)?;
            migrate_v3_to_v4(&mut connection)?;
            migrate_v4_to_v5(&mut connection)?;
        }
        2 => {
            migrate_v2_to_v3(&mut connection)?;
            migrate_v3_to_v4(&mut connection)?;
            migrate_v4_to_v5(&mut connection)?;
        }
        3 => {
            migrate_v3_to_v4(&mut connection)?;
            migrate_v4_to_v5(&mut connection)?;
        }
        4 => migrate_v4_to_v5(&mut connection)?,
        version if version == i64::from(SCHEMA_VERSION) => {}
        found => {
            return Err(TramuriError::UnsupportedSchema {
                found,
                supported: SCHEMA_VERSION,
            });
        }
    }

    Ok(already_initialized)
}

fn ensure_directory(path: &Path, create: bool, operation: &'static str) -> Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            Err(TramuriError::InvalidStatePath(path.to_path_buf()))
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !create => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir(path).map_err(|source| TramuriError::Io {
                operation,
                path: path.to_path_buf(),
                source,
            })?;
            Ok(true)
        }
        Err(source) => Err(TramuriError::Io {
            operation: "inspect Tramuri database directory",
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// Relocate the pre-0.3 database without overwriting a database at the new path.
/// SQLite is opened and checked first so committed WAL data is visible. Migration is
/// refused if checkpointing reports a busy connection or if any SQLite sidecar remains.
/// The main file is moved only after those checks succeed and the connection is closed.
fn migrate_legacy_database(root: &Path) -> Result<()> {
    let legacy_path = legacy_database_path(root);
    let database_path = database_path(root);
    let legacy_metadata = match std::fs::symlink_metadata(&legacy_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }
        Err(source) => {
            return Err(TramuriError::Io {
                operation: "inspect legacy Tramuri database",
                path: legacy_path,
                source,
            });
        }
    };
    if legacy_metadata.file_type().is_symlink() || !legacy_metadata.is_file() {
        return Err(TramuriError::InvalidStatePath(legacy_path));
    }

    ensure_directory(
        database_path
            .parent()
            .expect("database path has a parent directory"),
        true,
        "create Tramuri database directory",
    )?;

    match std::fs::symlink_metadata(&database_path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(TramuriError::InvalidStatePath(database_path));
        }
        Ok(_) => {
            return Err(TramuriError::ConflictingStateDatabases {
                legacy_path,
                database_path,
            });
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(TramuriError::Io {
                operation: "inspect Tramuri database",
                path: database_path,
                source,
            });
        }
    }

    let suffixes = ["-wal", "-shm", "-journal"];
    for suffix in suffixes {
        let new_sidecar = sidecar_path(&database_path, suffix);
        match std::fs::symlink_metadata(&new_sidecar) {
            Ok(_) => return Err(TramuriError::InvalidStatePath(new_sidecar)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(TramuriError::Io {
                    operation: "inspect Tramuri database sidecar",
                    path: new_sidecar,
                    source,
                });
            }
        }
    }

    let connection =
        Connection::open_with_flags(&legacy_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    let integrity: String = connection.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(TramuriError::InvalidStatePath(legacy_path));
    }
    let (busy, _, _): (i64, i64, i64) =
        connection.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
    if busy != 0 {
        return Err(TramuriError::DatabaseBusyForMigration(legacy_path));
    }
    drop(connection);

    for suffix in suffixes {
        let old_sidecar = sidecar_path(&legacy_path, suffix);
        match std::fs::symlink_metadata(&old_sidecar) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(TramuriError::InvalidStatePath(old_sidecar));
            }
            Ok(_) => return Err(TramuriError::DatabaseBusyForMigration(legacy_path)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(TramuriError::Io {
                    operation: "inspect legacy Tramuri database sidecar",
                    path: old_sidecar,
                    source,
                });
            }
        }
    }

    std::fs::rename(&legacy_path, &database_path).map_err(|source| TramuriError::Io {
        operation: "move legacy Tramuri database",
        path: database_path,
        source,
    })?;

    Ok(())
}

fn sidecar_path(database_path: &Path, suffix: &str) -> PathBuf {
    let mut value: OsString = database_path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

fn enable_foreign_keys(connection: &Connection) -> Result<()> {
    connection.pragma_update(None, "foreign_keys", true)?;
    Ok(())
}

fn migrate_schema(connection: &mut Connection) -> Result<()> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        1 => {
            migrate_v1_to_v2(connection)?;
            migrate_v2_to_v3(connection)?;
            migrate_v3_to_v4(connection)?;
            migrate_v4_to_v5(connection)
        }
        2 => {
            migrate_v2_to_v3(connection)?;
            migrate_v3_to_v4(connection)?;
            migrate_v4_to_v5(connection)
        }
        3 => {
            migrate_v3_to_v4(connection)?;
            migrate_v4_to_v5(connection)
        }
        4 => migrate_v4_to_v5(connection),
        version if version == i64::from(SCHEMA_VERSION) => Ok(()),
        found => Err(TramuriError::UnsupportedSchema {
            found,
            supported: SCHEMA_VERSION,
        }),
    }
}

fn create_v3_schema(connection: &mut Connection) -> Result<()> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    create_v2_tables(&transaction)?;
    create_mission_tables(&transaction)?;
    transaction.pragma_update(None, "user_version", 3)?;
    transaction.commit()?;
    Ok(())
}

fn migrate_v1_to_v2(connection: &mut Connection) -> Result<()> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    create_project_and_scan_tables(&transaction)?;
    transaction.execute(
        "ALTER TABLE file_inventory ADD COLUMN last_seen_scan_id INTEGER
         REFERENCES scan_runs(id)",
        [],
    )?;
    create_knowledge_tables(&transaction)?;
    insert_project_id(&transaction)?;
    transaction.pragma_update(None, "user_version", 2)?;
    transaction.commit()?;
    Ok(())
}

fn migrate_v2_to_v3(connection: &mut Connection) -> Result<()> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    create_mission_tables(&transaction)?;
    transaction.pragma_update(None, "user_version", 3)?;
    transaction.commit()?;
    Ok(())
}

fn migrate_v3_to_v4(connection: &mut Connection) -> Result<()> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute_batch(
        "CREATE TABLE executions (
            id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36),
            mission_id TEXT NOT NULL REFERENCES missions(id),
            contract_id TEXT NOT NULL,
            contract_revision INTEGER NOT NULL CHECK (contract_revision > 0),
            actor_kind TEXT NOT NULL CHECK (actor_kind IN ('human', 'agent', 'tool')),
            actor_reference TEXT,
            state TEXT NOT NULL CHECK (state IN ('in_progress', 'completed', 'blocked', 'failed', 'cancelled')),
            started_at_unix_ms INTEGER NOT NULL,
            ended_at_unix_ms INTEGER,
            notes TEXT,
            FOREIGN KEY (contract_id, contract_revision) REFERENCES success_contracts(id, revision)
        );
        CREATE INDEX executions_mission_order ON executions(mission_id, started_at_unix_ms, id);
        CREATE TABLE mission_evidence (
            id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36),
            mission_id TEXT NOT NULL REFERENCES missions(id),
            execution_id TEXT NOT NULL REFERENCES executions(id),
            kind TEXT NOT NULL CHECK (kind IN ('project_file', 'external_reference', 'human_attestation')),
            locator TEXT NOT NULL CHECK (length(trim(locator)) > 0),
            sha256 TEXT,
            hash_status TEXT CHECK (hash_status IN ('hashed', 'too_large')),
            scan_id INTEGER REFERENCES scan_runs(id),
            availability TEXT NOT NULL CHECK (availability IN ('observed', 'changed', 'missing', 'unverified')),
            captured_at_unix_ms INTEGER NOT NULL
        );
        CREATE INDEX mission_evidence_execution_order ON mission_evidence(execution_id, captured_at_unix_ms, id);
        CREATE TRIGGER mission_evidence_execution_mission_insert
        BEFORE INSERT ON mission_evidence
        WHEN (SELECT mission_id FROM executions WHERE id = NEW.execution_id) != NEW.mission_id
        BEGIN SELECT RAISE(ABORT, 'evidence mission does not match execution mission'); END;",
    )?;
    transaction.pragma_update(None, "user_version", 4)?;
    transaction.commit()?;
    Ok(())
}

fn migrate_v4_to_v5(connection: &mut Connection) -> Result<()> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute_batch(
        "CREATE UNIQUE INDEX success_criteria_assessment_identity
            ON success_criteria(id, contract_id, contract_revision);
        CREATE TABLE criterion_assessments (
            id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36),
            contract_id TEXT NOT NULL,
            contract_revision INTEGER NOT NULL CHECK (contract_revision > 0),
            criterion_id TEXT NOT NULL,
            result TEXT NOT NULL CHECK (result IN ('met', 'not_met', 'unverified')),
            verifier_kind TEXT NOT NULL CHECK (verifier_kind IN ('human', 'mechanism')),
            verifier_reference TEXT NOT NULL CHECK (length(trim(verifier_reference)) > 0),
            assessed_at_unix_ms INTEGER NOT NULL,
            explanation TEXT,
            FOREIGN KEY (contract_id, contract_revision)
                REFERENCES success_contracts(id, revision),
            FOREIGN KEY (criterion_id, contract_id, contract_revision)
                REFERENCES success_criteria(id, contract_id, contract_revision)
        );
        CREATE INDEX criterion_assessments_order
            ON criterion_assessments(contract_id, contract_revision, criterion_id, assessed_at_unix_ms, id);
        CREATE TABLE criterion_assessment_evidence (
            assessment_id TEXT NOT NULL REFERENCES criterion_assessments(id),
            evidence_id TEXT NOT NULL REFERENCES mission_evidence(id),
            PRIMARY KEY (assessment_id, evidence_id)
        ) WITHOUT ROWID;
        CREATE TRIGGER assessment_evidence_revision_insert
        BEFORE INSERT ON criterion_assessment_evidence
        WHEN NOT EXISTS (
            SELECT 1 FROM criterion_assessments AS a
            JOIN success_contracts AS c ON c.id = a.contract_id AND c.revision = a.contract_revision
            JOIN mission_evidence AS e ON e.id = NEW.evidence_id
            JOIN executions AS x ON x.id = e.execution_id
            WHERE a.id = NEW.assessment_id
              AND c.mission_id = x.mission_id
              AND x.contract_id = a.contract_id
              AND x.contract_revision = a.contract_revision
        )
        BEGIN SELECT RAISE(ABORT, 'assessment evidence does not match mission contract revision'); END;",
    )?;
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

fn create_mission_tables(transaction: &rusqlite::Transaction<'_>) -> Result<()> {
    transaction.execute_batch(
        "CREATE TABLE missions (
            id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36),
            objective TEXT NOT NULL CHECK (length(trim(objective)) > 0),
            context TEXT,
            state TEXT NOT NULL CHECK (state IN ('open', 'closed')),
            created_at_unix_ms INTEGER NOT NULL
        );
        CREATE INDEX missions_created_order ON missions(created_at_unix_ms, id);
        CREATE TABLE success_contracts (
            id TEXT NOT NULL CHECK (length(id) = 36),
            mission_id TEXT NOT NULL REFERENCES missions(id),
            revision INTEGER NOT NULL CHECK (revision > 0),
            created_at_unix_ms INTEGER NOT NULL,
            PRIMARY KEY (id, revision),
            UNIQUE (mission_id, revision)
        );
        CREATE TABLE success_criteria (
            id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36),
            contract_id TEXT NOT NULL,
            contract_revision INTEGER NOT NULL,
            description TEXT NOT NULL CHECK (length(trim(description)) > 0),
            required INTEGER NOT NULL CHECK (required IN (0, 1)),
            evidence_expectation TEXT,
            position INTEGER NOT NULL CHECK (position >= 0),
            UNIQUE (contract_id, contract_revision, position),
            FOREIGN KEY (contract_id, contract_revision)
                REFERENCES success_contracts(id, revision)
        );",
    )?;
    Ok(())
}

fn create_v2_tables(transaction: &rusqlite::Transaction<'_>) -> Result<()> {
    create_project_and_scan_tables(transaction)?;
    transaction.execute_batch(
        "CREATE TABLE file_inventory (
            path TEXT PRIMARY KEY NOT NULL,
            size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
            modified_unix_ms INTEGER,
            sha256 TEXT,
            hash_status TEXT NOT NULL CHECK (hash_status IN ('hashed', 'too_large', 'read_error')),
            last_seen_scan_id INTEGER REFERENCES scan_runs(id)
        ) WITHOUT ROWID;",
    )?;
    create_knowledge_tables(transaction)?;
    insert_project_id(transaction)?;
    Ok(())
}

fn create_project_and_scan_tables(transaction: &rusqlite::Transaction<'_>) -> Result<()> {
    transaction.execute_batch(
        "CREATE TABLE project_metadata (
            singleton_id INTEGER PRIMARY KEY CHECK (singleton_id = 1),
            project_id TEXT NOT NULL UNIQUE CHECK (length(project_id) = 36)
        );
        CREATE TABLE scan_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            observed_at_unix_ms INTEGER NOT NULL,
            complete INTEGER NOT NULL CHECK (complete IN (0, 1))
        );",
    )?;
    Ok(())
}

fn create_knowledge_tables(transaction: &rusqlite::Transaction<'_>) -> Result<()> {
    transaction.execute_batch(
        "CREATE TABLE knowledge_records (
            id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36),
            record_type TEXT NOT NULL CHECK (record_type IN ('fact', 'decision', 'inference', 'question')),
            statement TEXT NOT NULL CHECK (length(trim(statement)) > 0),
            status TEXT NOT NULL CHECK (status IN ('current', 'superseded', 'withdrawn')),
            created_at_unix_ms INTEGER NOT NULL,
            supersedes_id TEXT REFERENCES knowledge_records(id)
        );
        CREATE INDEX knowledge_records_order ON knowledge_records(created_at_unix_ms, id);
        CREATE TABLE evidence_references (
            id TEXT PRIMARY KEY NOT NULL CHECK (length(id) = 36),
            provenance_kind TEXT NOT NULL CHECK (provenance_kind IN ('project_file', 'human_input')),
            source_path TEXT,
            sha256 TEXT,
            hash_status TEXT CHECK (hash_status IN ('hashed', 'too_large')),
            captured_at_unix_ms INTEGER NOT NULL,
            CHECK (
                (provenance_kind = 'project_file' AND source_path IS NOT NULL)
                OR (provenance_kind = 'human_input' AND source_path IS NULL AND sha256 IS NULL)
            )
        );
        CREATE TABLE knowledge_evidence (
            knowledge_id TEXT NOT NULL REFERENCES knowledge_records(id),
            evidence_id TEXT NOT NULL REFERENCES evidence_references(id),
            position INTEGER NOT NULL CHECK (position >= 0),
            PRIMARY KEY (knowledge_id, evidence_id),
            UNIQUE (knowledge_id, position)
        );",
    )?;
    Ok(())
}

fn insert_project_id(transaction: &rusqlite::Transaction<'_>) -> Result<()> {
    transaction.execute(
        "INSERT INTO project_metadata (singleton_id, project_id) VALUES (1, ?1)",
        [uuid::Uuid::new_v4().to_string()],
    )?;
    Ok(())
}

fn validate_schema(connection: &Connection) -> Result<()> {
    let found: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if found != i64::from(SCHEMA_VERSION) {
        return Err(TramuriError::UnsupportedSchema {
            found,
            supported: SCHEMA_VERSION,
        });
    }
    Ok(())
}

pub(crate) fn load_inventory(connection: &Connection) -> Result<Vec<StoredFile>> {
    let mut statement = connection.prepare(
        "SELECT path, size_bytes, modified_unix_ms, sha256, hash_status, last_seen_scan_id
         FROM file_inventory ORDER BY path",
    )?;
    let rows = statement.query_map([], stored_file_from_row)?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(TramuriError::from)
}

pub(crate) fn load_inventory_file(
    connection: &Connection,
    path: &str,
) -> Result<Option<StoredFile>> {
    connection
        .query_row(
            "SELECT path, size_bytes, modified_unix_ms, sha256, hash_status, last_seen_scan_id
             FROM file_inventory WHERE path = ?1",
            [path],
            stored_file_from_row,
        )
        .optional()
        .map_err(TramuriError::from)
}

pub(crate) fn latest_scan(connection: &Connection) -> Result<Option<ScanObservation>> {
    connection
        .query_row(
            "SELECT id, complete FROM scan_runs ORDER BY id DESC LIMIT 1",
            [],
            |row| {
                Ok(ScanObservation {
                    id: row.get(0)?,
                    complete: row.get::<_, i64>(1)? != 0,
                })
            },
        )
        .optional()
        .map_err(TramuriError::from)
}

fn stored_file_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredFile> {
    let size_bytes: i64 = row.get(1)?;
    let hash_status_value: String = row.get(4)?;
    let hash_status = HashStatus::from_storage_value(&hash_status_value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            4,
            rusqlite::types::Type::Text,
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown stored hash status: {hash_status_value}"),
            )
            .into(),
        )
    })?;
    Ok(StoredFile {
        path: row.get(0)?,
        size_bytes: u64::try_from(size_bytes)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, size_bytes))?,
        modified_unix_ms: row.get(2)?,
        sha256: row.get(3)?,
        hash_status,
        last_seen_scan_id: row.get(5)?,
    })
}

pub(crate) fn persist_scan(
    connection: &mut Connection,
    files: &[StoredFile],
    deleted_paths: &[String],
    complete: bool,
    observed_at_unix_ms: i64,
) -> Result<i64> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute(
        "INSERT INTO scan_runs (observed_at_unix_ms, complete) VALUES (?1, ?2)",
        params![observed_at_unix_ms, if complete { 1_i64 } else { 0_i64 }],
    )?;
    let scan_id = transaction.last_insert_rowid();
    if complete {
        transaction.execute("DELETE FROM file_inventory", [])?;
    }

    for file in files
        .iter()
        .filter(|file| file.hash_status != HashStatus::ReadError)
    {
        let size_bytes = i64::try_from(file.size_bytes)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, i64::MAX))?;
        transaction.execute(
            "INSERT INTO file_inventory
                (path, size_bytes, modified_unix_ms, sha256, hash_status, last_seen_scan_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(path) DO UPDATE SET
                 size_bytes = excluded.size_bytes,
                 modified_unix_ms = excluded.modified_unix_ms,
                 sha256 = excluded.sha256,
                 hash_status = excluded.hash_status,
                 last_seen_scan_id = excluded.last_seen_scan_id",
            params![
                file.path,
                size_bytes,
                file.modified_unix_ms,
                file.sha256,
                file.hash_status.as_storage_value(),
                scan_id,
            ],
        )?;
    }

    if complete {
        for path in deleted_paths {
            transaction.execute("DELETE FROM file_inventory WHERE path = ?1", [path])?;
        }
    }

    transaction.commit()?;
    Ok(scan_id)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{StoredFile, load_inventory, persist_scan};
    use crate::{HashStatus, TramuriError};

    fn inventory_database() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory database should open");
        connection
            .execute_batch(
                "CREATE TABLE scan_runs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    observed_at_unix_ms INTEGER NOT NULL,
                    complete INTEGER NOT NULL CHECK (complete IN (0, 1))
                );
                CREATE TABLE file_inventory (
                    path TEXT PRIMARY KEY NOT NULL,
                    size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
                    modified_unix_ms INTEGER,
                    sha256 TEXT,
                    hash_status TEXT NOT NULL CHECK (hash_status IN ('hashed', 'too_large', 'read_error')),
                    last_seen_scan_id INTEGER REFERENCES scan_runs(id)
                ) WITHOUT ROWID;",
            )
            .expect("inventory table should be created");
        connection
    }

    #[test]
    fn persisted_hash_statuses_round_trip_without_storing_read_errors() {
        let mut connection = inventory_database();
        let files = [
            StoredFile {
                path: "hashed.txt".to_owned(),
                size_bytes: 4,
                modified_unix_ms: Some(1),
                sha256: Some("abcd".to_owned()),
                hash_status: HashStatus::Hashed,
                last_seen_scan_id: None,
            },
            StoredFile {
                path: "large.bin".to_owned(),
                size_bytes: 32 * 1024 * 1024 + 1,
                modified_unix_ms: Some(2),
                sha256: None,
                hash_status: HashStatus::TooLarge,
                last_seen_scan_id: None,
            },
            StoredFile {
                path: "unreadable.txt".to_owned(),
                size_bytes: 0,
                modified_unix_ms: None,
                sha256: None,
                hash_status: HashStatus::ReadError,
                last_seen_scan_id: None,
            },
        ];

        persist_scan(&mut connection, &files, &[], true, 1_700_000_000_000)
            .expect("scan should persist");
        let loaded = load_inventory(&connection).expect("inventory should load");

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].hash_status, HashStatus::Hashed);
        assert_eq!(loaded[1].hash_status, HashStatus::TooLarge);
        assert_eq!(loaded[0].path, "hashed.txt");
        assert_eq!(loaded[1].path, "large.bin");
    }

    #[test]
    fn unknown_database_hash_status_returns_a_conversion_error() {
        let connection = inventory_database();
        connection
            .pragma_update(None, "ignore_check_constraints", true)
            .expect("test should be able to bypass the schema check");
        connection
            .execute(
                "INSERT INTO file_inventory (path, size_bytes, modified_unix_ms, sha256, hash_status)
                 VALUES ('unknown.txt', 1, NULL, NULL, 'future_status')",
                [],
            )
            .expect("invalid value should be inserted for this diagnostic test");

        let error = load_inventory(&connection).expect_err("unknown status must not be accepted");

        assert!(matches!(
            error,
            TramuriError::Sqlite(rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                _
            ))
        ));
    }
}
