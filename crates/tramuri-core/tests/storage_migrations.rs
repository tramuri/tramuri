use std::fs;

use rusqlite::{Connection, params};
use tempfile::TempDir;
use tramuri_core::{SCHEMA_VERSION, init};

fn create_v1_project() -> TempDir {
    let project = TempDir::new().expect("project directory should be created");
    let state = project.path().join(".tramuri");
    fs::create_dir(&state).expect("state directory should be created");
    let connection =
        Connection::open(state.join("tramuri.sqlite3")).expect("database should be created");
    connection
        .execute_batch(
            "CREATE TABLE file_inventory (
                path TEXT PRIMARY KEY NOT NULL,
                size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
                modified_unix_ms INTEGER,
                sha256 TEXT,
                hash_status TEXT NOT NULL CHECK (hash_status IN ('hashed', 'too_large', 'read_error'))
            ) WITHOUT ROWID;
            PRAGMA user_version = 1;",
        )
        .expect("v1 schema should be created");
    connection
        .execute(
            "INSERT INTO file_inventory (path, size_bytes, modified_unix_ms, sha256, hash_status)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                "legacy.txt",
                23_i64,
                1_700_000_000_000_i64,
                "old-digest",
                "hashed"
            ],
        )
        .expect("legacy inventory row should be inserted");
    project
}

#[test]
fn v1_migration_preserves_inventory_and_creates_knowledge_schema() {
    let project = create_v1_project();

    let report = init(project.path()).expect("init should migrate a v1 database");

    assert!(report.already_initialized);
    assert_eq!(report.schema_version, 4);
    assert_eq!(SCHEMA_VERSION, 4);
    let connection = Connection::open(project.path().join(".tramuri/tramuri.sqlite3"))
        .expect("migrated database should open");
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should be readable");
    assert_eq!(version, 4);
    let row: (String, i64, Option<i64>, Option<String>, String) = connection
        .query_row(
            "SELECT path, size_bytes, modified_unix_ms, sha256, hash_status
             FROM file_inventory WHERE path = 'legacy.txt'",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .expect("legacy inventory must survive migration unchanged");
    assert_eq!(
        row,
        (
            "legacy.txt".to_owned(),
            23,
            Some(1_700_000_000_000),
            Some("old-digest".to_owned()),
            "hashed".to_owned()
        )
    );
    for table in [
        "project_metadata",
        "scan_runs",
        "knowledge_records",
        "evidence_references",
        "knowledge_evidence",
        "missions",
        "success_contracts",
        "success_criteria",
        "executions",
        "mission_evidence",
    ] {
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .expect("schema table should be queryable");
        assert_eq!(count, 1, "migration should create {table}");
    }
}

#[test]
fn v2_migration_preserves_existing_knowledge_and_inventory() {
    let project = create_v1_project();
    init(project.path()).expect("v1 project should migrate to current schema");
    let database = project.path().join(".tramuri/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch(
            "DROP TABLE success_criteria;
             DROP TABLE mission_evidence;
             DROP TABLE executions;
             DROP TABLE success_contracts;
             DROP TABLE missions;
             INSERT INTO scan_runs (observed_at_unix_ms, complete) VALUES (1700000000001, 1);
             INSERT INTO knowledge_records
                (id, record_type, statement, status, created_at_unix_ms)
             VALUES ('11111111-1111-4111-8111-111111111111', 'decision', 'Keep local data', 'current', 1700000000001);
             PRAGMA user_version = 2;",
        )
        .expect("fixture should be reduced to a schema-v2 database");
    drop(connection);

    init(project.path()).expect("schema-v2 project should migrate");
    let connection = Connection::open(database).expect("migrated database should open");
    let statement: String = connection
        .query_row(
            "SELECT statement FROM knowledge_records WHERE id = '11111111-1111-4111-8111-111111111111'",
            [],
            |row| row.get(0),
        )
        .expect("knowledge must remain intact");
    assert_eq!(statement, "Keep local data");
    let inventory_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM file_inventory WHERE path = 'legacy.txt'",
            [],
            |row| row.get(0),
        )
        .expect("inventory should remain queryable");
    assert_eq!(inventory_count, 1);
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("integrity check should run");
    assert_eq!(integrity, "ok");
    let foreign_key_failures: i64 = connection
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .expect("foreign key check should run");
    assert_eq!(foreign_key_failures, 0);
}

#[test]
fn failed_v2_migration_rolls_back_new_tables_and_preserves_version() {
    let project = tempfile::tempdir().expect("project directory should be created");
    init(project.path()).expect("project should initialize");
    let database = project.path().join(".tramuri/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch(
            "DROP TABLE success_criteria;
             DROP TABLE mission_evidence;
             DROP TABLE executions;
             DROP TABLE success_contracts;
             DROP TABLE missions;
             CREATE TABLE missions (conflicting INTEGER);
             PRAGMA user_version = 2;",
        )
        .expect("conflicting schema should be created");
    drop(connection);

    assert!(
        init(project.path()).is_err(),
        "migration conflict must fail"
    );
    let connection = Connection::open(database).expect("database should remain open");
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should remain readable");
    assert_eq!(version, 2);
    let conflict_columns: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('missions')",
            [],
            |row| row.get(0),
        )
        .expect("pre-existing conflicting table should remain");
    assert_eq!(conflict_columns, 1);
    let contract_exists: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'success_contracts'",
            [],
            |row| row.get(0),
        )
        .expect("migration table should be queryable");
    assert_eq!(contract_exists, 0, "partial DDL must roll back");
}

#[test]
fn failed_v1_migration_rolls_back_version_and_inventory() {
    let project = create_v1_project();
    let database = project.path().join(".tramuri/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch("CREATE TABLE knowledge_records (invalid INTEGER);")
        .expect("conflicting table should be created");
    drop(connection);

    assert!(
        init(project.path()).is_err(),
        "migration conflict must fail"
    );

    let connection = Connection::open(database).expect("database should remain open");
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should be readable after rollback");
    assert_eq!(version, 1);
    let row_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM file_inventory", [], |row| row.get(0))
        .expect("legacy inventory should remain queryable");
    assert_eq!(row_count, 1);
    let migration_table_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'project_metadata'",
            [],
            |row| row.get(0),
        )
        .expect("migration schema should be queryable");
    assert_eq!(
        migration_table_count, 0,
        "failed migration must roll back DDL"
    );
}

#[test]
fn v3_migration_adds_execution_evidence_tables_transactionally() {
    let project = tempfile::tempdir().expect("project directory should be created");
    init(project.path()).expect("project should initialize");
    let database = project.path().join(".tramuri/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch(
            "DROP TABLE mission_evidence; DROP TABLE executions; PRAGMA user_version = 3;",
        )
        .unwrap();
    drop(connection);

    init(project.path()).expect("v3 database should migrate to v4");
    let connection = Connection::open(database).unwrap();
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 4);
    for table in ["executions", "mission_evidence"] {
        let exists: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(exists, 1);
    }
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
}

#[test]
fn failed_v3_migration_rolls_back_and_keeps_schema_version() {
    let project = tempfile::tempdir().expect("project directory should be created");
    init(project.path()).expect("project should initialize");
    let database = project.path().join(".tramuri/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch("DROP TABLE mission_evidence; DROP TABLE executions; CREATE TABLE executions (conflicting INTEGER); PRAGMA user_version = 3;")
        .unwrap();
    drop(connection);

    assert!(init(project.path()).is_err());
    let connection = Connection::open(database).unwrap();
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 3);
    let tables: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mission_evidence'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(tables, 0);
}
