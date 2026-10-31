use rusqlite::Connection;
use tempfile::tempdir;
use tramuri_core::{
    CriterionInput, MissionInput, MissionState, init, mission_close, mission_contract_revise,
    mission_create, mission_get,
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

    let connection = Connection::open(workspace.path().join(".tramuri/tramuri.sqlite3"))
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
