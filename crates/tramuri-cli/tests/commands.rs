use std::fs;
use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

fn tramuri() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tramuri"))
}

#[test]
fn help_is_available() {
    let output = tramuri().arg("--help").output().expect("CLI should launch");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help should be UTF-8");
    assert!(stdout.contains("Usage: tramuri"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("scan"));
    assert!(stdout.contains("mission"));
}

#[test]
fn mission_create_show_and_contract_revision_work_without_scan() {
    let project = tempdir().expect("project directory should be created");
    let path = project.path().to_str().expect("path should be UTF-8");
    let initialized = tramuri()
        .args(["init", path, "--json"])
        .output()
        .expect("CLI should launch");
    assert!(initialized.status.success());

    let criteria = project.path().join("criteria.json");
    fs::write(
        &criteria,
        r#"{"criteria":[{"description":"Customer accepts","required":true}]}"#,
    )
    .expect("criteria fixture should be written");
    let create = tramuri()
        .args([
            "mission",
            "create",
            "--path",
            path,
            "--objective",
            "Deliver integration",
            "--criteria-file",
            criteria.to_str().expect("criteria path should be UTF-8"),
            "--json",
        ])
        .output()
        .expect("CLI should launch");
    assert!(
        create.status.success(),
        "{}",
        String::from_utf8_lossy(&create.stderr)
    );
    let mission: Value = serde_json::from_slice(&create.stdout).expect("Mission should be JSON");
    assert_eq!(mission["state"], "open");
    assert_eq!(mission["success_contract"]["revision"], 1);

    let show = tramuri()
        .args([
            "mission",
            "show",
            "--path",
            path,
            "--mission",
            mission["id"].as_str().expect("Mission should have ID"),
            "--json",
        ])
        .output()
        .expect("CLI should launch");
    assert!(show.status.success());
    let shown: Value = serde_json::from_slice(&show.stdout).expect("show should emit JSON");
    assert_eq!(shown, mission);

    fs::write(
        &criteria,
        r#"{"criteria":[{"description":"Customer accepts","required":true},{"description":"Runbook delivered","required":false}]}"#,
    )
    .expect("revised criteria should be written");
    let revise = tramuri()
        .args([
            "mission",
            "contract",
            "revise",
            "--path",
            path,
            "--mission",
            mission["id"].as_str().expect("Mission should have ID"),
            "--criteria-file",
            criteria.to_str().expect("criteria path should be UTF-8"),
            "--json",
        ])
        .output()
        .expect("CLI should launch");
    assert!(
        revise.status.success(),
        "{}",
        String::from_utf8_lossy(&revise.stderr)
    );
    let revised: Value = serde_json::from_slice(&revise.stdout).expect("revision should be JSON");
    assert_eq!(revised["success_contract"]["revision"], 2);
    assert_eq!(
        revised["success_contract"]["criteria"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let close = tramuri()
        .args([
            "mission",
            "close",
            "--path",
            path,
            "--mission",
            mission["id"].as_str().expect("Mission should have ID"),
            "--json",
        ])
        .output()
        .expect("CLI should launch");
    assert!(close.status.success());
    let closed: Value = serde_json::from_slice(&close.stdout).expect("close should emit JSON");
    assert_eq!(closed["state"], "closed");
    assert!(closed.get("outcome").is_none());
}

#[test]
fn execution_and_evidence_commands_record_without_verifying_outcome() {
    let project = tempdir().expect("project directory should be created");
    let path = project.path().to_str().expect("path should be UTF-8");
    assert!(
        tramuri()
            .args(["init", path])
            .output()
            .unwrap()
            .status
            .success()
    );
    let criteria = project.path().join("criteria.json");
    fs::write(
        &criteria,
        r#"{"criteria":[{"description":"Review completed"}]}"#,
    )
    .unwrap();
    let created = tramuri()
        .args([
            "mission",
            "create",
            "--path",
            path,
            "--objective",
            "Review",
            "--criteria-file",
            criteria.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(created.status.success());
    let mission: Value = serde_json::from_slice(&created.stdout).unwrap();
    let recorded = tramuri()
        .args([
            "execution",
            "record",
            "--path",
            path,
            "--mission",
            mission["id"].as_str().unwrap(),
            "--contract-revision",
            "1",
            "--actor-kind",
            "human",
            "--status",
            "completed",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        recorded.status.success(),
        "{}",
        String::from_utf8_lossy(&recorded.stderr)
    );
    let execution: Value = serde_json::from_slice(&recorded.stdout).unwrap();
    assert_eq!(execution["state"], "completed");
    assert!(execution.get("outcome").is_none());
    let evidence = tramuri()
        .args([
            "evidence",
            "add",
            "--path",
            path,
            "--mission",
            mission["id"].as_str().unwrap(),
            "--execution",
            execution["id"].as_str().unwrap(),
            "--kind",
            "human_attestation",
            "--locator",
            "reviewed-by-operator",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        evidence.status.success(),
        "{}",
        String::from_utf8_lossy(&evidence.stderr)
    );
    let evidence: Value = serde_json::from_slice(&evidence.stdout).unwrap();
    assert_eq!(evidence["availability"], "unverified");
}

#[test]
fn init_and_scan_emit_machine_readable_json() {
    let project = tempdir().expect("project directory should be created");
    fs::write(project.path().join("hello.txt"), "hello\n").expect("fixture should be written");

    let init = tramuri()
        .args([
            "init",
            project.path().to_str().expect("path should be UTF-8"),
            "--json",
        ])
        .output()
        .expect("CLI should launch");
    assert!(init.status.success());
    let init_json: Value = serde_json::from_slice(&init.stdout).expect("init should emit JSON");
    assert_eq!(init_json["schema_version"], 4);
    assert_eq!(init_json["already_initialized"], false);

    let repeated_init = tramuri()
        .args([
            "init",
            project.path().to_str().expect("path should be UTF-8"),
            "--json",
        ])
        .output()
        .expect("CLI should launch");
    assert!(repeated_init.status.success());
    let repeated_init_json: Value =
        serde_json::from_slice(&repeated_init.stdout).expect("repeated init should emit JSON");
    assert_eq!(repeated_init_json["already_initialized"], true);

    let scan = tramuri()
        .args([
            "scan",
            project.path().to_str().expect("path should be UTF-8"),
            "--json",
        ])
        .output()
        .expect("CLI should launch");
    assert!(scan.status.success());
    assert!(scan.stderr.is_empty());
    let scan_json: Value = serde_json::from_slice(&scan.stdout).expect("scan should emit JSON");
    assert_eq!(scan_json["complete"], true);
    assert_eq!(scan_json["files"][0]["path"], "hello.txt");
    assert_eq!(scan_json["added"][0], "hello.txt");

    let repeated_scan = tramuri()
        .args([
            "scan",
            project.path().to_str().expect("path should be UTF-8"),
            "--json",
        ])
        .output()
        .expect("CLI should launch");
    assert!(repeated_scan.status.success());
    let repeated_scan_json: Value =
        serde_json::from_slice(&repeated_scan.stdout).expect("repeated scan should emit JSON");
    assert_eq!(
        repeated_scan_json["added"]
            .as_array()
            .expect("added should be an array")
            .len(),
        0
    );
    assert_eq!(repeated_scan_json["unchanged"][0], "hello.txt");
}

#[test]
fn operational_error_is_nonzero_and_written_to_stderr() {
    let project = tempdir().expect("project directory should be created");
    let output = tramuri()
        .args([
            "scan",
            project.path().to_str().expect("path should be UTF-8"),
            "--json",
        ])
        .output()
        .expect("CLI should launch");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("error should be UTF-8");
    assert!(stderr.contains("tramuri init"));
}

#[test]
fn knowledge_commands_add_list_supersede_and_withdraw_records() {
    let project = tempdir().expect("project directory should be created");
    fs::write(project.path().join("design.md"), "chosen design\n")
        .expect("source should be written");
    let path = project.path().to_str().expect("path should be UTF-8");
    for args in [["init", path, "--json"], ["scan", path, "--json"]] {
        let output = tramuri().args(args).output().expect("CLI should launch");
        assert!(output.status.success());
    }

    let add = tramuri()
        .args([
            "knowledge",
            "add",
            path,
            "--type",
            "decision",
            "--statement",
            "Use SQLite",
            "--source",
            "design.md",
            "--json",
        ])
        .output()
        .expect("CLI should launch");
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let first: Value = serde_json::from_slice(&add.stdout).expect("add should emit JSON");
    assert_eq!(first["type"], "decision");
    assert_eq!(first["freshness"], "current");

    let next = tramuri()
        .args([
            "knowledge",
            "add",
            path,
            "--type",
            "decision",
            "--statement",
            "Keep SQLite",
            "--source",
            "design.md",
            "--supersedes",
            first["id"].as_str().expect("record has id"),
            "--json",
        ])
        .output()
        .expect("CLI should launch");
    assert!(
        next.status.success(),
        "{}",
        String::from_utf8_lossy(&next.stderr)
    );
    let second: Value = serde_json::from_slice(&next.stdout).expect("add should emit JSON");

    let current = tramuri()
        .args(["knowledge", "list", path, "--json"])
        .output()
        .expect("CLI should launch");
    let current: Value = serde_json::from_slice(&current.stdout).expect("list should emit JSON");
    assert_eq!(
        current.as_array().expect("list should be an array").len(),
        1
    );
    assert_eq!(current[0]["id"], second["id"]);

    let withdrawn = tramuri()
        .args([
            "knowledge",
            "withdraw",
            path,
            "--id",
            second["id"].as_str().expect("record has id"),
            "--json",
        ])
        .output()
        .expect("CLI should launch");
    assert!(
        withdrawn.status.success(),
        "{}",
        String::from_utf8_lossy(&withdrawn.stderr)
    );
    let withdrawn: Value = serde_json::from_slice(&withdrawn.stdout).expect("withdraw emits JSON");
    assert_eq!(withdrawn["status"], "withdrawn");
}
