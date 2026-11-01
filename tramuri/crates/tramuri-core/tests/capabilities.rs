use std::fs;

use tempfile::tempdir;
use tramuri_core::{
    CapabilityKind, CapabilityState, SCHEMA_VERSION, capability_list, capability_snapshot, init,
    scan, status_snapshot,
};

fn put(root: &std::path::Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn scan_discovers_supported_declarations_without_copying_configuration_values() {
    let project = tempdir().unwrap();
    init(project.path()).unwrap();
    put(
        project.path(),
        ".agents/skills/triage/SKILL.md",
        "---\nname: triage\ndescription: token: never-export-this\n---\nprivate body must not be exported\n",
    );
    put(project.path(), "CLAUDE.md", "Project instructions\n");
    put(
        project.path(),
        ".claude/agents/reviewer.md",
        "---\nname: reviewer\ndescription: Review changes carefully\nmcpServers:\n  - private-server\n---\nprivate agent body\n",
    );
    put(
        project.path(),
        ".claude/settings.json",
        r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"do-not-run-marker"}]}]},"mcpServers":{"private-server":{"command":"private executable","env":{"TOKEN":"never-export-this"}}}}"#,
    );
    put(
        project.path(),
        ".cursor/rules/security.mdc",
        "---\ndescription: Security review\nalwaysApply: true\n---\nbody omitted\n",
    );
    put(
        project.path(),
        ".cursor/mcp.json",
        r#"{"mcpServers":{"cursor-tool":{"url":"https://private.example.invalid","headers":{"Authorization":"never-export-this"}}}}"#,
    );
    put(
        project.path(),
        ".gemini/agents/installer.md",
        "---\nname: installer\ndescription: Prepare a safe installation\n---\n",
    );
    put(
        project.path(),
        ".gemini/settings.json",
        r#"{"hooksConfig":{"enabled":false},"hooks":{"BeforeTool":[{}]},"mcpServers":{"gemini-tool":{"command":"do-not-run-marker"}}}"#,
    );
    put(
        project.path(),
        ".github/copilot-instructions.md",
        "Use the project conventions.\n",
    );
    put(
        project.path(),
        ".github/agents/fde.agent.md",
        "---\nname: fde\ndescription: Customer integration helper\n---\n",
    );
    put(
        project.path(),
        ".github/hooks/session.json",
        r#"{"hooks":{"sessionStart":[{}]}}"#,
    );
    put(
        project.path(),
        ".vscode/mcp.json",
        r#"{"servers":{"workspace-tool":{"command":"do-not-run-marker"}}}"#,
    );
    put(
        project.path(),
        ".codex/hooks.json",
        r#"{"hooks":{"SessionStart":[{}]}}"#,
    );

    let report = scan(project.path()).unwrap();
    assert!(report.complete);
    let snapshot = capability_snapshot(project.path()).unwrap();
    assert_eq!(snapshot.schema, "tramuri.capabilities.v1");
    assert!(snapshot.scan_complete);
    assert!(
        snapshot
            .capabilities
            .iter()
            .any(|item| { item.category == CapabilityKind::Skill && item.name == "triage" })
    );
    assert!(snapshot.capabilities.iter().any(|item| {
        item.category == CapabilityKind::Skill
            && item.name == "triage"
            && item.description.is_none()
    }));
    assert!(
        snapshot.capabilities.iter().any(|item| {
            item.category == CapabilityKind::Agent && item.ecosystem == "claude_code"
        })
    );
    assert!(
        snapshot
            .capabilities
            .iter()
            .any(|item| { item.category == CapabilityKind::Rule && item.ecosystem == "cursor" })
    );
    assert!(snapshot.capabilities.iter().any(|item| {
        item.category == CapabilityKind::Hook && item.ecosystem == "github_copilot"
    }));
    let server = snapshot
        .capabilities
        .iter()
        .find(|item| item.category == CapabilityKind::McpServer && item.name == "private-server")
        .unwrap();
    assert_eq!(server.state, CapabilityState::DeclaredAndRecognized);
    assert_eq!(server.availability, "not_verified");
    let agent = snapshot
        .capabilities
        .iter()
        .find(|item| item.category == CapabilityKind::Agent && item.name == "reviewer")
        .unwrap();
    assert_eq!(agent.dependencies, vec!["private-server"]);
    assert_eq!(agent.relationships.len(), 1);
    assert_eq!(agent.relationships[0].target_id, server.id);
    assert_eq!(agent.relationships[0].provenance_path, agent.path);
    let encoded = serde_json::to_string(&snapshot).unwrap();
    assert!(!encoded.contains("never-export-this"));
    assert!(!encoded.contains("private executable"));
    assert!(!encoded.contains("private.example.invalid"));
    assert!(!encoded.contains("private agent body"));
    assert!(!encoded.contains("do-not-run-marker"));
    assert_eq!(SCHEMA_VERSION, 8);

    let first_ids: Vec<_> = snapshot
        .capabilities
        .iter()
        .map(|item| item.id.clone())
        .collect();
    let artifact_path = project
        .path()
        .join(".tramuri/discovery/latest-capabilities.json");
    let artifact_before = fs::read(&artifact_path).unwrap();
    scan(project.path()).unwrap();
    let rescanned = capability_snapshot(project.path()).unwrap();
    let second_ids: Vec<_> = rescanned
        .capabilities
        .iter()
        .map(|item| item.id.clone())
        .collect();
    assert_eq!(first_ids, second_ids);
    assert_eq!(artifact_before, fs::read(&artifact_path).unwrap());
    let filtered = capability_list(project.path(), Some(CapabilityKind::Skill)).unwrap();
    assert!(
        filtered
            .capabilities
            .iter()
            .all(|item| item.category == CapabilityKind::Skill)
    );
    assert_eq!(
        status_snapshot(project.path())
            .unwrap()
            .capability_summary
            .total,
        15
    );
}

#[test]
fn changed_and_removed_capabilities_follow_complete_scans() {
    let project = tempdir().unwrap();
    init(project.path()).unwrap();
    let skill = ".agents/skills/triage/SKILL.md";
    put(
        project.path(),
        skill,
        "---\nname: triage\ndescription: first\n---\n",
    );
    scan(project.path()).unwrap();
    let before = capability_snapshot(project.path())
        .unwrap()
        .capabilities
        .remove(0);

    put(
        project.path(),
        skill,
        "---\nname: triage\ndescription: changed\n---\n",
    );
    scan(project.path()).unwrap();
    let after = capability_snapshot(project.path())
        .unwrap()
        .capabilities
        .remove(0);
    assert_eq!(before.id, after.id);
    assert_ne!(before.sha256, after.sha256);
    assert_eq!(after.description.as_deref(), Some("changed"));

    fs::remove_file(project.path().join(skill)).unwrap();
    scan(project.path()).unwrap();
    assert!(
        capability_snapshot(project.path())
            .unwrap()
            .capabilities
            .is_empty()
    );
}

#[test]
fn invalid_or_oversized_descriptors_are_explicit_and_missing_scan_is_empty() {
    let project = tempdir().unwrap();
    init(project.path()).unwrap();
    let initial = capability_snapshot(project.path()).unwrap();
    assert!(!initial.scan_complete);
    assert!(initial.capabilities.is_empty());

    put(project.path(), ".mcp.json", "not json");
    put(
        project.path(),
        ".claude/skills/broken/SKILL.md",
        "---\nname: broken\nmissing closing marker\n",
    );
    let oversized = project.path().join(".cursor/mcp.json");
    fs::create_dir_all(oversized.parent().unwrap()).unwrap();
    fs::write(&oversized, vec![b' '; 1024 * 1024 + 1]).unwrap();
    scan(project.path()).unwrap();
    let snapshot = capability_snapshot(project.path()).unwrap();
    assert!(
        snapshot
            .capabilities
            .iter()
            .any(|item| item.state == CapabilityState::Invalid)
    );
    assert!(
        snapshot
            .capabilities
            .iter()
            .any(|item| item.state == CapabilityState::UnavailableReadError)
    );
    assert!(snapshot.capabilities.iter().any(|item| {
        item.limitation
            .as_deref()
            .is_some_and(|text| text.contains("closing"))
    }));
}

#[test]
fn ignored_agent_configuration_is_not_read_or_catalogued() {
    let project = tempdir().unwrap();
    init(project.path()).unwrap();
    put(project.path(), ".gitignore", ".claude/\n");
    put(
        project.path(),
        ".claude/skills/private/SKILL.md",
        "---\nname: private\ndescription: must not appear\n---\n",
    );
    scan(project.path()).unwrap();
    let snapshot = capability_snapshot(project.path()).unwrap();
    assert!(
        !snapshot
            .capabilities
            .iter()
            .any(|item| item.name == "private")
    );
}

#[cfg(unix)]
#[test]
fn incomplete_scan_retains_unseen_capabilities() {
    use std::os::unix::fs::symlink;

    let project = tempdir().unwrap();
    let outside = tempdir().unwrap();
    init(project.path()).unwrap();
    let skill = ".agents/skills/triage/SKILL.md";
    put(project.path(), skill, "---\nname: triage\n---\n");
    scan(project.path()).unwrap();
    fs::remove_file(project.path().join(skill)).unwrap();
    symlink(outside.path(), project.path().join("linked-directory")).unwrap();

    let report = scan(project.path()).unwrap();
    assert!(!report.complete);
    let snapshot = capability_snapshot(project.path()).unwrap();
    assert!(!snapshot.scan_complete);
    assert!(
        snapshot
            .capabilities
            .iter()
            .any(|item| item.name == "triage")
    );
    assert!(
        snapshot
            .warnings
            .iter()
            .any(|warning| warning.contains("incomplete"))
    );
}
