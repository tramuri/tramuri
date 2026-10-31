use std::fs;

use tempfile::TempDir;
use tramuri_core::{
    KnowledgeFilter, KnowledgeFreshness, KnowledgeInput, KnowledgeProvenanceKind, KnowledgeStatus,
    KnowledgeType, init, knowledge_add, knowledge_list, knowledge_withdraw, scan,
};

fn initialized_project() -> TempDir {
    let project = TempDir::new().expect("project should be created");
    let source = project.path().join("docs/architecture.md");
    fs::create_dir_all(source.parent().expect("source parent should exist"))
        .expect("source parent should be created");
    fs::write(&source, "Rust core with a shared SDK boundary\n").expect("source should be written");
    init(project.path()).expect("project should initialize");
    scan(project.path()).expect("source should be inventoried");
    project
}

fn file_input(kind: KnowledgeType, statement: &str) -> KnowledgeInput {
    KnowledgeInput {
        kind,
        statement: statement.to_owned(),
        sources: vec!["docs/architecture.md".to_owned()],
        human_input: false,
        supersedes: None,
    }
}

#[test]
fn knowledge_record_and_evidence_survive_database_reopen() {
    let project = initialized_project();
    let created = knowledge_add(
        project.path(),
        file_input(KnowledgeType::Fact, "The core is implemented in Rust."),
    )
    .expect("knowledge should persist");

    // Every API operation opens its own connection, modeling a later session.
    let records = knowledge_list(project.path(), KnowledgeFilter::default())
        .expect("knowledge should be retrievable");

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, created.id);
    assert_eq!(records[0].statement, "The core is implemented in Rust.");
    assert!(records[0].provenance[0].sha256.is_some());
}

#[test]
fn knowledge_records_keep_type_provenance_and_supersession_history() {
    let project = initialized_project();
    let first = knowledge_add(
        project.path(),
        file_input(KnowledgeType::Decision, "Use SQLite for the local store."),
    )
    .expect("initial decision should persist");
    let mut replacement = file_input(KnowledgeType::Decision, "Keep SQLite as the local store.");
    replacement.supersedes = Some(first.id.clone());
    let second =
        knowledge_add(project.path(), replacement).expect("explicit replacement should persist");

    let history = knowledge_list(
        project.path(),
        KnowledgeFilter {
            kind: Some(KnowledgeType::Decision),
            include_history: true,
        },
    )
    .expect("history should be available");

    assert_eq!(history.len(), 2);
    assert_eq!(history[0].status, KnowledgeStatus::Superseded);
    assert_eq!(history[1].status, KnowledgeStatus::Current);
    assert_eq!(history[1].supersedes.as_deref(), Some(first.id.as_str()));
    assert_eq!(history[1].kind, KnowledgeType::Decision);
    assert_eq!(
        history[1].provenance[0].path.as_deref(),
        Some("docs/architecture.md")
    );
    assert_eq!(history[1].freshness, KnowledgeFreshness::Current);
    assert_eq!(second.id, history[1].id);
}

#[test]
fn conflicting_records_remain_visible_until_explicitly_superseded() {
    let project = initialized_project();
    knowledge_add(
        project.path(),
        file_input(KnowledgeType::Decision, "Choose SQLite."),
    )
    .expect("first decision should persist");
    knowledge_add(
        project.path(),
        file_input(KnowledgeType::Decision, "Choose a different database."),
    )
    .expect("conflicting decision should persist independently");

    let records = knowledge_list(project.path(), KnowledgeFilter::default())
        .expect("current records should be listed");

    assert_eq!(records.len(), 2);
    assert!(
        records
            .iter()
            .all(|record| record.status == KnowledgeStatus::Current)
    );
}

#[test]
fn human_input_is_recorded_as_explicit_provenance() {
    let project = initialized_project();
    let record = knowledge_add(
        project.path(),
        KnowledgeInput {
            kind: KnowledgeType::Question,
            statement: "Should the package expose async APIs?".to_owned(),
            sources: Vec::new(),
            human_input: true,
            supersedes: None,
        },
    )
    .expect("explicit human input should be accepted as provenance");

    assert_eq!(record.provenance.len(), 1);
    assert_eq!(
        record.provenance[0].kind,
        KnowledgeProvenanceKind::HumanInput
    );
    assert_eq!(record.freshness, KnowledgeFreshness::Current);
}

#[test]
fn human_only_knowledge_does_not_require_a_scan() {
    let project = TempDir::new().expect("project should be created");
    init(project.path()).expect("project should initialize without source files");
    let record = knowledge_add(
        project.path(),
        KnowledgeInput {
            kind: KnowledgeType::Decision,
            statement: "Treat onboarding as a product workflow.".to_owned(),
            sources: Vec::new(),
            human_input: true,
            supersedes: None,
        },
    )
    .expect("human-only knowledge should persist without scanning");

    assert_eq!(
        record.provenance[0].kind,
        KnowledgeProvenanceKind::HumanInput
    );
    assert_eq!(record.freshness, KnowledgeFreshness::Current);
}

#[test]
fn withdrawn_record_is_retained_in_history_but_hidden_by_default() {
    let project = initialized_project();
    let record = knowledge_add(
        project.path(),
        file_input(
            KnowledgeType::Inference,
            "The repository likely targets Rust 2024.",
        ),
    )
    .expect("inference should persist");

    let withdrawn = knowledge_withdraw(project.path(), &record.id)
        .expect("withdrawal should update lifecycle state");
    let current = knowledge_list(project.path(), KnowledgeFilter::default())
        .expect("current records should be listed");
    let history = knowledge_list(
        project.path(),
        KnowledgeFilter {
            kind: None,
            include_history: true,
        },
    )
    .expect("withdrawn record should remain available in history");

    assert_eq!(withdrawn.status, KnowledgeStatus::Withdrawn);
    assert!(current.is_empty());
    assert_eq!(history.len(), 1);
}
