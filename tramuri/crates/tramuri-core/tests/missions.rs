use std::fs;

use rusqlite::Connection;
use tempfile::tempdir;
use tramuri_core::{
    ActorKind, CriterionAssessmentInput, CriterionAssessmentResult, CriterionInput, EvidenceInput,
    EvidenceKind, ExecutionInput, ExecutionState, MissionInput, MissionState, OutcomeState,
    VerifierKind, WorkPerspective, evidence_add, execution_record, execution_transition, init,
    mission_close, mission_contract_revise, mission_create, mission_get, outcome_assess,
    outcome_get, scan,
};

fn criteria(descriptions: &[&str]) -> Vec<CriterionInput> {
    descriptions
        .iter()
        .map(|description| CriterionInput {
            description: (*description).to_owned(),
            required: true,
            evidence_expectation: None,
        })
        .collect()
}

#[test]
fn mission_can_be_created_and_retrieved_without_git_or_scan() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("empty workspace should initialize");

    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Validate customer onboarding flow".to_owned(),
            context: None,
            criteria: criteria(&["Customer accepts the workflow"]),
        },
    )
    .expect("Mission should not require a repository scan");

    assert_eq!(mission.objective, "Validate customer onboarding flow");
    assert_eq!(mission.state, MissionState::Open);
    assert_eq!(mission.success_contract.revision, 1);
    assert_eq!(mission.success_contract.criteria.len(), 1);
    assert_eq!(mission_get(workspace.path(), &mission.id).unwrap(), mission);
}

#[test]
fn execution_perspective_is_persisted_separately_from_actor_identity() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Validate a customer integration".to_owned(),
            context: None,
            criteria: criteria(&["Customer acceptance recorded"]),
        },
    )
    .expect("Mission should be created");
    let execution = execution_record(
        workspace.path(),
        ExecutionInput {
            mission_id: mission.id,
            contract_revision: 1,
            actor_kind: ActorKind::Agent,
            actor_reference: Some("Codex".to_owned()),
            work_perspective: Some(WorkPerspective::Fde),
            state: ExecutionState::Completed,
            notes: None,
        },
    )
    .expect("execution should be recorded");

    assert_eq!(execution.actor_kind, ActorKind::Agent);
    assert_eq!(execution.actor_reference.as_deref(), Some("Codex"));
    assert_eq!(execution.work_perspective, Some(WorkPerspective::Fde));
}

#[test]
fn execution_transition_sets_end_time_and_keeps_terminal_state_immutable() {
    let workspace = tempdir().unwrap();
    init(workspace.path()).unwrap();
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Complete synthetic Discovery".to_owned(),
            context: None,
            criteria: Vec::new(),
        },
    )
    .unwrap();
    let execution = execution_record(
        workspace.path(),
        ExecutionInput {
            mission_id: mission.id,
            contract_revision: 1,
            actor_kind: ActorKind::Human,
            actor_reference: None,
            work_perspective: None,
            state: ExecutionState::InProgress,
            notes: None,
        },
    )
    .unwrap();
    assert_eq!(execution.ended_at_unix_ms, None);
    let completed =
        execution_transition(workspace.path(), &execution.id, ExecutionState::Completed).unwrap();
    assert_eq!(completed.state, ExecutionState::Completed);
    assert!(completed.ended_at_unix_ms.unwrap() >= execution.started_at_unix_ms);
    assert_eq!(
        execution_transition(workspace.path(), &execution.id, ExecutionState::Completed).unwrap(),
        completed
    );
    assert!(matches!(
        execution_transition(workspace.path(), &execution.id, ExecutionState::Failed),
        Err(tramuri_core::TramuriError::InvalidExecutionTransition { .. })
    ));
}

#[test]
fn contract_revision_preserves_prior_success_contract() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Deploy integration".to_owned(),
            context: Some("FDE engagement".to_owned()),
            criteria: criteria(&["Integration is accepted"]),
        },
    )
    .expect("Mission should be created");

    let revised = mission_contract_revise(
        workspace.path(),
        &mission.id,
        criteria(&["Integration is accepted", "Runbook is delivered"]),
    )
    .expect("contract should be revised");

    assert_eq!(revised.success_contract.revision, 2);
    assert_eq!(revised.success_contract.id, mission.success_contract.id);
    assert_eq!(revised.success_contract.criteria.len(), 2);
    assert_eq!(mission_get(workspace.path(), &mission.id).unwrap(), revised);

    let connection = Connection::open(workspace.path().join(".tramuri/database/tramuri.sqlite3"))
        .expect("Mission database should reopen");
    let prior_criteria: Vec<String> = connection
        .prepare(
            "SELECT description FROM success_criteria
             WHERE contract_id = ?1 AND contract_revision = 1 ORDER BY position",
        )
        .expect("prior criteria query should prepare")
        .query_map([&mission.success_contract.id], |row| row.get(0))
        .expect("prior criteria should query")
        .collect::<std::result::Result<Vec<_>, _>>()
        .expect("all prior criteria should be readable");
    assert_eq!(prior_criteria, ["Integration is accepted"]);
}

#[test]
fn mission_rejects_blank_objective_and_contract_without_required_criterion() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");

    let blank_objective = mission_create(
        workspace.path(),
        MissionInput {
            objective: "  ".to_owned(),
            context: None,
            criteria: criteria(&["A required criterion"]),
        },
    );
    assert!(blank_objective.is_err());

    let no_required = mission_create(
        workspace.path(),
        MissionInput {
            objective: "A valid objective".to_owned(),
            context: None,
            criteria: vec![CriterionInput {
                description: "Optional only".to_owned(),
                required: false,
                evidence_expectation: None,
            }],
        },
    );
    assert!(no_required.is_err());
}

#[test]
fn exploratory_mission_without_criteria_stays_pending_until_reviewed() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Understand why trial users abandon setup".to_owned(),
            context: None,
            criteria: Vec::new(),
        },
    )
    .expect("discovery should start before success criteria are complete");

    assert!(mission.success_contract.criteria.is_empty());
    assert_eq!(
        outcome_get(workspace.path(), &mission.id, None)
            .unwrap()
            .state,
        OutcomeState::Pending,
        "a contract with no criteria must never be vacuously verified"
    );
}

#[test]
fn closing_a_mission_changes_only_its_lifecycle_state() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Review release readiness".to_owned(),
            context: None,
            criteria: criteria(&["Reviewer accepts the release"]),
        },
    )
    .expect("Mission should be created");

    let closed = mission_close(workspace.path(), &mission.id).expect("Mission should close");

    assert_eq!(closed.state, MissionState::Closed);
    assert_eq!(closed.success_contract, mission.success_contract);
}

#[test]
fn completed_execution_stays_pending_until_required_criteria_are_assessed() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Complete product onboarding".to_owned(),
            context: None,
            criteria: criteria(&["Customer accepts the workflow"]),
        },
    )
    .expect("Mission should be created");
    execution_record(
        workspace.path(),
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
    .expect("completed execution should be recorded");

    let outcome = outcome_get(workspace.path(), &mission.id, None)
        .expect("outcome should be pending without criterion assessment");

    assert_eq!(outcome.state, OutcomeState::Pending);
    assert!(outcome.criteria[0].assessment_history.is_empty());
}

#[test]
fn met_assessment_with_same_revision_evidence_verifies_and_preserves_history() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Validate customer onboarding".to_owned(),
            context: None,
            criteria: criteria(&["Customer accepts the workflow"]),
        },
    )
    .expect("Mission should be created");
    let execution = execution_record(
        workspace.path(),
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
        workspace.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution.id,
            kind: EvidenceKind::HumanAttestation,
            locator: "customer-approved-onboarding".to_owned(),
        },
    )
    .expect("attestation reference should be recorded");

    let assessment = outcome_assess(
        workspace.path(),
        CriterionAssessmentInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            criterion_id: mission.success_contract.criteria[0].id.clone(),
            result: CriterionAssessmentResult::Met,
            evidence_ids: vec![evidence.id.clone()],
            verifier_kind: VerifierKind::Human,
            verifier_reference: "product-owner".to_owned(),
            explanation: Some("Reviewed customer acceptance".to_owned()),
        },
    )
    .expect("assessment should be appended");
    let verified = outcome_get(workspace.path(), &mission.id, Some(1))
        .expect("revision one outcome should be available");

    assert_eq!(assessment.result, CriterionAssessmentResult::Met);
    assert_eq!(assessment.evidence_ids, [evidence.id]);
    assert!(assessment.assessed_at_unix_ms > 0);
    assert_eq!(verified.state, OutcomeState::Verified);
    assert_eq!(
        verified.criteria[0].assessment_history.last().unwrap().id,
        assessment.id
    );

    let revised = mission_contract_revise(
        workspace.path(),
        &mission.id,
        criteria(&["Customer accepts the workflow", "Runbook is delivered"]),
    )
    .expect("new contract revision should be created");
    let current = outcome_get(workspace.path(), &mission.id, None)
        .expect("latest revision outcome should be available");

    assert_eq!(revised.success_contract.revision, 2);
    assert_eq!(current.state, OutcomeState::Pending);
    assert_eq!(verified.state, OutcomeState::Verified);
    assert_eq!(
        outcome_get(workspace.path(), &mission.id, Some(1))
            .unwrap()
            .criteria[0]
            .assessment_history
            .last()
            .unwrap()
            .id,
        assessment.id
    );
}

#[test]
fn failed_required_criterion_takes_precedence_over_other_pending_criteria() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Complete a customer integration".to_owned(),
            context: None,
            criteria: criteria(&["Integration works", "Customer accepts integration"]),
        },
    )
    .expect("Mission should be created");
    let execution = execution_record(
        workspace.path(),
        ExecutionInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            actor_kind: ActorKind::Human,
            actor_reference: None,
            work_perspective: None,
            state: ExecutionState::Completed,
            notes: None,
        },
    )
    .unwrap();
    let evidence = evidence_add(
        workspace.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution.id,
            kind: EvidenceKind::ExternalReference,
            locator: "https://example.test/integration-check".to_owned(),
        },
    )
    .unwrap();
    outcome_assess(
        workspace.path(),
        CriterionAssessmentInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            criterion_id: mission.success_contract.criteria[0].id.clone(),
            result: CriterionAssessmentResult::NotMet,
            evidence_ids: vec![evidence.id],
            verifier_kind: VerifierKind::Mechanism,
            verifier_reference: "acceptance-review".to_owned(),
            explanation: None,
        },
    )
    .unwrap();

    let outcome = outcome_get(workspace.path(), &mission.id, Some(1)).unwrap();

    assert_eq!(outcome.state, OutcomeState::NotAchieved);
    assert!(outcome.criteria[1].assessment_history.is_empty());
}

#[test]
fn met_assessment_requires_evidence() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Review delivery".to_owned(),
            context: None,
            criteria: criteria(&["Delivery accepted"]),
        },
    )
    .unwrap();
    let input = CriterionAssessmentInput {
        mission_id: mission.id.clone(),
        contract_revision: 1,
        criterion_id: mission.success_contract.criteria[0].id.clone(),
        result: CriterionAssessmentResult::Met,
        evidence_ids: Vec::new(),
        verifier_kind: VerifierKind::Human,
        verifier_reference: "reviewer".to_owned(),
        explanation: None,
    };

    assert!(matches!(
        outcome_assess(workspace.path(), input),
        Err(tramuri_core::TramuriError::MissingAssessmentEvidence)
    ));

    let missing_verifier = outcome_assess(
        workspace.path(),
        CriterionAssessmentInput {
            mission_id: mission.id,
            contract_revision: 1,
            criterion_id: mission.success_contract.criteria[0].id.clone(),
            result: CriterionAssessmentResult::Unverified,
            evidence_ids: Vec::new(),
            verifier_kind: VerifierKind::Human,
            verifier_reference: "   ".to_owned(),
            explanation: None,
        },
    );
    assert!(matches!(
        missing_verifier,
        Err(tramuri_core::TramuriError::EmptyVerifierReference)
    ));
}

#[test]
fn assessment_rejects_evidence_from_a_different_contract_revision() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Deliver an integration".to_owned(),
            context: None,
            criteria: criteria(&["Integration works"]),
        },
    )
    .unwrap();
    let revised = mission_contract_revise(
        workspace.path(),
        &mission.id,
        criteria(&["Integration works", "Runbook exists"]),
    )
    .unwrap();
    let execution = execution_record(
        workspace.path(),
        ExecutionInput {
            mission_id: mission.id.clone(),
            contract_revision: 2,
            actor_kind: ActorKind::Human,
            actor_reference: None,
            work_perspective: None,
            state: ExecutionState::Completed,
            notes: None,
        },
    )
    .unwrap();
    let evidence = evidence_add(
        workspace.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution.id,
            kind: EvidenceKind::HumanAttestation,
            locator: "reviewed-runbook".to_owned(),
        },
    )
    .unwrap();
    let assessment = outcome_assess(
        workspace.path(),
        CriterionAssessmentInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            criterion_id: mission.success_contract.criteria[0].id.clone(),
            result: CriterionAssessmentResult::Met,
            evidence_ids: vec![evidence.id],
            verifier_kind: VerifierKind::Human,
            verifier_reference: "reviewer".to_owned(),
            explanation: None,
        },
    );

    assert_eq!(revised.success_contract.revision, 2);
    assert!(assessment.is_err());
}

#[test]
fn a_later_assessment_becomes_current_without_erasing_prior_assessment() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Review a product workflow".to_owned(),
            context: None,
            criteria: criteria(&["Workflow is accepted"]),
        },
    )
    .unwrap();
    let execution = execution_record(
        workspace.path(),
        ExecutionInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            actor_kind: ActorKind::Human,
            actor_reference: None,
            work_perspective: None,
            state: ExecutionState::Completed,
            notes: None,
        },
    )
    .unwrap();
    let evidence = evidence_add(
        workspace.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution.id,
            kind: EvidenceKind::HumanAttestation,
            locator: "reviewer-attestation".to_owned(),
        },
    )
    .unwrap();
    let first = outcome_assess(
        workspace.path(),
        CriterionAssessmentInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            criterion_id: mission.success_contract.criteria[0].id.clone(),
            result: CriterionAssessmentResult::Unverified,
            evidence_ids: Vec::new(),
            verifier_kind: VerifierKind::Human,
            verifier_reference: "reviewer".to_owned(),
            explanation: Some("Need confirmation".to_owned()),
        },
    )
    .unwrap();
    assert_eq!(
        outcome_get(workspace.path(), &mission.id, Some(1))
            .unwrap()
            .state,
        OutcomeState::Pending
    );
    let second = outcome_assess(
        workspace.path(),
        CriterionAssessmentInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            criterion_id: mission.success_contract.criteria[0].id.clone(),
            result: CriterionAssessmentResult::Met,
            evidence_ids: vec![evidence.id],
            verifier_kind: VerifierKind::Human,
            verifier_reference: "product-owner".to_owned(),
            explanation: None,
        },
    )
    .unwrap();
    let outcome = outcome_get(workspace.path(), &mission.id, Some(1)).unwrap();
    let connection =
        Connection::open(workspace.path().join(".tramuri/database/tramuri.sqlite3")).unwrap();
    let history_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM criterion_assessments WHERE criterion_id = ?1",
            [&mission.success_contract.criteria[0].id],
            |row| row.get(0),
        )
        .unwrap();

    assert_ne!(first.id, second.id);
    assert_eq!(history_count, 2);
    assert_eq!(outcome.state, OutcomeState::Verified);
    assert_eq!(outcome.criteria[0].assessment_history.len(), 2);
    assert_eq!(
        outcome.criteria[0].assessment_history[0].result,
        CriterionAssessmentResult::Unverified
    );
    assert_eq!(
        outcome.criteria[0].assessment_history.last().unwrap().id,
        second.id
    );
}

#[test]
fn changed_file_evidence_no_longer_supports_a_verified_outcome() {
    let workspace = tempdir().expect("workspace should be created");
    fs::write(workspace.path().join("acceptance.txt"), "accepted").unwrap();
    init(workspace.path()).expect("workspace should initialize");
    scan(workspace.path()).expect("source evidence should be inventoried");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Validate customer acceptance".to_owned(),
            context: None,
            criteria: criteria(&["Customer accepts the workflow"]),
        },
    )
    .unwrap();
    let execution = execution_record(
        workspace.path(),
        ExecutionInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            actor_kind: ActorKind::Human,
            actor_reference: None,
            work_perspective: None,
            state: ExecutionState::Completed,
            notes: None,
        },
    )
    .unwrap();
    let evidence = evidence_add(
        workspace.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution.id,
            kind: EvidenceKind::ProjectFile,
            locator: "acceptance.txt".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(evidence.availability, "observed");
    outcome_assess(
        workspace.path(),
        CriterionAssessmentInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            criterion_id: mission.success_contract.criteria[0].id.clone(),
            result: CriterionAssessmentResult::Met,
            evidence_ids: vec![evidence.id],
            verifier_kind: VerifierKind::Human,
            verifier_reference: "reviewer".to_owned(),
            explanation: None,
        },
    )
    .unwrap();
    assert_eq!(
        outcome_get(workspace.path(), &mission.id, Some(1))
            .unwrap()
            .state,
        OutcomeState::Verified
    );

    fs::write(
        workspace.path().join("acceptance.txt"),
        "content changed after review",
    )
    .unwrap();
    scan(workspace.path()).expect("changed source should be observed");

    assert_eq!(
        outcome_get(workspace.path(), &mission.id, Some(1))
            .unwrap()
            .state,
        OutcomeState::Pending
    );
}

#[test]
fn optional_criteria_do_not_block_a_verified_outcome() {
    let workspace = tempdir().expect("workspace should be created");
    init(workspace.path()).expect("workspace should initialize");
    let mission = mission_create(
        workspace.path(),
        MissionInput {
            objective: "Deliver an integration".to_owned(),
            context: None,
            criteria: vec![
                CriterionInput {
                    description: "Integration accepted".to_owned(),
                    required: true,
                    evidence_expectation: None,
                },
                CriterionInput {
                    description: "Optional handover notes".to_owned(),
                    required: false,
                    evidence_expectation: None,
                },
            ],
        },
    )
    .unwrap();
    let execution = execution_record(
        workspace.path(),
        ExecutionInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            actor_kind: ActorKind::Human,
            actor_reference: None,
            work_perspective: None,
            state: ExecutionState::Completed,
            notes: None,
        },
    )
    .unwrap();
    let evidence = evidence_add(
        workspace.path(),
        EvidenceInput {
            mission_id: mission.id.clone(),
            execution_id: execution.id,
            kind: EvidenceKind::HumanAttestation,
            locator: "customer-confirmed".to_owned(),
        },
    )
    .unwrap();
    outcome_assess(
        workspace.path(),
        CriterionAssessmentInput {
            mission_id: mission.id.clone(),
            contract_revision: 1,
            criterion_id: mission.success_contract.criteria[0].id.clone(),
            result: CriterionAssessmentResult::Met,
            evidence_ids: vec![evidence.id],
            verifier_kind: VerifierKind::Human,
            verifier_reference: "reviewer".to_owned(),
            explanation: None,
        },
    )
    .unwrap();

    let outcome = outcome_get(workspace.path(), &mission.id, Some(1)).unwrap();

    assert_eq!(outcome.state, OutcomeState::Verified);
    assert!(outcome.criteria[1].assessment_history.is_empty());
}
