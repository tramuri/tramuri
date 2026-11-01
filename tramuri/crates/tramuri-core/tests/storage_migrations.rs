use std::fs;

use rusqlite::{Connection, params};
use tempfile::TempDir;
use tramuri_core::{
    ActorKind, CriterionInput, EvidenceInput, EvidenceKind, ExecutionInput, ExecutionState,
    KnowledgeInput, KnowledgeType, MissionInput, SCHEMA_VERSION, evidence_add, execution_get,
    execution_record, init, knowledge_add, mission_create, mission_get, mission_workflow_get,
    workspace_is_initialized,
};

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
fn workspace_initialization_probe_is_read_only_and_recognizes_legacy_database() {
    let empty = TempDir::new().expect("project directory should be created");
    assert!(!workspace_is_initialized(empty.path()).unwrap());
    assert!(!empty.path().join(".tramuri").exists());

    let legacy = create_v1_project();
    assert!(workspace_is_initialized(legacy.path()).unwrap());
    assert!(legacy.path().join(".tramuri/tramuri.sqlite3").is_file());
    assert!(
        !legacy
            .path()
            .join(".tramuri/database/tramuri.sqlite3")
            .exists()
    );
}

#[test]
fn v1_migration_preserves_inventory_and_creates_knowledge_schema() {
    let project = create_v1_project();

    let report = init(project.path()).expect("init should migrate a v1 database");

    assert!(report.already_initialized);
    assert_eq!(report.schema_version, 9);
    assert_eq!(SCHEMA_VERSION, 9);
    let new_database = project.path().join(".tramuri/database/tramuri.sqlite3");
    assert!(new_database.is_file());
    assert!(!project.path().join(".tramuri/tramuri.sqlite3").exists());
    let connection = Connection::open(&new_database).expect("migrated database should open");
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should be readable");
    assert_eq!(version, 9);
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
        "criterion_assessments",
        "criterion_assessment_evidence",
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
fn v8_to_v9_adds_workflow_tables_without_changing_mission_records() {
    let project = tempfile::tempdir().unwrap();
    init(project.path()).unwrap();
    let mission = mission_create(
        project.path(),
        MissionInput {
            objective: "Preserve existing work".to_owned(),
            context: None,
            criteria: Vec::new(),
        },
    )
    .unwrap();
    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute_batch(
            "DROP TABLE mission_work_perspective_history;
         DROP TABLE mission_workflows;
         PRAGMA user_version = 8;",
        )
        .unwrap();
    drop(connection);

    let report = init(project.path()).expect("v8 should migrate additively");
    assert_eq!(report.schema_version, 9);
    let migrated = mission_get(project.path(), &mission.id).unwrap();
    assert_eq!(migrated.objective, "Preserve existing work");
    assert_eq!(
        mission_workflow_get(project.path(), &mission.id).unwrap(),
        None
    );
}

#[test]
fn v7_migration_adds_readiness_without_changing_mission_records() {
    let project = TempDir::new().expect("project directory should be created");
    init(project.path()).expect("workspace should initialize");
    let mission = mission_create(
        project.path(),
        MissionInput {
            objective: "Continue an existing integration".to_owned(),
            context: None,
            criteria: Vec::new(),
        },
    )
    .expect("mission should be recorded");
    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch("DROP TABLE mission_work_perspective_history; DROP TABLE mission_workflows; DROP TABLE readiness_assessments; PRAGMA user_version = 7;")
        .expect("fixture should represent schema v7");
    drop(connection);

    let report = init(project.path()).expect("v7 database should migrate additively");
    assert_eq!(report.schema_version, SCHEMA_VERSION);
    assert_eq!(
        tramuri_core::mission_get(project.path(), &mission.id).unwrap(),
        mission,
        "the migration must preserve pre-existing Mission data"
    );
    let readiness = tramuri_core::readiness_get(project.path(), &mission.id).unwrap();
    assert_eq!(readiness.state, tramuri_core::ReadinessState::NotAssessed);
}

#[test]
fn v6_to_v7_migration_preserves_execution_and_defaults_perspective_to_none() {
    let project = tempfile::tempdir().expect("project directory should be created");
    init(project.path()).expect("project should initialize");
    let mission = mission_create(
        project.path(),
        MissionInput {
            objective: "Continue a synthetic FDE handoff".to_owned(),
            context: None,
            criteria: vec![tramuri_core::CriterionInput {
                description: "Handoff is accepted".to_owned(),
                required: true,
                evidence_expectation: None,
            }],
        },
    )
    .expect("Mission should be created");
    let execution = execution_record(
        project.path(),
        ExecutionInput {
            mission_id: mission.id,
            contract_revision: 1,
            actor_kind: ActorKind::Human,
            actor_reference: Some("synthetic-fde".to_owned()),
            work_perspective: None,
            state: ExecutionState::Completed,
            notes: None,
        },
    )
    .expect("execution should be recorded");
    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch(
            "DROP TABLE mission_work_perspective_history; DROP TABLE mission_workflows; DROP TABLE readiness_assessments;
             ALTER TABLE executions DROP COLUMN work_perspective;
             PRAGMA user_version = 6;",
        )
        .expect("fixture should represent schema v6");
    drop(connection);

    let report = init(project.path()).expect("v6 state should migrate additively");
    let restored =
        execution_get(project.path(), &execution.id).expect("old execution should remain readable");

    assert_eq!(report.schema_version, 9);
    assert_eq!(restored.id, execution.id);
    assert_eq!(restored.actor_reference.as_deref(), Some("synthetic-fde"));
    assert_eq!(restored.work_perspective, None);
}

#[test]
fn legacy_and_current_database_conflict_preserves_both_files() {
    let project = tempfile::tempdir().expect("project directory should be created");
    let state_dir = project.path().join(".tramuri");
    let database_dir = state_dir.join("database");
    fs::create_dir_all(&database_dir).expect("state directories should be created");
    let legacy = state_dir.join("tramuri.sqlite3");
    let current = database_dir.join("tramuri.sqlite3");
    fs::write(&legacy, b"legacy database sentinel").expect("legacy sentinel should be written");
    fs::write(&current, b"current database sentinel").expect("current sentinel should be written");

    let error = init(project.path()).expect_err("conflicting database paths must fail safely");
    assert!(matches!(
        error,
        tramuri_core::TramuriError::ConflictingStateDatabases { .. }
    ));
    assert_eq!(fs::read(legacy).unwrap(), b"legacy database sentinel");
    assert_eq!(fs::read(current).unwrap(), b"current database sentinel");
}

#[test]
fn legacy_database_moves_to_database_directory_and_keeps_records() {
    let project = tempfile::tempdir().expect("project directory should be created");
    let report = init(project.path()).expect("project should initialize");
    assert!(
        report
            .database_path
            .ends_with(".tramuri/database/tramuri.sqlite3")
    );

    let state_dir = project.path().join(".tramuri");
    let current = state_dir.join("database/tramuri.sqlite3");
    let legacy = state_dir.join("tramuri.sqlite3");
    let connection = Connection::open(&current).expect("current database should open");
    connection
        .execute_batch(
            "CREATE TABLE migration_marker (value TEXT NOT NULL);
             INSERT INTO migration_marker (value) VALUES ('preserved');",
        )
        .expect("migration marker should be written");
    drop(connection);
    fs::rename(&current, &legacy).expect("database should move to legacy location");
    fs::remove_dir(state_dir.join("database")).expect("empty database directory should be removed");

    let migrated = init(project.path()).expect("legacy database should migrate");

    assert!(migrated.already_initialized);
    assert!(
        migrated
            .database_path
            .ends_with(".tramuri/database/tramuri.sqlite3")
    );
    assert!(!legacy.exists());
    let connection =
        Connection::open(migrated.database_path).expect("migrated database should open");
    let preserved: String = connection
        .query_row("SELECT value FROM migration_marker", [], |row| row.get(0))
        .expect("existing database data should remain available");
    assert_eq!(preserved, "preserved");
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("integrity check should run");
    assert_eq!(integrity, "ok");
}

#[test]
fn v2_migration_preserves_existing_knowledge_and_inventory() {
    let project = create_v1_project();
    init(project.path()).expect("v1 project should migrate to current schema");
    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch(
            "DROP TABLE mission_capability_links;
             DROP TABLE mission_knowledge_links;
             DROP TABLE criterion_assessment_evidence;
             DROP TABLE criterion_assessments;
             DROP INDEX success_criteria_assessment_identity;
             DROP TABLE success_criteria;
             DROP TABLE mission_work_perspective_history; DROP TABLE mission_workflows; DROP TABLE readiness_assessments;
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
    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch(
            "DROP TABLE mission_work_perspective_history; DROP TABLE mission_workflows; DROP TABLE readiness_assessments;
             DROP TABLE mission_capability_links;
             DROP TABLE mission_knowledge_links;
             DROP TABLE criterion_assessment_evidence;
             DROP TABLE criterion_assessments;
             DROP INDEX success_criteria_assessment_identity;
             DROP TABLE success_criteria;
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
    let legacy_database = project.path().join(".tramuri/tramuri.sqlite3");
    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&legacy_database).expect("database should open");
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
fn v3_migration_adds_execution_evidence_and_assessment_tables_transactionally() {
    let project = tempfile::tempdir().expect("project directory should be created");
    init(project.path()).expect("project should initialize");
    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch(
            "DROP TABLE mission_work_perspective_history; DROP TABLE mission_workflows; DROP TABLE readiness_assessments;
             ALTER TABLE executions DROP COLUMN work_perspective;
             DROP TABLE mission_capability_links;
             DROP TABLE mission_knowledge_links;
             DROP TABLE criterion_assessment_evidence;
             DROP TABLE criterion_assessments;
             DROP INDEX success_criteria_assessment_identity;
             DROP TABLE mission_evidence; DROP TABLE executions; PRAGMA user_version = 3;",
        )
        .unwrap();
    drop(connection);

    init(project.path()).expect("v3 database should migrate to the current schema");
    let connection = Connection::open(database).unwrap();
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 9);
    for table in [
        "executions",
        "mission_evidence",
        "criterion_assessments",
        "criterion_assessment_evidence",
    ] {
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
    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch("DROP TABLE mission_work_perspective_history; DROP TABLE mission_workflows; DROP TABLE readiness_assessments; DROP TABLE mission_capability_links; DROP TABLE mission_knowledge_links; DROP TABLE criterion_assessment_evidence; DROP TABLE criterion_assessments; DROP INDEX success_criteria_assessment_identity; DROP TABLE mission_evidence; DROP TABLE executions; CREATE TABLE executions (conflicting INTEGER); PRAGMA user_version = 3;")
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

#[test]
fn v4_migration_adds_assessment_tables_and_preserves_existing_mission_data() {
    let project = tempfile::tempdir().expect("project directory should be created");
    init(project.path()).expect("project should initialize");
    let mission = mission_create(
        project.path(),
        MissionInput {
            objective: "Deliver a customer integration".to_owned(),
            context: None,
            criteria: vec![CriterionInput {
                description: "Customer accepts integration".to_owned(),
                required: true,
                evidence_expectation: None,
            }],
        },
    )
    .expect("mission should be created");
    let execution = execution_record(
        project.path(),
        ExecutionInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            actor_kind: ActorKind::Human,
            actor_reference: Some("operator".to_owned()),
            work_perspective: None,
            state: ExecutionState::Completed,
            notes: None,
        },
    )
    .expect("execution should be recorded");
    let evidence = evidence_add(
        project.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution.id.clone(),
            kind: EvidenceKind::HumanAttestation,
            locator: "customer-accepted".to_owned(),
        },
    )
    .expect("evidence should be recorded");
    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch(
            "DROP TABLE mission_work_perspective_history; DROP TABLE mission_workflows; DROP TABLE readiness_assessments;
             ALTER TABLE executions DROP COLUMN work_perspective;
             DROP TABLE mission_capability_links;
             DROP TABLE mission_knowledge_links;
             DROP TABLE criterion_assessment_evidence;
             DROP TABLE criterion_assessments;
             DROP INDEX success_criteria_assessment_identity;
             PRAGMA user_version = 4;",
        )
        .expect("fixture should represent schema v4");
    drop(connection);

    init(project.path()).expect("v4 database should migrate to current schema");

    let connection = Connection::open(database).expect("migrated database should open");
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should be readable");
    assert_eq!(version, 9);
    let preserved: (i64, i64, i64) = connection
        .query_row(
            "SELECT
               (SELECT COUNT(*) FROM missions WHERE id = ?1),
               (SELECT COUNT(*) FROM executions WHERE id = ?2),
               (SELECT COUNT(*) FROM mission_evidence WHERE id = ?3)",
            rusqlite::params![mission.id, execution.id, evidence.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("existing domain records should remain queryable");
    assert_eq!(preserved, (1, 1, 1));
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
fn failed_v4_migration_rolls_back_assessment_schema_and_keeps_version() {
    let project = tempfile::tempdir().expect("project directory should be created");
    init(project.path()).expect("project should initialize");
    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch(
            "DROP TABLE mission_work_perspective_history; DROP TABLE mission_workflows; DROP TABLE readiness_assessments;
             DROP TABLE mission_capability_links;
             DROP TABLE mission_knowledge_links;
             DROP TABLE criterion_assessment_evidence;
             DROP TABLE criterion_assessments;
             DROP INDEX success_criteria_assessment_identity;
             CREATE TABLE criterion_assessments (conflicting INTEGER);
             PRAGMA user_version = 4;",
        )
        .expect("conflicting fixture should be created");
    drop(connection);

    assert!(init(project.path()).is_err());

    let connection = Connection::open(database).expect("database should remain open");
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should remain readable");
    assert_eq!(version, 4);
    let identity_index: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type='index' AND name='success_criteria_assessment_identity'",
            [],
            |row| row.get(0),
        )
        .expect("migration index should be queryable");
    assert_eq!(identity_index, 0);
    let evidence_table: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type='table' AND name='criterion_assessment_evidence'",
            [],
            |row| row.get(0),
        )
        .expect("migration table should be queryable");
    assert_eq!(evidence_table, 0);
}

#[test]
fn v5_migration_adds_relationship_tables_and_preserves_existing_records() {
    let project = tempfile::tempdir().expect("project directory should be created");
    init(project.path()).expect("project should initialize");
    let mission = mission_create(
        project.path(),
        MissionInput {
            objective: "Retain v5 Mission".to_owned(),
            context: None,
            criteria: vec![CriterionInput {
                description: "Existing criterion remains".to_owned(),
                required: true,
                evidence_expectation: None,
            }],
        },
    )
    .expect("Mission should be created");
    let knowledge = knowledge_add(
        project.path(),
        KnowledgeInput {
            kind: KnowledgeType::Decision,
            statement: "Keep this decision through migration".to_owned(),
            sources: Vec::new(),
            human_input: true,
            supersedes: None,
        },
    )
    .expect("Knowledge record should be created");

    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch(
            "DROP TABLE mission_work_perspective_history; DROP TABLE mission_workflows; DROP TABLE readiness_assessments;
             ALTER TABLE executions DROP COLUMN work_perspective;
             DROP TABLE mission_capability_links;
             DROP TABLE mission_knowledge_links;
             PRAGMA user_version = 5;",
        )
        .expect("database fixture should represent v5");
    drop(connection);

    init(project.path()).expect("v5 schema should migrate");
    let connection = Connection::open(&database).expect("migrated database should open");
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should be readable");
    assert_eq!(version, 9);
    let preserved: (String, String) = connection
        .query_row(
            "SELECT m.objective, k.statement FROM missions AS m
             CROSS JOIN knowledge_records AS k WHERE m.id = ?1 AND k.id = ?2",
            rusqlite::params![mission.id, knowledge.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("existing domain records should remain queryable");
    assert_eq!(
        preserved,
        (
            "Retain v5 Mission".to_owned(),
            "Keep this decision through migration".to_owned()
        )
    );
    for table in ["mission_knowledge_links", "mission_capability_links"] {
        let exists: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )
            .expect("migration table should be queryable");
        assert_eq!(exists, 1, "expected migration table {table}");
    }
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("integrity check should run");
    assert_eq!(integrity, "ok");
}

#[test]
fn failed_v5_migration_rolls_back_link_tables_and_version() {
    let project = tempfile::tempdir().expect("project directory should be created");
    init(project.path()).expect("project should initialize");
    let database = project.path().join(".tramuri/database/tramuri.sqlite3");
    let connection = Connection::open(&database).expect("database should open");
    connection
        .execute_batch(
            "DROP TABLE mission_work_perspective_history; DROP TABLE mission_workflows; DROP TABLE readiness_assessments;
             DROP TABLE mission_capability_links;
             DROP TABLE mission_knowledge_links;
             CREATE TABLE mission_capability_links (conflicting INTEGER);
             PRAGMA user_version = 5;",
        )
        .expect("conflicting fixture should be created");
    drop(connection);

    assert!(init(project.path()).is_err());

    let connection = Connection::open(database).expect("database should remain open");
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should remain readable");
    assert_eq!(version, 5);
    let partial_table: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mission_knowledge_links'",
            [],
            |row| row.get(0),
        )
        .expect("link table check should run");
    assert_eq!(
        partial_table, 0,
        "failed migration must roll back created tables"
    );
}
