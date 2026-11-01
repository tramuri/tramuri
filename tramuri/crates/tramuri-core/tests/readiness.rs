use tempfile::tempdir;
use tramuri_core::{
    ActorKind, CriterionInput, ExecutionInput, ExecutionState, MissionInput, OutcomeState,
    ReadinessAssessmentInput, ReadinessDecision, ReadinessState, evidence_list, execution_list,
    init, mission_contract_revise, mission_create, outcome_get, readiness_assess, readiness_get,
    readiness_history,
};

fn new_mission(path: &std::path::Path) -> tramuri_core::Mission {
    mission_create(
        path,
        MissionInput {
            objective: "Explore customer onboarding friction".to_owned(),
            context: None,
            criteria: Vec::new(),
        },
    )
    .unwrap()
}

#[test]
fn readiness_is_persisted_and_activity_is_selected_explicitly() {
    let workspace = tempdir().unwrap();
    init(workspace.path()).unwrap();
    let mission = new_mission(workspace.path());
    let initial = readiness_get(workspace.path(), &mission.id).unwrap();
    assert_eq!(initial.state, ReadinessState::NotAssessed);
    assert_eq!(initial.activity, None);

    let assessment = readiness_assess(
        workspace.path(),
        ReadinessAssessmentInput {
            mission_id: mission.id.clone(),
            activity: "discover".to_owned(),
            decision: ReadinessDecision::Ready,
            criteria_applied: vec!["Investigation objective is understood".to_owned()],
            gaps: Vec::new(),
            blockers: Vec::new(),
            accepted_uncertainties: vec!["Sample size is not known yet".to_owned()],
            actor_reference: Some("researcher".to_owned()),
        },
    )
    .unwrap();

    let current = readiness_get(workspace.path(), &mission.id).unwrap();
    assert_eq!(current.state, ReadinessState::Ready);
    assert_eq!(current.activity.as_deref(), Some("discover"));
    assert_eq!(current.assessment.as_ref().unwrap(), &assessment);
    assert_eq!(
        readiness_history(workspace.path(), &mission.id).unwrap(),
        [assessment]
    );
}

#[test]
fn critical_blockers_cannot_be_hidden_by_a_ready_decision() {
    let workspace = tempdir().unwrap();
    init(workspace.path()).unwrap();
    let mission = new_mission(workspace.path());

    let hidden_blocker = readiness_assess(
        workspace.path(),
        ReadinessAssessmentInput {
            mission_id: mission.id.clone(),
            activity: "deploy".to_owned(),
            decision: ReadinessDecision::Ready,
            criteria_applied: vec!["Deployment prerequisites reviewed".to_owned()],
            gaps: Vec::new(),
            blockers: vec!["Production credentials are not approved".to_owned()],
            accepted_uncertainties: Vec::new(),
            actor_reference: None,
        },
    );
    assert!(hidden_blocker.is_err());

    let no_blocker = readiness_assess(
        workspace.path(),
        ReadinessAssessmentInput {
            mission_id: mission.id.clone(),
            activity: "deploy".to_owned(),
            decision: ReadinessDecision::Blocked,
            criteria_applied: Vec::new(),
            gaps: Vec::new(),
            blockers: Vec::new(),
            accepted_uncertainties: Vec::new(),
            actor_reference: None,
        },
    );
    assert!(no_blocker.is_err());

    let no_checks = readiness_assess(
        workspace.path(),
        ReadinessAssessmentInput {
            mission_id: mission.id,
            activity: "deploy".to_owned(),
            decision: ReadinessDecision::Ready,
            criteria_applied: Vec::new(),
            gaps: Vec::new(),
            blockers: Vec::new(),
            accepted_uncertainties: Vec::new(),
            actor_reference: None,
        },
    );
    assert!(no_checks.is_err());
}

#[test]
fn relevant_contract_change_marks_readiness_stale_without_erasing_history() {
    let workspace = tempdir().unwrap();
    init(workspace.path()).unwrap();
    let mission = new_mission(workspace.path());
    readiness_assess(
        workspace.path(),
        ReadinessAssessmentInput {
            mission_id: mission.id.clone(),
            activity: "prototype".to_owned(),
            decision: ReadinessDecision::Ready,
            criteria_applied: vec!["Prototype goal is understood".to_owned()],
            gaps: Vec::new(),
            blockers: Vec::new(),
            accepted_uncertainties: Vec::new(),
            actor_reference: None,
        },
    )
    .unwrap();

    mission_contract_revise(
        workspace.path(),
        &mission.id,
        vec![CriterionInput {
            description: "Prototype demonstrates the onboarding change".to_owned(),
            required: true,
            evidence_expectation: None,
        }],
    )
    .unwrap();

    let current = readiness_get(workspace.path(), &mission.id).unwrap();
    assert_eq!(current.state, ReadinessState::Stale);
    assert_eq!(current.assessed_state, Some(ReadinessDecision::Ready));
    assert_eq!(
        readiness_history(workspace.path(), &mission.id)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn readiness_is_advisory_and_does_not_block_execution_or_change_outcome() {
    let workspace = tempdir().unwrap();
    init(workspace.path()).unwrap();
    let mission = new_mission(workspace.path());
    readiness_assess(
        workspace.path(),
        ReadinessAssessmentInput {
            mission_id: mission.id.clone(),
            activity: "discover".to_owned(),
            decision: ReadinessDecision::Blocked,
            criteria_applied: Vec::new(),
            gaps: Vec::new(),
            blockers: vec!["Waiting for customer interview".to_owned()],
            accepted_uncertainties: Vec::new(),
            actor_reference: None,
        },
    )
    .unwrap();

    let execution = tramuri_core::execution_record(
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
    assert_eq!(execution.state, ExecutionState::Completed);
    assert_eq!(
        execution_list(workspace.path(), &mission.id).unwrap().len(),
        1
    );
    assert!(
        evidence_list(workspace.path(), &mission.id)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        outcome_get(workspace.path(), &mission.id, None)
            .unwrap()
            .state,
        OutcomeState::Pending
    );
}
