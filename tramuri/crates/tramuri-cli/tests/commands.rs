use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;
use tempfile::tempdir;
use tramuri_core::{DefineStepId, MissionPerspective, define_start, define_submit, mission_start};

fn tramuri() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tramuri"))
}

fn run_hook_check(platform: &str, event: &str, input: &Value) -> std::process::Output {
    let mut child = tramuri()
        .args(["hook-check", "--platform", platform, "--event", event])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("hook checker should launch");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(serde_json::to_string(input).unwrap().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
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
    assert!(stdout.contains("install"));
}

#[test]
fn native_hook_cli_blocks_then_unlocks_only_after_successful_start() {
    let project = tempdir().unwrap();
    let cwd = project.path().to_str().unwrap();
    let read = serde_json::json!({
        "cwd":cwd,
        "session_id":"hook-cli-test-session",
        "tool_use_id":"read-before-start",
        "tool_name":"Bash",
        "tool_input":{"command":"Get-ChildItem"}
    });
    let denied = run_hook_check("claude", "before-tool", &read);
    assert!(denied.status.success());
    let response: Value = serde_json::from_slice(&denied.stdout).unwrap();
    assert_eq!(response["hookSpecificOutput"]["permissionDecision"], "deny");

    let prompt = serde_json::json!({"cwd":cwd,"session_id":"hook-cli-test-session"});
    let reset = run_hook_check("claude", "prompt", &prompt);
    assert!(reset.status.success());
    assert!(reset.stdout.is_empty());

    let preflight = serde_json::json!({
        "cwd":cwd,
        "session_id":"hook-cli-test-session",
        "tool_use_id":"status-before-start",
        "tool_name":"Bash",
        "tool_input":{"command":"tramuri status"}
    });
    let allowed_preflight = run_hook_check("claude", "before-tool", &preflight);
    assert!(allowed_preflight.status.success());
    assert!(allowed_preflight.stdout.is_empty());

    let help = serde_json::json!({
        "cwd":cwd,
        "session_id":"hook-cli-test-session",
        "tool_use_id":"help-before-start",
        "tool_name":"Bash",
        "tool_input":{"command":"tramuri start --help"}
    });
    assert!(
        run_hook_check("claude", "before-tool", &help)
            .stdout
            .is_empty()
    );
    let mut help_completed = help.clone();
    help_completed["tool_response"] = serde_json::json!({"status":"completed"});
    assert!(
        run_hook_check("claude", "after-tool", &help_completed)
            .stdout
            .is_empty()
    );

    let still_locked = run_hook_check("claude", "before-tool", &read);
    let response: Value = serde_json::from_slice(&still_locked.stdout).unwrap();
    assert_eq!(response["hookSpecificOutput"]["permissionDecision"], "deny");

    let start = serde_json::json!({
        "cwd":cwd,
        "session_id":"hook-cli-test-session",
        "tool_use_id":"start-success",
        "tool_name":"Bash",
        "tool_input":{"command":"tramuri start --goal \"new product\" --perspective product_engineer"}
    });
    assert!(
        run_hook_check("claude", "before-tool", &start)
            .stdout
            .is_empty()
    );
    let mut completed = start.clone();
    completed["tool_response"] = serde_json::json!({"status":"completed"});
    assert!(
        run_hook_check("claude", "after-tool", &completed)
            .stdout
            .is_empty()
    );

    let mut read_after = read;
    read_after["tool_use_id"] = Value::String("read-after-start".into());
    let allowed = run_hook_check("claude", "before-tool", &read_after);
    assert!(allowed.status.success());
    assert!(allowed.stdout.is_empty());
}

#[test]
fn start_routes_exploratory_missions_and_keeps_readiness_unassessed() {
    let project = tempdir().unwrap();
    let path = project.path().to_str().unwrap();
    let output = tramuri()
        .args([
            "start",
            "--goal",
            "Criar um produto para pequenos negócios",
            "--perspective",
            "product_engineer",
            "--json",
            "--path",
            path,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["workflow"]["perspective"], "product_engineer");
    assert_eq!(result["workflow"]["next_activity"], "product_framing");
    assert_eq!(
        result["mission"]["success_contract"]["criteria"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let status = tramuri()
        .args(["status", "--json", "--path", path])
        .output()
        .unwrap();
    let status: Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(
        status["missions"][0]["workflow"]["next_activity"],
        "product_framing"
    );
    assert_eq!(status["missions"][0]["readiness"]["state"], "not_assessed");

    let resumed = tramuri()
        .args([
            "start",
            "--goal",
            "Criar um produto para pequenos negócios",
            "--json",
            "--path",
            path,
        ])
        .output()
        .unwrap();
    assert!(
        resumed.status.success(),
        "{}",
        String::from_utf8_lossy(&resumed.stderr)
    );
    let resumed: Value = serde_json::from_slice(&resumed.stdout).unwrap();
    assert_eq!(resumed["mission_reused"], true);
    assert_eq!(resumed["mission"]["id"], result["mission"]["id"]);

    let ambiguous = tempdir().unwrap();
    let ambiguous_path = ambiguous.path().to_str().unwrap();
    let output = tramuri()
        .args([
            "start",
            "--goal",
            "Criar uma solução",
            "--path",
            ambiguous_path,
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        !ambiguous
            .path()
            .join(".tramuri/database/tramuri.sqlite3")
            .exists()
    );

    let clear_but_unselected = tempdir().unwrap();
    let clear_path = clear_but_unselected.path().to_str().unwrap();
    let output = tramuri()
        .args([
            "start",
            "--goal",
            "Quero criar um SaaS para vender cursos online",
            "--path",
            clear_path,
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("choose who will lead this Mission"));
    assert!(
        !clear_but_unselected
            .path()
            .join(".tramuri/database/tramuri.sqlite3")
            .exists()
    );

    let unsupported = tramuri()
        .args([
            "start",
            "--goal",
            "Some objective",
            "--perspective",
            "other",
            "--path",
            clear_path,
        ])
        .output()
        .unwrap();
    assert!(!unsupported.status.success());
    assert!(
        String::from_utf8_lossy(&unsupported.stderr).contains("product_engineer, fde, or mixed")
    );
}

#[test]
fn status_refresh_updates_report_without_opening_browser() {
    let project = tempdir().unwrap();
    let path = project.path().to_str().unwrap();
    let started = tramuri()
        .args([
            "start",
            "--goal",
            "Create a course marketplace",
            "--perspective",
            "product_engineer",
            "--json",
            "--path",
            path,
        ])
        .output()
        .unwrap();
    assert!(started.status.success());
    let started: Value = serde_json::from_slice(&started.stdout).unwrap();
    let mission_id = started["mission"]["id"].as_str().unwrap();

    let refresh = tramuri()
        .args(["status", "--refresh", "--path", path])
        .output()
        .unwrap();
    assert!(
        refresh.status.success(),
        "{}",
        String::from_utf8_lossy(&refresh.stderr)
    );
    let stdout = String::from_utf8_lossy(&refresh.stdout);
    assert!(stdout.contains("Refreshed local Tramuri reports"));
    assert!(!stdout.contains("opened"));

    let report_dir = project.path().join(".tramuri/reports");
    for filename in ["tramauri.html", "tramauri.md", "tramauri.json"] {
        assert!(report_dir.join(filename).is_file(), "missing {filename}");
    }
    let json: Value = serde_json::from_slice(
        &fs::read(report_dir.join("tramauri.json")).expect("JSON report should be written"),
    )
    .unwrap();
    assert_eq!(json["missions"][0]["id"], mission_id);

    let changed = tramuri()
        .args([
            "mission",
            "workflow",
            "activity",
            "--mission",
            mission_id,
            "--activity",
            "define",
            "--path",
            path,
        ])
        .output()
        .unwrap();
    assert!(changed.status.success());

    let refresh = tramuri()
        .args(["status", "--refresh", "--path", path])
        .output()
        .unwrap();
    assert!(refresh.status.success());
    let json: Value = serde_json::from_slice(
        &fs::read(report_dir.join("tramauri.json")).expect("JSON report should be refreshed"),
    )
    .unwrap();
    assert_eq!(json["missions"][0]["workflow"]["next_activity"], "define");
    let markdown = fs::read_to_string(report_dir.join("tramauri.md")).unwrap();
    assert!(markdown.contains("Product definition"));
}

#[test]
fn status_refresh_rejects_other_output_modes() {
    let project = tempdir().unwrap();
    let path = project.path().to_str().unwrap();
    let init = tramuri().args(["init", path]).output().unwrap();
    assert!(init.status.success());

    for args in [
        vec!["status", "--refresh", "--json", "--path", path],
        vec!["status", "--refresh", "--open", "--path", path],
        vec!["status", "--refresh", "--hint", "--path", path],
        vec!["status", "--agent", "--refresh", "--path", path],
        vec![
            "status",
            "--refresh",
            "--hook-format",
            "claude",
            "--path",
            path,
        ],
    ] {
        let output = tramuri().args(args).output().unwrap();
        assert!(!output.status.success());
        assert!(
            !project
                .path()
                .join(".tramuri/reports/tramauri.json")
                .exists()
        );
    }
}

#[test]
fn workflow_activity_command_updates_product_phase_without_changing_perspective() {
    let project = tempdir().unwrap();
    let path = project.path().to_str().unwrap();
    let started = tramuri()
        .args([
            "start",
            "--goal",
            "Quero criar um SaaS de cursos online",
            "--perspective",
            "product_engineer",
            "--json",
            "--path",
            path,
        ])
        .output()
        .unwrap();
    assert!(started.status.success());
    let started: Value = serde_json::from_slice(&started.stdout).unwrap();
    let mission_id = started["mission"]["id"].as_str().unwrap();
    assert_eq!(started["workflow"]["perspective_source"], "explicit");
    assert_eq!(started["workflow"]["next_activity"], "product_framing");

    let updated = tramuri()
        .args([
            "mission",
            "workflow",
            "activity",
            "--mission",
            mission_id,
            "--activity",
            "discover",
            "--path",
            path,
            "--json",
        ])
        .output()
        .unwrap();
    assert!(updated.status.success());
    let updated: Value = serde_json::from_slice(&updated.stdout).unwrap();
    assert_eq!(updated["next_activity"], "discover");
    assert_eq!(updated["perspective"], "product_engineer");
    assert_eq!(updated["perspective_source"], "explicit");
    assert_eq!(updated["perspective_history"].as_array().unwrap().len(), 1);

    let resumed = tramuri()
        .args(["resume", "--mission", mission_id, "--path", path])
        .output()
        .unwrap();
    assert!(resumed.status.success());
    assert!(String::from_utf8_lossy(&resumed.stdout).contains("Current focus: Discovery"));
}

#[test]
fn legacy_mission_can_record_a_transparent_inferred_product_perspective() {
    let project = tempdir().unwrap();
    let path = project.path().to_str().unwrap();
    assert!(
        tramuri()
            .args(["init", path])
            .output()
            .unwrap()
            .status
            .success()
    );
    let created = tramuri()
        .args([
            "mission",
            "create",
            "--path",
            path,
            "--objective",
            "Criar um SaaS para vender cursos online",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(created.status.success());
    let mission: Value = serde_json::from_slice(&created.stdout).unwrap();
    let mission_id = mission["id"].as_str().unwrap();

    let updated = tramuri()
        .args([
            "mission",
            "workflow",
            "set",
            "--path",
            path,
            "--mission",
            mission_id,
            "--perspective",
            "product_engineer",
            "--source",
            "inferred",
            "--activity",
            "product_framing",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        updated.status.success(),
        "{}",
        String::from_utf8_lossy(&updated.stderr)
    );
    let workflow: Value = serde_json::from_slice(&updated.stdout).unwrap();
    assert_eq!(workflow["perspective"], "product_engineer");
    assert_eq!(workflow["perspective_source"], "inferred");
    assert_eq!(workflow["next_activity"], "product_framing");
}

#[test]
fn profile_and_execution_transition_commands_are_explicit() {
    let project = tempdir().unwrap();
    let path = project.path().to_str().unwrap();
    assert!(
        tramuri()
            .args(["init", path])
            .output()
            .unwrap()
            .status
            .success()
    );
    let profile = tramuri()
        .args(["profile", "set", "fde", "--path", path])
        .output()
        .unwrap();
    assert!(
        profile.status.success(),
        "{}",
        String::from_utf8_lossy(&profile.stderr)
    );
    let profile = tramuri()
        .args(["profile", "show", "--json", "--path", path])
        .output()
        .unwrap();
    let profile: Value = serde_json::from_slice(&profile.stdout).unwrap();
    assert_eq!(profile["default_work_perspective"], "fde");

    let start = tramuri()
        .args([
            "start",
            "--goal",
            "quero explorar uma necessidade",
            "--perspective",
            "fde",
            "--json",
            "--path",
            path,
        ])
        .output()
        .unwrap();
    assert!(start.status.success());
    let start: Value = serde_json::from_slice(&start.stdout).unwrap();
    let mission_id = start["mission"]["id"].as_str().unwrap();
    let execution = tramuri()
        .args([
            "execution",
            "record",
            "--mission",
            mission_id,
            "--contract-revision",
            "1",
            "--actor-kind",
            "human",
            "--status",
            "in_progress",
            "--json",
            "--path",
            path,
        ])
        .output()
        .unwrap();
    assert!(execution.status.success());
    let execution: Value = serde_json::from_slice(&execution.stdout).unwrap();
    assert_eq!(execution["work_perspective"], "fde");
    let execution_id = execution["id"].as_str().unwrap();
    let transition = tramuri()
        .args([
            "execution",
            "update",
            "--id",
            execution_id,
            "--status",
            "completed",
            "--json",
            "--path",
            path,
        ])
        .output()
        .unwrap();
    assert!(
        transition.status.success(),
        "{}",
        String::from_utf8_lossy(&transition.stderr)
    );
    let transition: Value = serde_json::from_slice(&transition.stdout).unwrap();
    assert_eq!(transition["state"], "completed");
    assert!(transition["ended_at_unix_ms"].as_i64().is_some());
}

#[test]
fn session_hook_hint_uses_each_hosts_native_output_contract_without_workspace_data() {
    for (platform, expected_key) in [
        ("claude", "hookSpecificOutput"),
        ("cursor", "additional_context"),
        ("gemini", "hookSpecificOutput"),
    ] {
        let output = tramuri()
            .args(["status", "--hint", "--hook-format", platform])
            .output()
            .expect("CLI should launch");
        assert!(output.status.success());
        let value: Value =
            serde_json::from_slice(&output.stdout).expect("hook output should be JSON");
        let context = if platform == "cursor" {
            value[expected_key]
                .as_str()
                .expect("Cursor context should be text")
        } else {
            assert_eq!(value[expected_key]["hookEventName"], "SessionStart");
            value[expected_key]["additionalContext"]
                .as_str()
                .expect("native context should be text")
        };
        assert!(context.contains("substantive new project"));
        assert!(context.contains("do not ask whether to use Tramuri"));
        assert!(!context.contains("Mission ID"));
    }

    let codex = tramuri()
        .args(["status", "--hint"])
        .output()
        .expect("CLI should launch");
    assert!(codex.status.success());
    assert!(String::from_utf8_lossy(&codex.stdout).contains("Tramuri integration is installed"));
}

#[test]
fn install_and_uninstall_cli_support_all_native_workflow_gate_platforms() {
    for platform in ["codex", "claude", "cursor", "gemini", "copilot"] {
        let project = tempdir().expect("project should be created");
        let path = project.path().to_str().expect("path should be UTF-8");
        let install = tramuri()
            .args([
                "install",
                "--project",
                "--platform",
                platform,
                "--path",
                path,
            ])
            .output()
            .expect("CLI should launch");
        assert!(
            install.status.success(),
            "{}",
            String::from_utf8_lossy(&install.stderr)
        );
        assert!(String::from_utf8_lossy(&install.stdout).contains("Native project hooks block"));

        let status = tramuri()
            .args(["integrations", "status", "--path", path])
            .output()
            .expect("CLI should launch");
        assert!(status.status.success());
        assert!(
            String::from_utf8_lossy(&status.stdout).contains(&format!("{platform}: installed"))
        );

        let uninstall = tramuri()
            .args([
                "uninstall",
                "--project",
                "--platform",
                platform,
                "--path",
                path,
            ])
            .output()
            .expect("CLI should launch");
        assert!(
            uninstall.status.success(),
            "{}",
            String::from_utf8_lossy(&uninstall.stderr)
        );
        assert!(!String::from_utf8_lossy(&uninstall.stdout).is_empty());
    }
}

#[test]
fn install_status_and_uninstall_manage_only_tramuri_content() {
    let project = tempdir().expect("project directory should be created");
    fs::write(
        project.path().join("AGENTS.md"),
        "# Existing instructions\nKeep this.\n",
    )
    .expect("user instructions should be written");
    let path = project.path().to_str().expect("path should be UTF-8");

    let install = tramuri()
        .args([
            "install",
            "--project",
            "--platform",
            "codex",
            "--path",
            path,
        ])
        .output()
        .expect("CLI should launch");
    assert!(
        install.status.success(),
        "{}",
        String::from_utf8_lossy(&install.stderr)
    );

    let instructions = fs::read_to_string(project.path().join("AGENTS.md")).unwrap();
    assert!(instructions.starts_with("# Existing instructions\nKeep this."));
    assert!(
        project
            .path()
            .join(".agents/skills/tramuri/SKILL.md")
            .is_file()
    );
    assert!(
        project
            .path()
            .join(".agents/skills/tramuri/references/project-discovery.md")
            .is_file()
    );
    for skill in [
        "tramuri-pe-objective-context",
        "tramuri-pe-business-discovery",
        "tramuri-pe-requirements",
        "tramuri-pe-evidence-hypotheses",
        "tramuri-pe-product-brief",
        "tramuri-pe-decision-support",
    ] {
        assert!(
            project
                .path()
                .join(format!(".agents/skills/{skill}/SKILL.md"))
                .is_file(),
            "managed integration should package {skill}"
        );
    }
    assert!(project.path().join(".codex/hooks.json").is_file());
    let install_output = String::from_utf8_lossy(&install.stdout);
    assert!(install_output.contains("TRAMURI"));
    assert!(install_output.contains("Native project hooks block"));
    assert!(install_output.contains("database-file presence"));
    assert!(install_output.contains("AGENTS.md"));
    assert!(install_output.contains("starts or resumes a Mission"));
    assert!(install_output.contains("initializes this workspace when needed"));
    assert!(!install_output.contains("run `tramuri init .`"));

    let hook: serde_json::Value =
        serde_json::from_slice(&fs::read(project.path().join(".codex/hooks.json")).unwrap())
            .unwrap();
    assert_eq!(
        hook["hooks"]["SessionStart"][0]["hooks"][0]["command"],
        "tramuri status --hint"
    );
    let normalized_instructions = instructions
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(normalized_instructions.contains("even when the user does not mention Tramuri"));
    assert!(!instructions.contains("may provide a compact"));

    let status = tramuri()
        .args(["integrations", "status", "--path", path])
        .output()
        .expect("CLI should launch");
    assert!(status.status.success());
    assert!(String::from_utf8_lossy(&status.stdout).contains("codex: installed"));

    let uninstall = tramuri()
        .args([
            "uninstall",
            "--project",
            "--platform",
            "codex",
            "--path",
            path,
        ])
        .output()
        .expect("CLI should launch");
    assert!(uninstall.status.success());
    let instructions = fs::read_to_string(project.path().join("AGENTS.md")).unwrap();
    assert_eq!(instructions, "# Existing instructions\nKeep this.\n");
    assert!(
        !project
            .path()
            .join(".agents/skills/tramuri/SKILL.md")
            .exists()
    );
    assert!(
        !project
            .path()
            .join(".agents/skills/tramuri/references/project-discovery.md")
            .exists()
    );
    let hooks: Value =
        serde_json::from_slice(&fs::read(project.path().join(".codex/hooks.json")).unwrap())
            .unwrap();
    assert!(
        hooks["hooks"]["SessionStart"]
            .as_array()
            .unwrap()
            .is_empty()
    );
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
fn mission_list_status_and_resume_share_safe_workspace_state() {
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
        r#"{"criteria":[{"description":"Review acceptance","required":true}]}"#,
    )
    .unwrap();
    let create = tramuri()
        .args([
            "mission",
            "create",
            "--path",
            path,
            "--objective",
            "Synthetic workflow",
            "--context",
            "private-context",
            "--criteria-file",
            criteria.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(create.status.success());
    let mission: Value = serde_json::from_slice(&create.stdout).unwrap();
    let mission_id = mission["id"].as_str().unwrap();

    let listed = tramuri()
        .args(["mission", "list", "--path", path, "--json"])
        .output()
        .unwrap();
    assert!(listed.status.success());
    let list: Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(list[0]["id"], mission_id);

    let status = tramuri()
        .args(["status", "--path", path, "--json"])
        .output()
        .unwrap();
    assert!(status.status.success());
    let status: Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status["schema"], "tramuri.status.v4");
    assert_eq!(status["missions"][0]["outcome"]["state"], "pending");
    assert!(status["missions"][0].get("context").is_none());
    assert!(!status.to_string().contains("private-context"));

    let resumed = tramuri().args(["resume", "--path", path]).output().unwrap();
    assert!(
        resumed.status.success(),
        "{}",
        String::from_utf8_lossy(&resumed.stderr)
    );
    assert!(String::from_utf8_lossy(&resumed.stdout).contains("Synthetic workflow"));

    let second = tramuri()
        .args([
            "mission",
            "create",
            "--path",
            path,
            "--objective",
            "Another workflow",
            "--criteria-file",
            criteria.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(second.status.success());
    let ambiguous = tramuri().args(["resume", "--path", path]).output().unwrap();
    assert!(!ambiguous.status.success());
    assert!(String::from_utf8_lossy(&ambiguous.stderr).contains("multiple open Missions"));
}

#[test]
fn product_engineering_start_initializes_define_and_cli_gates_progression() {
    let project = tempdir().unwrap();
    let path = project.path().to_str().unwrap();
    let started = tramuri()
        .args([
            "start",
            "--path",
            path,
            "--goal",
            "Quero criar um SaaS para vender cursos online",
            "--perspective",
            "product_engineer",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        started.status.success(),
        "{}",
        String::from_utf8_lossy(&started.stderr)
    );
    let start_progress = String::from_utf8_lossy(&started.stderr);
    assert!(start_progress.contains("Product Engineering · DEFINE"));
    assert!(start_progress.contains("▶ 1/5 · Compreender objetivo e contexto"));
    let started: Value = serde_json::from_slice(&started.stdout).unwrap();
    let id = started["mission"]["id"].as_str().unwrap();
    let report_dir = project.path().join(".tramuri/reports");
    assert_eq!(started["define"]["current_step"], "objective_context");

    let next = tramuri()
        .args([
            "mission",
            "define",
            "next",
            "--path",
            path,
            "--mission",
            id,
            "--json",
        ])
        .output()
        .unwrap();
    assert!(next.status.success());
    assert!(
        String::from_utf8_lossy(&next.stderr).contains("▶ 1/5 · Compreender objetivo e contexto")
    );
    let next: Value = serde_json::from_slice(&next.stdout).unwrap();
    assert_eq!(next["current_step"]["id"], "objective_context");
    assert_eq!(
        next["current_step"]["skill"],
        "tramuri-pe-objective-context"
    );

    let artifact = project.path().join("objective.json");
    fs::write(&artifact, r#"{"original_intent":"short","interpreted_objective":"","known_information":[],"hypotheses":[],"gaps":[]}"#).unwrap();
    let rejected = tramuri()
        .args([
            "mission",
            "define",
            "submit",
            "--path",
            path,
            "--mission",
            id,
            "--step",
            "objective_context",
            "--file",
            artifact.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(rejected.status.success());
    let rejected_progress = String::from_utf8_lossy(&rejected.stderr);
    let rejected: Value = serde_json::from_slice(&rejected.stdout).unwrap();
    assert_eq!(rejected["accepted"], false);
    assert_eq!(rejected["progress"]["completed"], 0);
    assert!(rejected_progress.contains("corrigir resultado"));
    fs::write(&artifact, r#"{"original_intent":"Quero criar um SaaS para vender cursos online","interpreted_objective":"Explorar uma plataforma de cursos","known_information":["SaaS","cursos online"],"hypotheses":["instrutores são compradores"],"gaps":["mercado inicial"]}"#).unwrap();
    let accepted = tramuri()
        .args([
            "mission",
            "define",
            "submit",
            "--path",
            path,
            "--mission",
            id,
            "--step",
            "objective_context",
            "--file",
            artifact.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(accepted.status.success());
    let accepted_progress = String::from_utf8_lossy(&accepted.stderr);
    let accepted: Value = serde_json::from_slice(&accepted.stdout).unwrap();
    assert_eq!(accepted["accepted"], true);
    assert_eq!(accepted["next_step"], "business_discovery");
    assert!(accepted_progress.contains("☑ 1/5 · Compreender objetivo e contexto"));
    assert!(accepted_progress.contains("▶ 2/5 · Identificar problema, público e negócio"));
    let report: Value = serde_json::from_slice(
        &fs::read(project.path().join(".tramuri/reports/tramauri.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(report["missions"][0]["define"]["completed"], 1);
    let html = fs::read_to_string(report_dir.join("tramauri.html")).unwrap();
    assert!(html.contains("defineTitle"));
    assert!(html.contains("DEFINE"));
    let markdown = fs::read_to_string(report_dir.join("tramauri.md")).unwrap();
    assert!(markdown.contains("Product Engineering DEFINE"));
    assert!(markdown.contains("1/5"));
}

#[test]
fn define_review_cli_preserves_previous_accepted_artifact() {
    let project = tempdir().unwrap();
    let path = project.path().to_str().unwrap();
    let mission_id = mission_start(
        project.path(),
        "Quero criar um SaaS para cursos",
        None,
        Some(MissionPerspective::ProductEngineer),
    )
    .unwrap()
    .mission
    .id;
    define_start(project.path(), &mission_id).unwrap();
    let entries = [
        (
            DefineStepId::ObjectiveContext,
            serde_json::json!({"original_intent":"Cursos","interpreted_objective":"Explorar plataforma","known_information":["SaaS"],"hypotheses":["instrutores compram"],"gaps":["mercado" ]}),
        ),
        (
            DefineStepId::BusinessDiscovery,
            serde_json::json!({"problem":"Venda fragmentada","target_audience":["instrutores"],"business_objectives":["validar demanda"],"value_proposition":"vender cursos","uncertainties":["mercado"],"references":[{"step":"objective_context","revision":1}]}),
        ),
        (
            DefineStepId::Requirements,
            serde_json::json!({"functional_requirements":["publicar curso"],"non_functional_requirements":["privacidade"],"constraints":["sem cobrança real"],"dependencies":[],"acceptance_criteria":["ver curso"],"success_criteria":["jornada funciona"],"gaps":[],"references":[{"step":"business_discovery","revision":1}]}),
        ),
        (
            DefineStepId::KnowledgeSynthesis,
            serde_json::json!({"classifications":[{"kind":"hypothesis","statement":"instrutores compram"}],"references":[{"step":"requirements","revision":1}],"open_questions":["mercado inicial"]}),
        ),
        (
            DefineStepId::ProductBrief,
            serde_json::json!({"objective":"definir plataforma","problem":"venda fragmentada","target_audience":["instrutores"],"value_proposition":"vender cursos","main_journey":["publicar","comprar","assistir"],"requirements":[{"step":"requirements","revision":1}],"mvp_scope":["catálogo"],"exclusions":["pagamentos reais"],"risks":["aquisição"],"assumptions":["web"],"approved_decisions":[],"candidate_success_criteria":["aluno assiste aula"],"next_action":"prototipar","references":[{"step":"knowledge_synthesis","revision":1}]}),
        ),
    ];
    for (step, payload) in &entries {
        assert!(
            define_submit(
                project.path(),
                &mission_id,
                *step,
                payload.clone(),
                None,
                None
            )
            .unwrap()
            .accepted
        );
    }
    let artifact_path = project.path().join("brief-review.json");
    let mut revised = entries[4].1.clone();
    revised["next_action"] = serde_json::json!("prototipar jornada do instrutor");
    fs::write(&artifact_path, serde_json::to_vec(&revised).unwrap()).unwrap();
    let output = tramuri()
        .args([
            "mission",
            "define",
            "review",
            "--path",
            path,
            "--mission",
            &mission_id,
            "--step",
            "product_brief",
            "--file",
            artifact_path.to_str().unwrap(),
            "--reason",
            "detalhar a jornada",
            "--author",
            "Product Engineer",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["accepted"], true);
    assert_eq!(result["workflow_state"], "completed");
    let history = tramuri()
        .args([
            "mission",
            "define",
            "next",
            "--path",
            path,
            "--mission",
            &mission_id,
            "--history",
            "--json",
        ])
        .output()
        .unwrap();
    let history: Value = serde_json::from_slice(&history.stdout).unwrap();
    let briefs: Vec<_> = history
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["step"] == "product_brief")
        .collect();
    assert_eq!(briefs.len(), 2);
    assert_eq!(briefs[1]["revision"], 2);
    assert_eq!(
        briefs[1]["payload"]["next_action"],
        "prototipar jornada do instrutor"
    );
    let brief_id = briefs[1]["id"].as_str().unwrap();
    let assessment_file = project.path().join("brief-assessment.json");
    let assessment = serde_json::json!({
        "schema":"tramuri.product-brief-assessment.v1",
        "brief_artifact_id":brief_id,
        "brief_revision":2,
        "rubric_version":"tramuri.product-brief-rubric.v1",
        "overall":"concerns",
        "criteria":[
            {"criterion":"goal_alignment","result":"pass","rationale":"synthetic review","references":[{"step":"product_brief","revision":2,"field":"objective"}]},
            {"criterion":"target_consistency","result":"concern","rationale":"synthetic audience mismatch","references":[{"step":"business_discovery","revision":1,"field":"target_audience"},{"step":"product_brief","revision":2,"field":"target_audience"}]},
            {"criterion":"evidence_assumption_separation","result":"pass","rationale":"synthetic review","references":[{"step":"product_brief","revision":2,"field":"assumptions"}]},
            {"criterion":"success_criteria_fit","result":"pass","rationale":"synthetic review","references":[{"step":"product_brief","revision":2,"field":"candidate_success_criteria"}]},
            {"criterion":"requirements_traceability","result":"pass","rationale":"synthetic review","references":[{"step":"product_brief","revision":2,"field":"requirements"}]},
            {"criterion":"decision_authorization","result":"pass","rationale":"synthetic review","references":[{"step":"product_brief","revision":2,"field":"approved_decisions"}]},
            {"criterion":"internal_consistency","result":"pass","rationale":"synthetic review","references":[{"step":"product_brief","revision":2,"field":"main_journey"}]}
        ],
        "limitations":["No real customer validation."],
        "evaluation_boundary":"host_semantic_assessment_not_core_verified"
    });
    fs::write(&assessment_file, serde_json::to_vec(&assessment).unwrap()).unwrap();
    let assessment_output = tramuri()
        .args([
            "mission",
            "define",
            "review",
            "--path",
            path,
            "--mission",
            &mission_id,
            "--assessment-file",
            assessment_file.to_str().unwrap(),
            "--evaluator",
            "synthetic Codex evaluator",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        assessment_output.status.success(),
        "{}",
        String::from_utf8_lossy(&assessment_output.stderr)
    );
    let assessment_result: Value = serde_json::from_slice(&assessment_output.stdout).unwrap();
    assert_eq!(assessment_result["overall"], "concerns");
    assert_eq!(
        assessment_result["evaluation_boundary"],
        "host_semantic_assessment_not_core_verified"
    );
    let assessment_report: Value = serde_json::from_slice(
        &fs::read(project.path().join(".tramuri/reports/tramauri.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        assessment_report["define_details"][0]["brief_assessment"]["state"],
        "current"
    );

    let mut business_revision = entries[1].1.clone();
    business_revision["target_audience"] = serde_json::json!(["instrutores independentes"]);
    fs::write(
        &artifact_path,
        serde_json::to_vec(&business_revision).unwrap(),
    )
    .unwrap();
    let upstream_review = tramuri()
        .args([
            "mission",
            "define",
            "review",
            "--path",
            path,
            "--mission",
            &mission_id,
            "--step",
            "business_discovery",
            "--file",
            artifact_path.to_str().unwrap(),
            "--reason",
            "corrigir público-alvo",
            "--author",
            "Product Engineer",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        upstream_review.status.success(),
        "{}",
        String::from_utf8_lossy(&upstream_review.stderr)
    );
    let current_report: Value = serde_json::from_slice(
        &fs::read(project.path().join(".tramuri/reports/tramauri.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(current_report["report_schema"], "tramuri.report.v5");
    let detail = &current_report["define_details"][0];
    assert_eq!(detail["workflow_state"], "stale");
    assert_eq!(detail["completed"], 2);
    assert_eq!(detail["steps"][2]["state"], "stale");
    assert_eq!(detail["steps"][4]["state"], "stale");
    assert_eq!(detail["brief"]["state"], "stale");
    assert_eq!(detail["brief"]["revision"], 2);
    assert_eq!(detail["brief_assessment"]["state"], "stale");
    let refreshed_md =
        fs::read_to_string(project.path().join(".tramuri/reports/tramauri.md")).unwrap();
    assert!(refreshed_md.contains("Product Brief · revision 2 · `stale`"));
    let refreshed_html =
        fs::read_to_string(project.path().join(".tramuri/reports/tramauri.html")).unwrap();
    assert!(refreshed_html.contains("Refresh report"));
    assert!(refreshed_html.contains("window.location.reload()"));
    assert!(refreshed_html.contains("Needs revalidation"));
}

#[test]
fn explicit_mission_relationships_appear_in_resume_without_leaking_to_workspace_status() {
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
        r#"{"criteria":[{"description":"Customer confirms the workflow","required":true}]}"#,
    )
    .unwrap();
    let created = tramuri()
        .args([
            "mission",
            "create",
            "--path",
            path,
            "--objective",
            "Synthetic customer workflow",
            "--criteria-file",
            criteria.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(created.status.success());
    let mission: Value = serde_json::from_slice(&created.stdout).unwrap();
    let mission_id = mission["id"].as_str().unwrap();

    let knowledge = tramuri()
        .args([
            "knowledge",
            "add",
            path,
            "--type",
            "decision",
            "--statement",
            "Use the synthetic customer sandbox",
            "--human-input",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(knowledge.status.success());
    let knowledge: Value = serde_json::from_slice(&knowledge.stdout).unwrap();
    let knowledge_id = knowledge["id"].as_str().unwrap();

    let skill = project.path().join(".agents/skills/sandbox/SKILL.md");
    fs::create_dir_all(skill.parent().unwrap()).unwrap();
    fs::write(
        &skill,
        "---\nname: sandbox\ndescription: Use a mock tenant\n---\n",
    )
    .unwrap();
    assert!(
        tramuri()
            .args(["scan", path])
            .output()
            .unwrap()
            .status
            .success()
    );
    let capabilities = tramuri()
        .args(["capabilities", "--path", path, "--type", "skill", "--json"])
        .output()
        .unwrap();
    assert!(capabilities.status.success());
    let capabilities: Value = serde_json::from_slice(&capabilities.stdout).unwrap();
    let capability_id = capabilities["capabilities"][0]["id"].as_str().unwrap();

    for args in [
        vec![
            "mission",
            "link-knowledge",
            "--path",
            path,
            "--mission",
            mission_id,
            "--knowledge",
            knowledge_id,
            "--actor-kind",
            "human",
            "--actor",
            "fde-one",
            "--json",
        ],
        vec![
            "mission",
            "link-capability",
            "--path",
            path,
            "--mission",
            mission_id,
            "--capability",
            capability_id,
            "--actor-kind",
            "agent",
            "--actor",
            "agent-session",
            "--json",
        ],
    ] {
        let linked = tramuri().args(args).output().unwrap();
        assert!(
            linked.status.success(),
            "{}",
            String::from_utf8_lossy(&linked.stderr)
        );
    }

    let relationships = tramuri()
        .args([
            "mission",
            "relationships",
            "--path",
            path,
            "--mission",
            mission_id,
            "--json",
        ])
        .output()
        .unwrap();
    assert!(relationships.status.success());
    let relationships: Value = serde_json::from_slice(&relationships.stdout).unwrap();
    assert_eq!(
        relationships["knowledge"][0]["knowledge"]["id"],
        knowledge_id
    );
    assert_eq!(
        relationships["knowledge"][0]["provenance"]["actor_reference"],
        "fde-one"
    );
    assert_eq!(
        relationships["capabilities"][0]["capability_id"],
        capability_id
    );
    assert_eq!(relationships["capabilities"][0]["freshness"], "current");

    let resume = tramuri()
        .args(["resume", "--path", path, "--mission", mission_id, "--json"])
        .output()
        .unwrap();
    assert!(resume.status.success());
    let resume: Value = serde_json::from_slice(&resume.stdout).unwrap();
    assert_eq!(resume["id"], mission_id);
    assert_eq!(
        resume["relationships"]["knowledge"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        resume["relationships"]["capabilities"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let status = tramuri()
        .args(["status", "--path", path, "--json"])
        .output()
        .unwrap();
    assert!(status.status.success());
    assert!(
        !String::from_utf8_lossy(&status.stdout).contains("Use the synthetic customer sandbox")
    );

    let unlinked = tramuri()
        .args([
            "mission",
            "unlink-knowledge",
            "--path",
            path,
            "--mission",
            mission_id,
            "--knowledge",
            knowledge_id,
            "--json",
        ])
        .output()
        .unwrap();
    assert!(unlinked.status.success());
    assert_eq!(String::from_utf8_lossy(&unlinked.stdout).trim(), "true");
}

#[test]
fn capabilities_cli_lists_filtered_declarations_after_scan_without_exposing_config_values() {
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
    let skill = project.path().join(".agents/skills/review/SKILL.md");
    fs::create_dir_all(skill.parent().unwrap()).unwrap();
    fs::write(
        &skill,
        "---\nname: review\ndescription: Review requirements\n---\nprivate skill body\n",
    )
    .unwrap();
    let mcp = project.path().join(".cursor/mcp.json");
    fs::create_dir_all(mcp.parent().unwrap()).unwrap();
    fs::write(
        &mcp,
        r#"{"mcpServers":{"internal":{"command":"secret-command","env":{"TOKEN":"secret-value"}}}}"#,
    )
    .unwrap();
    let scan = tramuri().args(["scan", path]).output().unwrap();
    assert!(scan.status.success());

    let all = tramuri()
        .args(["capabilities", "--path", path, "--json"])
        .output()
        .unwrap();
    assert!(
        all.status.success(),
        "{}",
        String::from_utf8_lossy(&all.stderr)
    );
    let all: Value = serde_json::from_slice(&all.stdout).unwrap();
    assert_eq!(all["schema"], "tramuri.capabilities.v1");
    assert_eq!(all["capabilities"].as_array().unwrap().len(), 2);
    assert!(!all.to_string().contains("secret-command"));
    assert!(!all.to_string().contains("secret-value"));
    assert!(!all.to_string().contains("private skill body"));

    let skills = tramuri()
        .args(["capabilities", "--path", path, "--type", "skill", "--json"])
        .output()
        .unwrap();
    assert!(skills.status.success());
    let skills: Value = serde_json::from_slice(&skills.stdout).unwrap();
    assert_eq!(skills["capabilities"].as_array().unwrap().len(), 1);
    assert_eq!(skills["capabilities"][0]["name"], "review");
}

#[test]
fn explicit_agent_snapshot_is_silent_for_uninitialized_project() {
    let project = tempdir().expect("project directory should be created");
    let path = project.path().to_str().expect("path should be UTF-8");

    let result = tramuri()
        .args(["status", "--path", path, "--agent"])
        .output()
        .unwrap();

    assert!(result.status.success());
    assert!(result.stdout.is_empty());
}

#[test]
fn status_hint_never_emits_workspace_records() {
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
        r#"{"criteria":[{"description":"private criterion","required":true}]}"#,
    )
    .unwrap();
    assert!(
        tramuri()
            .args([
                "mission",
                "create",
                "--path",
                path,
                "--objective",
                "private objective",
                "--criteria-file",
                criteria.to_str().unwrap()
            ])
            .output()
            .unwrap()
            .status
            .success()
    );

    let result = tramuri()
        .args(["status", "--path", path, "--hint"])
        .output()
        .unwrap();
    let output = String::from_utf8_lossy(&result.stdout);
    assert!(result.status.success());
    assert!(output.contains("A regular Tramuri database file was detected"));
    assert!(output.contains("substantive new project"));
    assert!(!output.contains("private objective"));
    assert!(!output.contains("private criterion"));
}

#[test]
fn uninitialized_session_hint_guides_start_without_initializing_or_reading_prompts() {
    let project = tempdir().expect("project directory should be created");
    let path = project.path().to_str().expect("path should be UTF-8");
    for format in [None, Some("claude"), Some("cursor"), Some("gemini")] {
        let mut command = tramuri();
        command.args(["status", "--path", path, "--hint"]);
        if let Some(format) = format {
            command.args(["--hook-format", format]);
        }
        let result = command.output().expect("session hint should run");
        assert!(result.status.success());
        let output = String::from_utf8_lossy(&result.stdout);
        assert!(output.contains("No Tramuri database was detected"));
        assert!(output.contains("invoke the Tramuri start workflow"));
        assert!(output.contains("even when the user does not name Tramuri"));
        assert!(output.contains("Do not create Missions for one-shot questions"));
        assert!(!project.path().join(".tramuri").exists());
        if format.is_some() {
            assert!(serde_json::from_slice::<serde_json::Value>(&result.stdout).is_ok());
        }
    }
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
fn execution_and_evidence_retrieval_lists_and_shows_json_records() {
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
        r#"{"criteria":[{"description":"Customer confirms"}]}"#,
    )
    .unwrap();
    let created = tramuri()
        .args([
            "mission",
            "create",
            "--path",
            path,
            "--objective",
            "FDE handoff",
            "--criteria-file",
            criteria.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(created.status.success());
    let mission: Value = serde_json::from_slice(&created.stdout).unwrap();
    let mission_id = mission["id"].as_str().unwrap();

    let mut executions = Vec::new();
    let mut evidences = Vec::new();
    for (actor, status, locator) in [
        ("human", "blocked", "https://example.invalid/blocked"),
        ("tool", "completed", "https://example.invalid/completed"),
    ] {
        let output = tramuri()
            .args([
                "execution",
                "record",
                "--path",
                path,
                "--mission",
                mission_id,
                "--contract-revision",
                "1",
                "--actor-kind",
                actor,
                "--status",
                status,
                "--actor",
                "fde-1",
                "--work-perspective",
                "fde",
                "--notes",
                "handoff notes",
                "--json",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let execution: Value = serde_json::from_slice(&output.stdout).unwrap();
        let evidence = tramuri()
            .args([
                "evidence",
                "add",
                "--path",
                path,
                "--mission",
                mission_id,
                "--execution",
                execution["id"].as_str().unwrap(),
                "--kind",
                "external_reference",
                "--locator",
                locator,
                "--json",
            ])
            .output()
            .unwrap();
        assert!(
            evidence.status.success(),
            "{}",
            String::from_utf8_lossy(&evidence.stderr)
        );
        executions.push(execution);
        evidences.push(serde_json::from_slice::<Value>(&evidence.stdout).unwrap());
    }

    let list = tramuri()
        .args([
            "execution",
            "list",
            "--path",
            path,
            "--mission",
            mission_id,
            "--json",
        ])
        .output()
        .unwrap();
    assert!(list.status.success());
    let listed_executions: Vec<Value> = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(listed_executions.len(), 2);
    assert!(
        listed_executions
            .iter()
            .all(|item| item["work_perspective"] == "fde")
    );
    let mut expected_executions = executions.clone();
    expected_executions.sort_by(|left, right| {
        right["started_at_unix_ms"]
            .as_i64()
            .cmp(&left["started_at_unix_ms"].as_i64())
            .then_with(|| right["id"].as_str().cmp(&left["id"].as_str()))
    });
    assert_eq!(listed_executions, expected_executions);
    for execution in &executions {
        let shown = tramuri()
            .args([
                "execution",
                "show",
                "--path",
                path,
                "--id",
                execution["id"].as_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap();
        assert!(shown.status.success());
        assert_eq!(
            serde_json::from_slice::<Value>(&shown.stdout).unwrap(),
            *execution
        );
    }

    let list = tramuri()
        .args([
            "evidence",
            "list",
            "--path",
            path,
            "--mission",
            mission_id,
            "--json",
        ])
        .output()
        .unwrap();
    assert!(list.status.success());
    let listed_evidences: Vec<Value> = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(listed_evidences.len(), 2);
    let mut expected_evidences = evidences.clone();
    expected_evidences.sort_by(|left, right| {
        right["captured_at_unix_ms"]
            .as_i64()
            .cmp(&left["captured_at_unix_ms"].as_i64())
            .then_with(|| right["id"].as_str().cmp(&left["id"].as_str()))
    });
    assert_eq!(listed_evidences, expected_evidences);
    for evidence in &evidences {
        let shown = tramuri()
            .args([
                "evidence",
                "show",
                "--path",
                path,
                "--id",
                evidence["id"].as_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap();
        assert!(shown.status.success());
        assert_eq!(
            serde_json::from_slice::<Value>(&shown.stdout).unwrap(),
            *evidence
        );
    }

    let empty = tramuri()
        .args([
            "execution",
            "list",
            "--path",
            path,
            "--mission",
            "missing-mission",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(!empty.status.success());
    assert!(String::from_utf8_lossy(&empty.stderr).contains("mission was not found"));
}

#[test]
fn outcome_commands_record_assessments_and_report_verified_state() {
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
        r#"{"criteria":[{"description":"Customer accepts","required":true}]}"#,
    )
    .unwrap();
    let created = tramuri()
        .args([
            "mission",
            "create",
            "--path",
            path,
            "--objective",
            "Deliver integration",
            "--criteria-file",
            criteria.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(created.status.success());
    let mission: Value = serde_json::from_slice(&created.stdout).unwrap();
    let execution = tramuri()
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
    assert!(execution.status.success());
    let execution: Value = serde_json::from_slice(&execution.stdout).unwrap();
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
            "reviewer-confirmed",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(evidence.status.success());
    let evidence: Value = serde_json::from_slice(&evidence.stdout).unwrap();
    let assessed = tramuri()
        .args([
            "outcome",
            "assess",
            "--path",
            path,
            "--mission",
            mission["id"].as_str().unwrap(),
            "--contract-revision",
            "1",
            "--criterion",
            mission["success_contract"]["criteria"][0]["id"]
                .as_str()
                .unwrap(),
            "--result",
            "met",
            "--evidence",
            evidence["id"].as_str().unwrap(),
            "--verifier-kind",
            "human",
            "--verifier",
            "product-owner",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        assessed.status.success(),
        "{}",
        String::from_utf8_lossy(&assessed.stderr)
    );
    let assessment: Value = serde_json::from_slice(&assessed.stdout).unwrap();
    assert_eq!(assessment["result"], "met");
    let outcome = tramuri()
        .args([
            "outcome",
            "show",
            "--path",
            path,
            "--mission",
            mission["id"].as_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(outcome.status.success());
    let outcome: Value = serde_json::from_slice(&outcome.stdout).unwrap();
    assert_eq!(outcome["state"], "verified");
}

#[test]
fn exploratory_mission_and_readiness_are_available_through_cli() {
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

    let created = tramuri()
        .args([
            "mission",
            "create",
            "--path",
            path,
            "--objective",
            "Investigate a customer workflow",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        created.status.success(),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    let mission: Value = serde_json::from_slice(&created.stdout).unwrap();
    assert!(
        mission["success_contract"]["criteria"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let initial_status = tramuri()
        .args(["status", "--path", path, "--json"])
        .output()
        .unwrap();
    assert!(initial_status.status.success());
    let initial_status: Value = serde_json::from_slice(&initial_status.stdout).unwrap();
    assert_eq!(
        initial_status["missions"][0]["readiness"]["state"],
        "not_assessed"
    );
    assert_eq!(initial_status["missions"][0]["outcome"]["state"], "pending");

    let mission_id = mission["id"].as_str().unwrap();
    let assessed = tramuri()
        .args([
            "readiness",
            "assess",
            "--path",
            path,
            "--mission",
            mission_id,
            "--activity",
            "discover",
            "--state",
            "blocked",
            "--blocker",
            "Waiting for a customer interview",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        assessed.status.success(),
        "{}",
        String::from_utf8_lossy(&assessed.stderr)
    );
    let assessment: Value = serde_json::from_slice(&assessed.stdout).unwrap();
    assert_eq!(assessment["decision"], "blocked");

    let shown = tramuri()
        .args([
            "readiness",
            "show",
            "--path",
            path,
            "--mission",
            mission_id,
            "--json",
        ])
        .output()
        .unwrap();
    assert!(shown.status.success());
    let shown: Value = serde_json::from_slice(&shown.stdout).unwrap();
    assert_eq!(shown["current"]["state"], "blocked");
    assert_eq!(shown["history"].as_array().unwrap().len(), 1);
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
    assert_eq!(init_json["schema_version"], 10);
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
