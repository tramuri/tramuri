use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use tramuri_core::{
    CapabilityKind, CapabilitySnapshot, CapabilityState, CriterionAssessmentResult, MissionState,
    OutcomeState, StatusSnapshot, capability_snapshot,
};

const JSON_FILE: &str = "tramauri.json";
const MARKDOWN_FILE: &str = "tramauri.md";
const HTML_FILE: &str = "tramauri.html";
const REPORT_HTML: &str = include_str!("../assets/tramauri-report.html");

#[derive(serde::Serialize)]
struct ReportDocument<'a> {
    #[serde(flatten)]
    status: &'a StatusSnapshot,
    report_schema: &'static str,
    capability_schema: &'a str,
    capability_scan_complete: bool,
    capabilities: Vec<ReportCapability<'a>>,
    capability_warnings: &'a [String],
    execution_actors: Vec<ReportExecutionActor<'a>>,
}

#[derive(serde::Serialize)]
struct ReportExecutionActor<'a> {
    mission_id: &'a str,
    execution_id: &'a str,
    actor_reference: &'a str,
}

#[derive(serde::Serialize)]
struct ReportCapability<'a> {
    id: &'a str,
    name: &'a str,
    category: CapabilityKind,
    ecosystem: &'a str,
    path: &'a str,
    state: CapabilityState,
    availability: &'a str,
    limitation: Option<&'a str>,
}

pub fn generate_and_open(
    root: &Path,
    snapshot: &StatusSnapshot,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let html_path = write_reports(root, snapshot)?;
    open_local_file(&html_path)?;
    Ok(html_path)
}

pub fn write_reports(
    root: &Path,
    snapshot: &StatusSnapshot,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = fs::canonicalize(root)?;
    let state_dir = root.join(".tramuri");
    ensure_directory(&state_dir)?;
    let reports_dir = state_dir.join("reports");
    ensure_directory(&reports_dir)?;

    let capabilities = capability_snapshot(&root)?;
    let stable_snapshot =
        preserve_generation_time_if_unchanged(&reports_dir, snapshot, &capabilities);
    let document = report_document(&stable_snapshot, &capabilities);
    let json = serde_json::to_string_pretty(&document)?;
    let markdown = render_markdown(&stable_snapshot, &capabilities);
    let html = render_html(&document)?;
    write_report(&reports_dir.join(JSON_FILE), json.as_bytes())?;
    write_report(&reports_dir.join(MARKDOWN_FILE), markdown.as_bytes())?;
    let html_path = reports_dir.join(HTML_FILE);
    write_report(&html_path, html.as_bytes())?;
    Ok(html_path)
}

fn ensure_directory(path: &Path) -> std::io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "refusing to write through a symlink or non-directory: {}",
                    path.display()
                ),
            ))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => fs::create_dir(path),
        Err(error) => Err(error),
    }
}

fn report_document<'a>(
    snapshot: &'a StatusSnapshot,
    capabilities: &'a CapabilitySnapshot,
) -> ReportDocument<'a> {
    ReportDocument {
        status: snapshot,
        report_schema: "tramuri.report.v2",
        capability_schema: &capabilities.schema,
        capability_scan_complete: capabilities.scan_complete,
        capabilities: capabilities
            .capabilities
            .iter()
            .map(|item| ReportCapability {
                id: &item.id,
                name: &item.name,
                category: item.category,
                ecosystem: &item.ecosystem,
                path: &item.path,
                state: item.state,
                availability: &item.availability,
                limitation: item.limitation.as_deref(),
            })
            .collect(),
        capability_warnings: &capabilities.warnings,
        execution_actors: snapshot
            .missions
            .iter()
            .flat_map(|mission| {
                mission.executions.iter().filter_map(move |execution| {
                    execution.actor_reference.as_deref().map(|actor_reference| {
                        ReportExecutionActor {
                            mission_id: &mission.id,
                            execution_id: &execution.id,
                            actor_reference,
                        }
                    })
                })
            })
            .collect(),
    }
}

fn write_report(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(path)
        && (metadata.file_type().is_symlink() || !metadata.is_file())
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing to replace a symlink or non-file: {}",
                path.display()
            ),
        ));
    }
    if fs::read(path).is_ok_and(|existing| existing == contents) {
        return Ok(());
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| std::io::Error::other("system clock is before the Unix epoch"))?
        .as_nanos();
    let temporary = path.with_file_name(format!(".tramuri-{}-{stamp}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(contents)?;
    file.sync_all()?;
    drop(file);
    if cfg!(windows) && path.exists() {
        fs::remove_file(path)?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

fn preserve_generation_time_if_unchanged(
    reports_dir: &Path,
    snapshot: &StatusSnapshot,
    capabilities: &CapabilitySnapshot,
) -> StatusSnapshot {
    let path = reports_dir.join(JSON_FILE);
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return snapshot.clone();
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return snapshot.clone();
    }
    let Ok(previous_bytes) = fs::read(&path) else {
        return snapshot.clone();
    };
    let Ok(mut previous) = serde_json::from_slice::<serde_json::Value>(&previous_bytes) else {
        return snapshot.clone();
    };
    let Some(previous_generated) = previous
        .get("generated_at_unix_ms")
        .and_then(serde_json::Value::as_i64)
    else {
        return snapshot.clone();
    };
    let mut current = match serde_json::to_value(report_document(snapshot, capabilities)) {
        Ok(current) => current,
        Err(_) => return snapshot.clone(),
    };
    previous
        .as_object_mut()
        .map(|object| object.remove("generated_at_unix_ms"));
    current
        .as_object_mut()
        .map(|object| object.remove("generated_at_unix_ms"));
    if previous == current {
        let mut stable = snapshot.clone();
        stable.generated_at_unix_ms = previous_generated;
        stable
    } else {
        snapshot.clone()
    }
}

fn open_local_file(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("explorer.exe");
        command.arg(path);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(path);
        command
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };
    command.spawn().map(|_| ())
}

pub fn render_markdown(snapshot: &StatusSnapshot, capabilities: &CapabilitySnapshot) -> String {
    let mut output = format!(
        "# Tramuri workspace status\n\nSchema: `{}`  \nGenerated at (Unix ms): `{}`  \nMissions: {} ({} open, {} closed)\n\n",
        snapshot.schema,
        snapshot.generated_at_unix_ms,
        snapshot.workspace.mission_count,
        snapshot.workspace.open_mission_count,
        snapshot.workspace.closed_mission_count,
    );
    output.push_str("## Missions\n\n");
    if snapshot.missions.is_empty() {
        output.push_str("No Missions are recorded.\n");
    }
    for mission in &snapshot.missions {
        output.push_str(&format!(
            "### {} — {}\n\n- State: `{}`\n- Current activity: `{}`\n- Context readiness: `{}`\n- Success Contract: revision {}\n- Outcome: `{}`\n- Executions: {}\n- Evidence references: {}\n\n",
            mission.id,
            md_escape(&mission.objective),
            state_name(mission.state),
            mission.readiness.activity.as_deref().unwrap_or("not selected"),
            readiness_name(mission.readiness.state),
            mission.contract.revision,
            outcome_name(mission.outcome.state),
            mission.executions.len(),
            mission.evidence.len(),
        ));
        if let Some(readiness) = &mission.readiness.assessment {
            for item in &readiness.criteria_applied {
                output.push_str(&format!("- Readiness check applied: {}\n", md_escape(item)));
            }
            for (label, items) in [
                ("Gap", &readiness.gaps),
                ("Blocker", &readiness.blockers),
                ("Accepted uncertainty", &readiness.accepted_uncertainties),
            ] {
                for item in items {
                    output.push_str(&format!("- {label}: {}\n", md_escape(item)));
                }
            }
            output.push_str(&format!(
                "- Readiness policy `{}` · context references: {} · reviewed at `{}` by declared actor `{}`\n",
                md_escape(&readiness.policy_version),
                readiness.context_references.len(),
                readiness.assessed_at_unix_ms,
                md_escape(
                    readiness
                        .actor_reference
                        .as_deref()
                        .unwrap_or("not recorded")
                ),
            ));
        }
        output.push_str("Criteria:\n\n");
        for review in &mission.outcome.criteria {
            let latest = review
                .assessments
                .last()
                .map(|a| assessment_name(a.result))
                .unwrap_or("unverified");
            output.push_str(&format!(
                "- [{}] {} — `{}`\n",
                if review.criterion.required {
                    "required"
                } else {
                    "optional"
                },
                md_escape(&review.criterion.description),
                latest,
            ));
            for assessment in &review.assessments {
                output.push_str(&format!(
                    "  - Review `{}`: `{}` by `{}` at `{}`; evidence IDs: {}\n",
                    assessment.id,
                    assessment_name(assessment.result),
                    format!("{:?}", assessment.verifier_kind).to_lowercase(),
                    assessment.assessed_at_unix_ms,
                    if assessment.evidence_ids.is_empty() {
                        "none".to_owned()
                    } else {
                        assessment.evidence_ids.join(", ")
                    },
                ));
            }
        }
        output.push('\n');
        if mission.contract_history.len() > 1 {
            output.push_str("Success Contract history:\n\n");
            for revision in mission.contract_history.iter().rev().skip(1) {
                output.push_str(&format!(
                    "- Revision {} — outcome `{}`\n",
                    revision.contract_revision,
                    outcome_name(revision.state)
                ));
                for review in &revision.criteria {
                    let result = review
                        .assessments
                        .last()
                        .map(|assessment| assessment_name(assessment.result))
                        .unwrap_or("unverified");
                    output.push_str(&format!(
                        "  - {} — `{}`\n",
                        md_escape(&review.criterion.description),
                        result
                    ));
                }
            }
            output.push('\n');
        }
        output.push_str("Executions:\n\n");
        if mission.executions.is_empty() {
            output.push_str("- None recorded.\n");
        }
        for execution in &mission.executions {
            output.push_str(&format!(
                "- `{}` — `{:?}` by `{:?}`, contract revision {}, started `{}`\n",
                execution.id,
                execution.state,
                execution.actor_kind,
                execution.contract_revision,
                execution.started_at_unix_ms
            ));
        }
        output.push_str("\nEvidence references:\n\n");
        if mission.evidence.is_empty() {
            output.push_str("- None recorded.\n");
        }
        for evidence in &mission.evidence {
            output.push_str(&format!(
                "- `{}` — `{:?}`, availability `{}`, captured `{}`, execution `{}`\n",
                evidence.id,
                evidence.kind,
                evidence.availability,
                evidence.captured_at_unix_ms,
                evidence.execution_id
            ));
        }
        output.push('\n');
    }
    output.push_str("## Recent activity\n\n");
    if snapshot.recent_activity.is_empty() {
        output.push_str("No execution, evidence, or assessment activity is recorded.\n");
    } else {
        for activity in &snapshot.recent_activity {
            output.push_str(&format!(
                "- `{}` — `{}` for Mission `{}` at `{}`\n",
                activity.kind,
                activity.record_id,
                activity.mission_id,
                activity.occurred_at_unix_ms
            ));
        }
    }
    output.push_str("\n## Declarative capabilities\n\n");
    output.push_str(&format!(
        "Registry: `{}` · scan complete: `{}` · recognized records: {}\n\n",
        capabilities.schema,
        capabilities.scan_complete,
        capabilities.capabilities.len(),
    ));
    if capabilities.capabilities.is_empty() {
        output.push_str("No declarative capabilities were recognized.\n");
    }
    for capability in &capabilities.capabilities {
        output.push_str(&format!(
            "- `{}` `{}` · `{}` · `{}` · source `{}` · availability `{}`{}\n",
            capability.category.as_str(),
            md_escape(&capability.name),
            md_escape(&capability.ecosystem),
            format!("{:?}", capability.state).to_lowercase(),
            md_escape(&capability.path),
            md_escape(&capability.availability),
            capability
                .limitation
                .as_deref()
                .map(|value| format!(" · note: {}", md_escape(value)))
                .unwrap_or_default(),
        ));
    }
    for warning in &capabilities.warnings {
        output.push_str(&format!("- Warning: {}\n", md_escape(warning)));
    }
    output.push_str(
        "\nKnowledge appears under a Mission only when explicitly linked. Free-form context, execution notes, evidence locators, verifier references, and outcome review explanations are excluded from this report. Readiness gaps and blockers are included because they were explicitly recorded for that assessment.\n",
    );
    output
}

fn render_html(document: &ReportDocument<'_>) -> Result<String, serde_json::Error> {
    let mut json = serde_json::to_string(document)?;
    json = json
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029");
    Ok(REPORT_HTML.replace("__TRAMURI_REPORT_DATA__", &json))
}

fn state_name(value: MissionState) -> &'static str {
    match value {
        MissionState::Open => "open",
        MissionState::Closed => "closed",
    }
}
fn outcome_name(value: OutcomeState) -> &'static str {
    match value {
        OutcomeState::Pending => "pending",
        OutcomeState::NotAchieved => "not_achieved",
        OutcomeState::Verified => "verified",
    }
}

fn readiness_name(value: tramuri_core::ReadinessState) -> &'static str {
    match value {
        tramuri_core::ReadinessState::NotAssessed => "not_assessed",
        tramuri_core::ReadinessState::Preparing => "preparing",
        tramuri_core::ReadinessState::NeedsReview => "needs_review",
        tramuri_core::ReadinessState::Ready => "ready",
        tramuri_core::ReadinessState::Stale => "stale",
        tramuri_core::ReadinessState::Blocked => "blocked",
    }
}
fn assessment_name(value: CriterionAssessmentResult) -> &'static str {
    match value {
        CriterionAssessmentResult::Met => "met",
        CriterionAssessmentResult::NotMet => "not_met",
        CriterionAssessmentResult::Unverified => "unverified",
    }
}
fn md_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "\\|")
        .replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tramuri_core::{
        ActorKind, CriterionInput, ExecutionInput, ExecutionState, MissionInput,
        ReadinessAssessmentInput, ReadinessDecision, WorkPerspective, capability_snapshot,
        execution_record, init, mission_create, readiness_assess, scan, status_snapshot,
    };

    #[test]
    fn report_writes_required_filenames_and_escapes_script_payloads() {
        let project = tempdir().unwrap();
        init(project.path()).unwrap();
        let mission = mission_create(
            project.path(),
            MissionInput {
                objective: "<script>alert('unsafe')</script> & rollout".to_owned(),
                context: Some("private-context".to_owned()),
                criteria: vec![CriterionInput {
                    description: "Check <img src=x onerror=alert(1)>".to_owned(),
                    required: true,
                    evidence_expectation: None,
                }],
            },
        )
        .unwrap();
        execution_record(
            project.path(),
            ExecutionInput {
                mission_id: mission.id.clone(),
                contract_revision: 1,
                actor_kind: ActorKind::Agent,
                actor_reference: Some("Codex (synthetic)".to_owned()),
                work_perspective: Some(WorkPerspective::ProductEngineer),
                state: ExecutionState::Completed,
                notes: None,
            },
        )
        .unwrap();
        readiness_assess(
            project.path(),
            ReadinessAssessmentInput {
                mission_id: mission.id.clone(),
                activity: "prototype".to_owned(),
                decision: ReadinessDecision::Blocked,
                criteria_applied: vec!["Check the privacy boundary".to_owned()],
                gaps: vec!["Review stakeholder input".to_owned()],
                blockers: vec!["<script>do not execute</script>".to_owned()],
                accepted_uncertainties: Vec::new(),
                actor_reference: Some("synthetic-reviewer".to_owned()),
            },
        )
        .unwrap();
        let skill = project.path().join(".agents/skills/test/SKILL.md");
        fs::create_dir_all(skill.parent().unwrap()).unwrap();
        fs::write(
            &skill,
            "---\nname: test\ndescription: \"<script>unsafe</script>\"\n---\n",
        )
        .unwrap();
        scan(project.path()).unwrap();
        let snapshot = status_snapshot(project.path()).unwrap();
        let capabilities = capability_snapshot(project.path()).unwrap();
        assert_eq!(capabilities.capabilities.len(), 1);
        let document = report_document(&snapshot, &capabilities);
        let report_json = serde_json::to_value(&document).unwrap();
        assert_eq!(report_json["missions"][0]["readiness"]["state"], "blocked");
        assert_eq!(
            report_json["missions"][0]["readiness"]["assessment"]["blockers"][0],
            "<script>do not execute</script>"
        );
        let html = render_html(&document).unwrap();
        assert!(!html.contains("<script>alert('unsafe')</script>"));
        assert!(html.contains("\\u003cscript\\u003e"));
        assert!(!html.contains("private-context"));
        assert!(!html.contains("<script>do not execute</script>"));
        assert!(html.contains("readinessBlocked"));
        assert!(html.contains("nextActivity"));
        assert!(html.contains("Codex (synthetic)"));
        assert!(html.contains("Product Engineer"));
        assert!(html.contains("PT-BR"));
        assert!(html.contains("Your work, in context."));
        assert!(html.contains("Seu trabalho, com contexto."));
        let markdown = render_markdown(&snapshot, &capabilities);
        assert!(markdown.contains("&lt;script&gt;"));
        assert!(markdown.contains("Blocker: &lt;script&gt;do not execute&lt;/script&gt;"));

        let html_path = write_reports(project.path(), &snapshot).unwrap();
        let report_dir = project.path().join(".tramuri/reports");
        assert_eq!(html_path.file_name().unwrap(), HTML_FILE);
        assert!(report_dir.join(JSON_FILE).is_file());
        assert!(report_dir.join(MARKDOWN_FILE).is_file());
        assert!(report_dir.join(HTML_FILE).is_file());
        assert_eq!(fs::read_dir(report_dir).unwrap().count(), 3);
        let json: serde_json::Value = serde_json::from_slice(
            &fs::read(project.path().join(".tramuri/reports").join(JSON_FILE)).unwrap(),
        )
        .unwrap();
        assert_eq!(json["schema"], "tramuri.status.v2");
        assert_eq!(json["capabilities"][0]["name"], "test");
        assert_eq!(json["capability_summary"]["total"], 1);
        assert_eq!(json["workspace"]["mission_count"], 1);
    }
}
