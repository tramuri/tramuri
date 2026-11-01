use std::fs;

use tempfile::tempdir;
use tramuri_core::{
    ActorKind, CriterionInput, EvidenceInput, EvidenceKind, ExecutionInput, ExecutionState,
    MissionInput, TramuriError, evidence_add, evidence_get, evidence_list, execution_get,
    execution_list, execution_record, init, mission_create, scan,
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

#[test]
fn retrieval_preserves_execution_and_evidence_fields_and_reports_empty_or_missing_records() {
    let root = tempdir().expect("tempdir");
    let first_mission = mission(root.path());
    let second_mission = mission(root.path());
    let execution = execution(root.path(), &first_mission.id, ExecutionState::Blocked);
    let evidence = evidence_add(
        root.path(),
        EvidenceInput {
            mission_id: first_mission.id.clone(),
            execution_id: execution.id.clone(),
            kind: EvidenceKind::ExternalReference,
            locator: "https://example.invalid/handoff".into(),
        },
    )
    .unwrap();

    assert_eq!(
        execution_get(root.path(), &execution.id).unwrap(),
        execution
    );
    assert_eq!(
        execution_list(root.path(), &first_mission.id).unwrap(),
        vec![execution.clone()]
    );
    assert_eq!(evidence_get(root.path(), &evidence.id).unwrap(), evidence);
    assert_eq!(
        evidence_list(root.path(), &first_mission.id).unwrap().len(),
        1
    );
    assert!(
        execution_list(root.path(), &second_mission.id)
            .unwrap()
            .is_empty()
    );
    assert!(
        evidence_list(root.path(), &second_mission.id)
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        execution_list(root.path(), "missing-mission"),
        Err(TramuriError::MissionNotFound(id)) if id == "missing-mission"
    ));
    assert!(matches!(
        execution_get(root.path(), "missing-execution"),
        Err(TramuriError::ExecutionNotFound(id)) if id == "missing-execution"
    ));
    assert!(matches!(
        evidence_get(root.path(), "missing-evidence"),
        Err(TramuriError::EvidenceNotFound(id)) if id == "missing-evidence"
    ));
}

#[test]
fn retrieval_orders_latest_first_and_uses_id_for_timestamp_ties() {
    let root = tempdir().expect("tempdir");
    let mission = mission(root.path());
    let first = execution(root.path(), &mission.id, ExecutionState::InProgress);
    let second = execution(root.path(), &mission.id, ExecutionState::Completed);
    let third = execution(root.path(), &mission.id, ExecutionState::Blocked);
    let connection =
        rusqlite::Connection::open(root.path().join(".tramuri/database/tramuri.sqlite3")).unwrap();
    connection
        .execute(
            "UPDATE executions SET started_at_unix_ms = CASE id
                WHEN ?1 THEN 10 WHEN ?2 THEN 20 WHEN ?3 THEN 20 END",
            [&first.id, &second.id, &third.id],
        )
        .unwrap();
    let first_id = first.id.clone();
    let mut expected_executions = vec![first, second, third];
    for item in &mut expected_executions {
        item.started_at_unix_ms = if item.id == first_id { 10 } else { 20 };
    }
    expected_executions.sort_by(|left, right| {
        right
            .started_at_unix_ms
            .cmp(&left.started_at_unix_ms)
            .then_with(|| right.id.cmp(&left.id))
    });
    let actual_executions = execution_list(root.path(), &mission.id).unwrap();
    assert_eq!(
        actual_executions
            .iter()
            .map(|item| &item.id)
            .collect::<Vec<_>>(),
        expected_executions
            .iter()
            .map(|item| &item.id)
            .collect::<Vec<_>>()
    );

    let mut evidences = Vec::new();
    for execution in &expected_executions {
        evidences.push(
            evidence_add(
                root.path(),
                EvidenceInput {
                    mission_id: mission.id.clone(),
                    execution_id: execution.id.clone(),
                    kind: EvidenceKind::ExternalReference,
                    locator: format!("https://example.invalid/{}", execution.id),
                },
            )
            .unwrap(),
        );
    }
    connection
        .execute(
            "UPDATE mission_evidence SET captured_at_unix_ms = CASE execution_id
                WHEN ?1 THEN 100 WHEN ?2 THEN 200 WHEN ?3 THEN 200 END",
            [
                expected_executions[0].id.as_str(),
                expected_executions[1].id.as_str(),
                expected_executions[2].id.as_str(),
            ],
        )
        .unwrap();
    for item in &mut evidences {
        item.captured_at_unix_ms = if item.execution_id == expected_executions[0].id {
            100
        } else {
            200
        };
    }
    evidences.sort_by(|left, right| {
        right
            .captured_at_unix_ms
            .cmp(&left.captured_at_unix_ms)
            .then_with(|| right.id.cmp(&left.id))
    });
    let actual_evidences = evidence_list(root.path(), &mission.id).unwrap();
    assert_eq!(
        actual_evidences
            .iter()
            .map(|item| &item.id)
            .collect::<Vec<_>>(),
        evidences.iter().map(|item| &item.id).collect::<Vec<_>>()
    );
}

#[test]
fn retrieval_rejects_invalid_persisted_enum_and_availability_values() {
    let root = tempdir().expect("tempdir");
    let mission = mission(root.path());
    let invalid_actor = execution(root.path(), &mission.id, ExecutionState::InProgress);
    let invalid_state = execution(root.path(), &mission.id, ExecutionState::Completed);
    let connection =
        rusqlite::Connection::open(root.path().join(".tramuri/database/tramuri.sqlite3")).unwrap();
    connection
        .execute_batch("PRAGMA ignore_check_constraints = ON;")
        .unwrap();
    connection
        .execute(
            "UPDATE executions SET actor_kind = 'future_actor' WHERE id = ?1",
            [&invalid_actor.id],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE executions SET state = 'future_state' WHERE id = ?1",
            [&invalid_state.id],
        )
        .unwrap();
    for result in [
        execution_get(root.path(), &invalid_actor.id),
        execution_get(root.path(), &invalid_state.id),
    ] {
        assert!(matches!(
            result,
            Err(TramuriError::Sqlite(
                rusqlite::Error::FromSqlConversionFailure(..)
            ))
        ));
    }
    assert!(matches!(
        execution_list(root.path(), &mission.id),
        Err(TramuriError::Sqlite(
            rusqlite::Error::FromSqlConversionFailure(..)
        ))
    ));

    let evidence = evidence_add(
        root.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution(root.path(), &mission.id, ExecutionState::InProgress).id,
            kind: EvidenceKind::ExternalReference,
            locator: "https://example.invalid/evidence".into(),
        },
    )
    .unwrap();
    let invalid_kind = evidence_add(
        root.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution(root.path(), &mission.id, ExecutionState::InProgress).id,
            kind: EvidenceKind::ExternalReference,
            locator: "https://example.invalid/invalid-kind".into(),
        },
    )
    .unwrap();
    let invalid_hash_status = evidence_add(
        root.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution(root.path(), &mission.id, ExecutionState::InProgress).id,
            kind: EvidenceKind::ExternalReference,
            locator: "https://example.invalid/invalid-hash-status".into(),
        },
    )
    .unwrap();
    connection
        .execute(
            "UPDATE mission_evidence SET kind = 'future_kind' WHERE id = ?1",
            [&invalid_kind.id],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE mission_evidence SET hash_status = 'future_hash_status' WHERE id = ?1",
            [&invalid_hash_status.id],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE mission_evidence SET availability = 'future_availability' WHERE id = ?1",
            [&evidence.id],
        )
        .unwrap();
    for evidence_id in [&invalid_kind.id, &invalid_hash_status.id, &evidence.id] {
        assert!(matches!(
            evidence_get(root.path(), evidence_id),
            Err(TramuriError::Sqlite(
                rusqlite::Error::FromSqlConversionFailure(..)
            ))
        ));
    }
    assert!(matches!(
        evidence_list(root.path(), &mission.id),
        Err(TramuriError::Sqlite(
            rusqlite::Error::FromSqlConversionFailure(..)
        ))
    ));
}

#[test]
fn retrieval_keeps_captured_provenance_without_revalidating_project_files() {
    let root = tempdir().expect("tempdir");
    let mission = mission(root.path());
    let source = root.path().join("deliverable.txt");
    fs::write(&source, "captured content").unwrap();
    scan(root.path()).unwrap();
    let execution = execution(root.path(), &mission.id, ExecutionState::InProgress);
    let evidence = evidence_add(
        root.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution.id,
            kind: EvidenceKind::ProjectFile,
            locator: "deliverable.txt".into(),
        },
    )
    .unwrap();
    fs::write(source, "changed after capture").unwrap();

    let recovered = evidence_get(root.path(), &evidence.id).unwrap();
    assert_eq!(recovered, evidence);
    assert_eq!(recovered.availability, "observed");
}
