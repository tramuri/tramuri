use std::fs;

use tempfile::tempdir;
use tramuri_core::{
    ActorKind, CapabilityKind, CriterionInput, KnowledgeInput, KnowledgeType, MissionInput,
    RelationshipFreshness, capability_list, init, knowledge_add, mission_capability_link,
    mission_capability_unlink, mission_create, mission_knowledge_link, mission_knowledge_unlink,
    mission_relationships, mission_resume, scan, status_snapshot,
};

fn mission(path: &std::path::Path, objective: &str) -> String {
    mission_create(
        path,
        MissionInput {
            objective: objective.to_owned(),
            context: Some("private context".to_owned()),
            criteria: vec![CriterionInput {
                description: "Review the handoff".to_owned(),
                required: true,
                evidence_expectation: None,
            }],
        },
    )
    .unwrap()
    .id
}

#[test]
fn explicit_links_are_shared_idempotent_provenanced_and_mission_scoped() {
    let project = tempdir().unwrap();
    init(project.path()).unwrap();
    let first = mission(project.path(), "First product workflow");
    let second = mission(project.path(), "Second customer integration");
    let knowledge = knowledge_add(
        project.path(),
        KnowledgeInput {
            kind: KnowledgeType::Decision,
            statement: "Use the customer's test tenant only".to_owned(),
            sources: Vec::new(),
            human_input: true,
            supersedes: None,
        },
    )
    .unwrap();

    let first_links = mission_knowledge_link(
        project.path(),
        &first,
        &knowledge.id,
        ActorKind::Human,
        Some("fde-1"),
    )
    .unwrap();
    let first_provenance = first_links.knowledge[0].provenance.clone();
    let repeated = mission_knowledge_link(
        project.path(),
        &first,
        &knowledge.id,
        ActorKind::Agent,
        Some("agent-session-2"),
    )
    .unwrap();
    assert_eq!(repeated.knowledge.len(), 1);
    assert_eq!(repeated.knowledge[0].provenance, first_provenance);

    let shared = mission_knowledge_link(
        project.path(),
        &second,
        &knowledge.id,
        ActorKind::Human,
        Some("fde-2"),
    )
    .unwrap();
    assert_eq!(shared.knowledge.len(), 1);
    assert_eq!(shared.knowledge[0].knowledge.id, knowledge.id);
    assert_eq!(
        mission_relationships(project.path(), &first)
            .unwrap()
            .knowledge
            .len(),
        1
    );
    assert!(mission_knowledge_unlink(project.path(), &first, &knowledge.id).unwrap());
    assert!(!mission_knowledge_unlink(project.path(), &first, &knowledge.id).unwrap());
    assert_eq!(
        mission_relationships(project.path(), &first)
            .unwrap()
            .knowledge
            .len(),
        0
    );
    assert_eq!(
        tramuri_core::knowledge_list(
            project.path(),
            tramuri_core::KnowledgeFilter {
                kind: None,
                include_history: true,
            }
        )
        .unwrap()
        .len(),
        1,
        "unlinking must preserve the Knowledge record"
    );
}

#[test]
fn capability_links_track_hash_changes_and_complete_removals() {
    let project = tempdir().unwrap();
    init(project.path()).unwrap();
    let mission_id = mission(project.path(), "Prepare reliable release");
    let skill = project.path().join(".agents/skills/release/SKILL.md");
    fs::create_dir_all(skill.parent().unwrap()).unwrap();
    fs::write(
        &skill,
        "---\nname: release\ndescription: Verify release notes\n---\n",
    )
    .unwrap();
    scan(project.path()).unwrap();
    let capability = capability_list(project.path(), Some(CapabilityKind::Skill))
        .unwrap()
        .capabilities
        .remove(0);
    let linked = mission_capability_link(
        project.path(),
        &mission_id,
        &capability.id,
        ActorKind::Human,
        Some("release-owner"),
    )
    .unwrap();
    assert_eq!(
        linked.capabilities[0].freshness,
        RelationshipFreshness::Current
    );
    assert_eq!(
        linked.capabilities[0].sha256_at_link,
        capability.sha256.unwrap()
    );

    fs::write(
        &skill,
        "---\nname: release\ndescription: Review package metadata\n---\n",
    )
    .unwrap();
    scan(project.path()).unwrap();
    let changed = mission_relationships(project.path(), &mission_id).unwrap();
    assert_eq!(
        changed.capabilities[0].freshness,
        RelationshipFreshness::Changed
    );
    assert!(changed.capabilities[0].capability.is_some());

    fs::remove_file(skill).unwrap();
    scan(project.path()).unwrap();
    let removed = mission_relationships(project.path(), &mission_id).unwrap();
    assert_eq!(
        removed.capabilities[0].freshness,
        RelationshipFreshness::Missing
    );
    assert!(removed.capabilities[0].capability.is_none());
    assert!(mission_capability_unlink(project.path(), &mission_id, &capability.id).unwrap());
    assert!(
        mission_relationships(project.path(), &mission_id)
            .unwrap()
            .capabilities
            .is_empty()
    );
}

#[test]
fn resume_includes_only_explicit_context_and_preserves_outcome_state() {
    let project = tempdir().unwrap();
    init(project.path()).unwrap();
    let mission_id = mission(project.path(), "Synthetic customer handoff");
    let knowledge = knowledge_add(
        project.path(),
        KnowledgeInput {
            kind: KnowledgeType::Decision,
            statement: "Never use production credentials".to_owned(),
            sources: Vec::new(),
            human_input: true,
            supersedes: None,
        },
    )
    .unwrap();
    mission_knowledge_link(
        project.path(),
        &mission_id,
        &knowledge.id,
        ActorKind::Human,
        None,
    )
    .unwrap();

    let status = status_snapshot(project.path()).unwrap();
    assert!(
        !serde_json::to_string(&status)
            .unwrap()
            .contains("Never use production credentials")
    );
    let resumed = mission_resume(project.path(), &mission_id).unwrap();
    assert_eq!(resumed.mission.id, mission_id);
    assert_eq!(resumed.relationships.knowledge.len(), 1);
    assert_eq!(
        resumed.relationships.knowledge[0].knowledge.statement,
        knowledge.statement
    );
    assert_eq!(
        resumed.mission.outcome.state,
        tramuri_core::OutcomeState::Pending
    );
    assert!(
        serde_json::to_value(&resumed)
            .unwrap()
            .get("relationships")
            .is_some()
    );
}

#[cfg(unix)]
#[test]
fn incomplete_scan_does_not_claim_a_linked_capability_is_current_or_missing() {
    use std::os::unix::fs::symlink;

    let project = tempdir().unwrap();
    let outside = tempdir().unwrap();
    init(project.path()).unwrap();
    let mission_id = mission(project.path(), "Partial scan test");
    let skill = project.path().join(".agents/skills/release/SKILL.md");
    fs::create_dir_all(skill.parent().unwrap()).unwrap();
    fs::write(&skill, "---\nname: release\n---\n").unwrap();
    scan(project.path()).unwrap();
    let capability = capability_list(project.path(), Some(CapabilityKind::Skill))
        .unwrap()
        .capabilities
        .remove(0);
    mission_capability_link(
        project.path(),
        &mission_id,
        &capability.id,
        ActorKind::Human,
        None,
    )
    .unwrap();
    fs::remove_file(&skill).unwrap();
    symlink(outside.path(), project.path().join("external-link")).unwrap();
    let report = scan(project.path()).unwrap();
    assert!(!report.complete);
    let result = mission_relationships(project.path(), &mission_id)
        .unwrap()
        .capabilities
        .remove(0);
    assert_eq!(result.freshness, RelationshipFreshness::Unverified);
    assert!(matches!(
        mission_capability_link(
            project.path(),
            &mission_id,
            &capability.id,
            ActorKind::Human,
            None,
        ),
        Err(tramuri_core::TramuriError::CapabilityNotFresh(_))
    ));
}
