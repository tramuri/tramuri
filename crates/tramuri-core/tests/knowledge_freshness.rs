use std::fs;

use tempfile::TempDir;
use tramuri_core::{
    KnowledgeFilter, KnowledgeFreshness, KnowledgeInput, KnowledgeType, init, knowledge_add,
    knowledge_list, scan,
};

#[cfg(unix)]
use std::os::unix::fs::symlink;

fn initialized_project() -> TempDir {
    let project = TempDir::new().expect("project should be created");
    fs::write(
        project.path().join("facts.md"),
        "The observed value is 10.\n",
    )
    .expect("source should be written");
    init(project.path()).expect("project should initialize");
    scan(project.path()).expect("source should be inventoried");
    project
}

fn add_fact(project: &TempDir) {
    knowledge_add(
        project.path(),
        KnowledgeInput {
            kind: KnowledgeType::Fact,
            statement: "The observed value is 10.".to_owned(),
            sources: vec!["facts.md".to_owned()],
            human_input: false,
            supersedes: None,
        },
    )
    .expect("fact should be linked to its source");
}

fn only_freshness(project: &TempDir) -> KnowledgeFreshness {
    let records = knowledge_list(project.path(), KnowledgeFilter::default())
        .expect("knowledge should be retrievable");
    assert_eq!(records.len(), 1);
    records[0].freshness
}

#[test]
fn changed_source_marks_evidence_stale() {
    let project = initialized_project();
    add_fact(&project);
    fs::write(
        project.path().join("facts.md"),
        "The observed value is 11.\n",
    )
    .expect("source should change");
    scan(project.path()).expect("changed source should be rescanned");

    assert_eq!(only_freshness(&project), KnowledgeFreshness::Changed);
}

#[test]
fn absent_source_is_missing_only_after_a_complete_scan() {
    let project = initialized_project();
    add_fact(&project);
    fs::remove_file(project.path().join("facts.md")).expect("source should be removed");
    let report = scan(project.path()).expect("project should scan");

    assert!(report.complete);
    assert_eq!(report.deleted, ["facts.md"]);
    assert_eq!(only_freshness(&project), KnowledgeFreshness::Missing);
}

#[test]
fn ignored_existing_source_is_unavailable_not_missing() {
    let project = initialized_project();
    add_fact(&project);
    fs::write(project.path().join(".gitignore"), "facts.md\n")
        .expect("ignore rule should be written");
    scan(project.path()).expect("project should scan");

    assert!(project.path().join("facts.md").is_file());
    assert_eq!(only_freshness(&project), KnowledgeFreshness::Unavailable);
}

#[cfg(unix)]
#[test]
fn incomplete_scan_does_not_mark_unseen_source_current_or_missing() {
    let project = initialized_project();
    add_fact(&project);
    fs::remove_file(project.path().join("facts.md")).expect("source should be removed");
    let outside = TempDir::new().expect("outside directory should be created");
    fs::write(outside.path().join("external.txt"), "outside project")
        .expect("outside file should be written");
    symlink(
        outside.path().join("external.txt"),
        project.path().join("external-link.txt"),
    )
    .expect("symlink should be created");

    let report = scan(project.path()).expect("partial scan should return a report");

    assert!(!report.complete);
    assert!(report.deleted.is_empty());
    assert_eq!(only_freshness(&project), KnowledgeFreshness::Unverified);
}

#[test]
fn unindexed_or_sensitive_paths_cannot_be_added_as_file_evidence() {
    let project = initialized_project();
    fs::write(project.path().join(".env"), "SECRET=do not index")
        .expect("sensitive file should be written");
    scan(project.path()).expect("sensitive file should be excluded");

    let result = knowledge_add(
        project.path(),
        KnowledgeInput {
            kind: KnowledgeType::Fact,
            statement: "A claim with excluded evidence.".to_owned(),
            sources: vec![".env".to_owned()],
            human_input: false,
            supersedes: None,
        },
    );

    assert!(
        result.is_err(),
        "sensitive sources must not be stored as evidence"
    );
}
