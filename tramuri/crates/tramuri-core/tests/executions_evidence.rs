use std::fs;

use tempfile::tempdir;
use tramuri_core::{
    ActorKind, CriterionInput, EvidenceInput, EvidenceKind, ExecutionInput, ExecutionState,
    MissionInput, evidence_add, execution_record, init, mission_create, scan,
};

fn mission(root: &std::path::Path) -> tramuri_core::Mission {
    init(root).expect("workspace initializes");
    mission_create(
        root,
        MissionInput {
            objective: "Deliver a product outcome".into(),
            context: None,
            criteria: vec![CriterionInput {
                description: "Reviewed".into(),
                required: true,
                evidence_expectation: None,
            }],
        },
    )
    .expect("mission is created")
}

fn execution(
    root: &std::path::Path,
    mission_id: &str,
    state: ExecutionState,
) -> tramuri_core::Execution {
    execution_record(
        root,
        ExecutionInput {
            mission_id: mission_id.into(),
            contract_revision: 1,
            actor_kind: ActorKind::Human,
            actor_reference: Some("reviewer".into()),
            state,
            notes: None,
        },
    )
    .expect("execution is recorded")
}

#[test]
fn completed_execution_is_not_a_verified_outcome_and_is_persisted() {
    let root = tempdir().expect("tempdir");
    let mission = mission(root.path());
    let item = execution(root.path(), &mission.id, ExecutionState::Completed);
    assert_eq!(item.state, ExecutionState::Completed);
    assert!(item.ended_at_unix_ms.is_some());
    assert_eq!(item.contract_revision, 1);
    assert!(
        !serde_json::to_value(&item)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("outcome")
    );
    let connection =
        rusqlite::Connection::open(root.path().join(".tramuri/database/tramuri.sqlite3")).unwrap();
    let persisted: String = connection
        .query_row(
            "SELECT state FROM executions WHERE id = ?1",
            [&item.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(persisted, "completed");
}

#[test]
fn file_evidence_requires_latest_safe_scanned_file_and_never_stores_contents() {
    let root = tempdir().expect("tempdir");
    let mission = mission(root.path());
    fs::write(root.path().join("result.txt"), "private result contents").unwrap();
    scan(root.path()).expect("scan completes");
    let execution = execution(root.path(), &mission.id, ExecutionState::InProgress);
    let evidence = evidence_add(
        root.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution.id.clone(),
            kind: EvidenceKind::ProjectFile,
            locator: "result.txt".into(),
        },
    )
    .expect("observed source can be referenced");
    assert_eq!(evidence.availability, "observed");
    let connection =
        rusqlite::Connection::open(root.path().join(".tramuri/database/tramuri.sqlite3")).unwrap();
    let stored: String = connection
        .query_row(
            "SELECT locator FROM mission_evidence WHERE id = ?1",
            [&evidence.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored, "result.txt");
    let stored_fields: String = connection
        .query_row(
            "SELECT group_concat(name, ',') FROM pragma_table_info('mission_evidence')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!stored_fields.contains("content"));
    assert!(
        evidence_add(
            root.path(),
            EvidenceInput {
                mission_id: mission.id,
                execution_id: execution.id,
                kind: EvidenceKind::ProjectFile,
                locator: ".env".into()
            }
        )
        .is_err()
    );
    assert!(
        evidence_add(
            root.path(),
            EvidenceInput {
                mission_id: "other-mission".into(),
                execution_id: evidence.execution_id,
                kind: EvidenceKind::HumanAttestation,
                locator: "reviewed".into()
            }
        )
        .is_err()
    );
}

#[test]
fn external_evidence_rejects_credentials_and_secret_bearing_query_strings() {
    let root = tempdir().expect("tempdir");
    let mission = mission(root.path());
    let execution = execution(root.path(), &mission.id, ExecutionState::Completed);
    let result = evidence_add(
        root.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution.id.clone(),
            kind: EvidenceKind::ExternalReference,
            locator: "https://user:token@example.test/report".into(),
        },
    );
    assert!(result.is_err());
    let result = evidence_add(
        root.path(),
        EvidenceInput {
            mission_id: mission.id,
            execution_id: execution.id,
            kind: EvidenceKind::ExternalReference,
            locator: "https://example.test/report?token=secret".into(),
        },
    );
    assert!(result.is_err());
}

#[test]
fn missing_or_unobserved_file_evidence_is_rejected() {
    let root = tempdir().expect("tempdir");
    let mission = mission(root.path());
    let execution = execution(root.path(), &mission.id, ExecutionState::InProgress);
    assert!(
        evidence_add(
            root.path(),
            EvidenceInput {
                mission_id: mission.id,
                execution_id: execution.id,
                kind: EvidenceKind::ProjectFile,
                locator: "unscanned.txt".into()
            }
        )
        .is_err()
    );
}

#[test]
fn changed_file_is_retained_as_changed_and_incomplete_scan_is_unverified() {
    let root = tempdir().expect("tempdir");
    let mission = mission(root.path());
    let source = root.path().join("artifact.txt");
    fs::write(&source, "before").unwrap();
    fs::write(root.path().join("stable.txt"), "stable").unwrap();
    scan(root.path()).unwrap();
    let execution = execution(root.path(), &mission.id, ExecutionState::InProgress);
    fs::write(&source, "substantially changed after the scan").unwrap();
    let changed = evidence_add(
        root.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution.id.clone(),
            kind: EvidenceKind::ProjectFile,
            locator: "artifact.txt".into(),
        },
    )
    .unwrap();
    assert_eq!(changed.availability, "changed");

    let connection =
        rusqlite::Connection::open(root.path().join(".tramuri/database/tramuri.sqlite3")).unwrap();
    connection
        .execute("UPDATE scan_runs SET complete = 0", [])
        .unwrap();
    let incomplete = evidence_add(
        root.path(),
        EvidenceInput {
            mission_id: mission.id,
            execution_id: execution.id,
            kind: EvidenceKind::ProjectFile,
            locator: "stable.txt".into(),
        },
    )
    .unwrap();
    assert_eq!(incomplete.availability, "unverified");
}

#[test]
fn ignored_and_symlink_paths_cannot_be_attached_as_file_evidence() {
    let root = tempdir().expect("tempdir");
    let mission = mission(root.path());
    fs::write(root.path().join(".gitignore"), "ignored.txt\n").unwrap();
    fs::write(root.path().join("ignored.txt"), "not indexed").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink("ignored.txt", root.path().join("link.txt")).unwrap();
    scan(root.path()).unwrap();
    let execution = execution(root.path(), &mission.id, ExecutionState::InProgress);
    for locator in ["ignored.txt", "link.txt"] {
        assert!(
            evidence_add(
                root.path(),
                EvidenceInput {
                    mission_id: mission.id.clone(),
                    execution_id: execution.id.clone(),
                    kind: EvidenceKind::ProjectFile,
                    locator: locator.into()
                }
            )
            .is_err()
        );
    }
}
