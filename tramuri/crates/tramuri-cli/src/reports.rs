use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use tramuri_core::{
    CapabilityKind, CapabilitySnapshot, CapabilityState, CriterionAssessmentResult, MissionState,
    OutcomeState, StatusSnapshot, capability_snapshot, define_artifact_history,
    define_brief_assessment_get,
};

const JSON_FILE: &str = "tramauri.json";
const MARKDOWN_FILE: &str = "tramauri.md";
const HTML_FILE: &str = "tramauri.html";
const REPORT_HTML: &str = include_str!("../assets/tramauri-report.html");
const REPORT_LOGO_PNG: &[u8] = include_bytes!("../assets/tramuri-logo-mark.png");

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
    define_details: &'a [ReportDefineSummary],
}

#[derive(serde::Serialize)]
struct ReportDefineSummary {
    mission_id: String,
    workflow_state: String,
    current_step: Option<String>,
    completed: u8,
    total: u8,
    clarification_count: u32,
    steps: Vec<ReportDefineStep>,
    hypotheses: Vec<String>,
    pending_decisions: Vec<String>,
    brief: Option<ReportBriefSummary>,
    brief_assessment: Option<ReportBriefAssessment>,
}

#[derive(serde::Serialize)]
struct ReportBriefAssessment {
    state: String,
    evaluator: String,
    rubric_version: String,
    overall: String,
    assessed_at_unix_ms: i64,
    limitation_count: usize,
    criteria: Vec<ReportBriefAssessmentCriterion>,
}

#[derive(serde::Serialize)]
struct ReportBriefAssessmentCriterion {
    criterion: String,
    result: String,
}

#[derive(serde::Serialize)]
struct ReportDefineStep {
    id: String,
    state: String,
    artifact_revision: Option<u32>,
}

#[derive(serde::Serialize)]
struct ReportBriefSummary {
    state: String,
    revision: u32,
    objective: Option<String>,
    problem: Option<String>,
    target_audience: Vec<String>,
    value_proposition: Option<String>,
    mvp_scope: Vec<String>,
    constraints: Vec<String>,
    risks: Vec<String>,
    next_action: Option<String>,
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
    let define_details = build_define_summaries(&root, snapshot)?;
    let stable_snapshot = preserve_generation_time_if_unchanged(
        &reports_dir,
        snapshot,
        &capabilities,
        &define_details,
    );
    let document = report_document(&stable_snapshot, &capabilities, &define_details);
    let json = serde_json::to_string_pretty(&document)?;
    let markdown = render_markdown_with_define(&stable_snapshot, &capabilities, &define_details);
    let html = render_html(&document)?;
    write_report(&reports_dir.join(JSON_FILE), json.as_bytes())?;
    write_report(&reports_dir.join(MARKDOWN_FILE), markdown.as_bytes())?;
    let html_path = reports_dir.join(HTML_FILE);
    write_report(&html_path, html.as_bytes())?;
    Ok(html_path)
}

fn build_define_summaries(
    root: &Path,
    snapshot: &StatusSnapshot,
) -> Result<Vec<ReportDefineSummary>, Box<dyn std::error::Error>> {
    let mut summaries = Vec::new();
    for mission in &snapshot.missions {
        let Some(define) = &mission.define else {
            continue;
        };
        let history = define_artifact_history(root, &mission.id, None)?;
        let artifact_for = |step: tramuri_core::DefineStepId| {
            define
                .steps
                .iter()
                .find(|item| item.id == step)
                .and_then(|item| item.artifact_id.as_deref())
                .and_then(|id| history.iter().find(|artifact| artifact.id == id))
        };
        let steps = define
            .steps
            .iter()
            .map(|step| ReportDefineStep {
                id: step.id.as_str().to_owned(),
                state: format!("{:?}", step.state).to_lowercase(),
                artifact_revision: artifact_for(step.id).map(|artifact| artifact.revision),
            })
            .collect();
        let synthesis = artifact_for(tramuri_core::DefineStepId::KnowledgeSynthesis);
        let hypotheses = if define.steps.iter().any(|step| {
            step.id == tramuri_core::DefineStepId::KnowledgeSynthesis
                && step.state == tramuri_core::DefineStepState::Completed
        }) {
            synthesis
                .and_then(|artifact| artifact.payload.get("classifications"))
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter(|item| {
                    item.get("kind").and_then(serde_json::Value::as_str) == Some("hypothesis")
                })
                .filter_map(|item| item.get("statement").and_then(serde_json::Value::as_str))
                .filter_map(safe_report_text)
                .take(5)
                .collect()
        } else {
            Vec::new()
        };
        let mut pending_decisions: Vec<String> = tramuri_core::define_next(root, &mission.id)?
            .questions
            .into_iter()
            .filter(|question| question.state != "answered")
            .filter_map(|question| safe_report_text(&question.rationale))
            .take(5)
            .collect();
        if pending_decisions.is_empty()
            && define.steps.iter().any(|step| {
                step.id == tramuri_core::DefineStepId::KnowledgeSynthesis
                    && step.state == tramuri_core::DefineStepState::Completed
            })
        {
            pending_decisions = synthesis
                .and_then(|artifact| artifact.payload.get("open_questions"))
                .map(safe_string_values)
                .unwrap_or_default();
        }

        let brief_step = define
            .steps
            .iter()
            .find(|step| step.id == tramuri_core::DefineStepId::ProductBrief);
        let brief_artifact = artifact_for(tramuri_core::DefineStepId::ProductBrief);
        let requirements = artifact_for(tramuri_core::DefineStepId::Requirements);
        let brief = brief_step.zip(brief_artifact).map(|(step, artifact)| {
            let get_text = |key: &str| {
                artifact
                    .payload
                    .get(key)
                    .and_then(serde_json::Value::as_str)
                    .and_then(safe_report_text)
            };
            let constraints = artifact
                .payload
                .get("constraints")
                .map(safe_string_values)
                .filter(|items| !items.is_empty())
                .or_else(|| {
                    requirements
                        .and_then(|item| item.payload.get("constraints"))
                        .map(safe_string_values)
                })
                .unwrap_or_default();
            ReportBriefSummary {
                state: format!("{:?}", step.state).to_lowercase(),
                revision: artifact.revision,
                objective: get_text("objective"),
                problem: get_text("problem"),
                target_audience: artifact
                    .payload
                    .get("target_audience")
                    .map(safe_string_values)
                    .unwrap_or_default(),
                value_proposition: get_text("value_proposition"),
                mvp_scope: artifact
                    .payload
                    .get("mvp_scope")
                    .map(safe_string_values)
                    .unwrap_or_default(),
                constraints,
                risks: artifact
                    .payload
                    .get("risks")
                    .map(safe_string_values)
                    .unwrap_or_default(),
                next_action: (step.state == tramuri_core::DefineStepState::Completed)
                    .then(|| get_text("next_action"))
                    .flatten(),
            }
        });
        let brief_assessment = define_brief_assessment_get(root, &mission.id)?.map(|assessment| {
            let state = if brief_step.is_some_and(|step| {
                step.state == tramuri_core::DefineStepState::Completed
                    && brief_artifact.is_some_and(|artifact| {
                        artifact.id == assessment.brief_artifact_id
                            && artifact.revision == assessment.brief_revision
                    })
            }) {
                "current"
            } else {
                "stale"
            };
            ReportBriefAssessment {
                state: state.to_owned(),
                evaluator: safe_report_text(&assessment.evaluator)
                    .unwrap_or_else(|| "redacted".to_owned()),
                rubric_version: assessment.rubric_version,
                overall: assessment.overall,
                assessed_at_unix_ms: assessment.occurred_at_unix_ms,
                limitation_count: assessment.limitations.len(),
                criteria: assessment
                    .criteria
                    .into_iter()
                    .map(|item| ReportBriefAssessmentCriterion {
                        criterion: item.criterion,
                        result: item.result,
                    })
                    .collect(),
            }
        });
        summaries.push(ReportDefineSummary {
            mission_id: mission.id.clone(),
            workflow_state: format!("{:?}", define.state).to_lowercase(),
            current_step: define.current_step.map(|step| step.as_str().to_owned()),
            completed: define.completed,
            total: define.total,
            clarification_count: define.clarification_count,
            steps,
            hypotheses,
            pending_decisions,
            brief,
            brief_assessment,
        });
    }
    Ok(summaries)
}

fn safe_string_values(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .filter_map(safe_report_text)
        .take(5)
        .collect()
}

fn safe_report_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if [
        "password",
        "passwd",
        "secret",
        "api_key",
        "api-key",
        "token=",
        "bearer ",
        "credential",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return Some("[redacted: sensitive-looking content]".to_owned());
    }
    Some(trimmed.chars().take(240).collect())
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
    define_details: &'a [ReportDefineSummary],
) -> ReportDocument<'a> {
    ReportDocument {
        status: snapshot,
        report_schema: "tramuri.report.v5",
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
        define_details,
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
    define_details: &[ReportDefineSummary],
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
    let mut current =
        match serde_json::to_value(report_document(snapshot, capabilities, define_details)) {
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

#[cfg(test)]
fn render_markdown(snapshot: &StatusSnapshot, capabilities: &CapabilitySnapshot) -> String {
    render_markdown_with_define(snapshot, capabilities, &[])
}

fn render_markdown_with_define(
    snapshot: &StatusSnapshot,
    capabilities: &CapabilitySnapshot,
    define_details: &[ReportDefineSummary],
) -> String {
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
            "### {} — {}\n\n- State: `{}`\n- Work perspective: `{}`\n- Current focus / next activity: `{}`\n- Context readiness: `{}`\n- Success Contract: revision {}\n- Outcome: `{}`\n- Executions: {} ({} in progress)\n- Evidence references: {}\n\n",
            mission.id,
            md_escape(&mission.objective),
            state_name(mission.state),
            mission.workflow.as_ref().and_then(|workflow| workflow.perspective).map(|p| p.as_str()).unwrap_or("not recorded"),
            md_escape(mission.workflow.as_ref().and_then(|workflow| workflow.next_activity.as_deref()).or(mission.readiness.activity.as_deref()).map(activity_name).unwrap_or("not selected")),
            readiness_name(mission.readiness.state),
            mission.contract.revision,
            outcome_name(mission.outcome.state),
            mission.executions.len(),
            mission.executions.iter().filter(|execution| execution.state == tramuri_core::ExecutionState::InProgress).count(),
            mission.evidence.len(),
        ));
        if let Some(define) = &mission.define {
            output.push_str(&format!(
                "#### Product Engineering DEFINE — {:?} ({}/{})\n\n",
                define.state, define.completed, define.total
            ));
            for step in &define.steps {
                output.push_str(&format!(
                    "- [{}] `{}` — {:?}\n",
                    if step.state == tramuri_core::DefineStepState::Completed {
                        "x"
                    } else {
                        " "
                    },
                    step.id.as_str(),
                    step.state
                ));
            }
            output.push_str(&format!(
                "- Clarifications: {}/{}\n\n",
                define.clarification_count, define.clarification_maximum
            ));
            output.push_str("Workflow events (newest first):\n\n");
            for event in define.events.iter().rev().take(20) {
                output.push_str(&format!(
                    "- `{}` · `{}` · `{}`\n",
                    event.occurred_at_unix_ms,
                    event
                        .step
                        .map(tramuri_core::DefineStepId::as_str)
                        .unwrap_or("workflow"),
                    md_escape(&event.event_type)
                ));
            }
            output.push('\n');
        }
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
    if !define_details.is_empty() {
        output.push_str("## Product Engineering DEFINE details\n\n");
        for detail in define_details {
            output.push_str(&format!(
                "### Mission `{}` · `{}` · {}/{} validated\n\n",
                detail.mission_id, detail.workflow_state, detail.completed, detail.total
            ));
            if let Some(step) = &detail.current_step {
                output.push_str(&format!("- Current step: `{}`\n", md_escape(step)));
            }
            output.push_str(&format!(
                "- Clarifications used: {}\n",
                detail.clarification_count
            ));
            for step in &detail.steps {
                output.push_str(&format!(
                    "- `{}`: `{}`{}\n",
                    md_escape(&step.id),
                    step.state,
                    step.artifact_revision
                        .map(|revision| format!(" · artifact revision {revision}"))
                        .unwrap_or_default()
                ));
            }
            for hypothesis in &detail.hypotheses {
                output.push_str(&format!("- Hypothesis: {}\n", md_escape(hypothesis)));
            }
            for question in &detail.pending_decisions {
                output.push_str(&format!(
                    "- Pending decision / question: {}\n",
                    md_escape(question)
                ));
            }
            if let Some(brief) = &detail.brief {
                output.push_str(&format!(
                    "\n#### Product Brief · revision {} · `{}`\n\n",
                    brief.revision, brief.state
                ));
                for (label, value) in [
                    ("Objective", brief.objective.as_ref()),
                    ("Problem", brief.problem.as_ref()),
                    ("Value proposition", brief.value_proposition.as_ref()),
                ] {
                    if let Some(value) = value {
                        output.push_str(&format!("- {label}: {}\n", md_escape(value)));
                    }
                }
                for (label, values) in [
                    ("Target audience", &brief.target_audience),
                    ("MVP", &brief.mvp_scope),
                    ("Constraints", &brief.constraints),
                    ("Risks", &brief.risks),
                ] {
                    if !values.is_empty() {
                        output.push_str(&format!(
                            "- {label}: {}\n",
                            values
                                .iter()
                                .map(|value| md_escape(value))
                                .collect::<Vec<_>>()
                                .join("; ")
                        ));
                    }
                }
                if let Some(action) = &brief.next_action {
                    output.push_str(&format!(
                        "- Suggested action when the Brief was created (may be outdated): {}\n",
                        md_escape(action)
                    ));
                }
            }
            if let Some(assessment) = &detail.brief_assessment {
                output.push_str(&format!(
                    "\n#### Semantic assessment · `{}` · `{}`\n\n- Evaluator (declared): `{}`\n- Rubric: `{}`\n- Limitation notes: {}\n- Core verified semantic truth: no\n",
                    assessment.state,
                    assessment.overall,
                    md_escape(&assessment.evaluator),
                    md_escape(&assessment.rubric_version),
                    assessment.limitation_count,
                ));
                for criterion in &assessment.criteria {
                    output.push_str(&format!(
                        "- `{}`: `{}`\n",
                        md_escape(&criterion.criterion),
                        md_escape(&criterion.result)
                    ));
                }
            }
            output.push('\n');
        }
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

fn activity_name(value: &str) -> &str {
    match value {
        "product_framing" => "Product framing",
        "discover" => "Discovery",
        "define" => "Product definition",
        "scope" => "MVP scope",
        "prototype" => "Prototype",
        "implement" => "Implementation",
        "validate" | "verify" => "Validation",
        "deploy" => "Deployment",
        "launch" => "Launch",
        "learn" => "Learning",
        custom => custom,
    }
}

fn render_html(document: &ReportDocument<'_>) -> Result<String, serde_json::Error> {
    let mut json = serde_json::to_string(document)?;
    json = json
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029");
    let logo = format!("data:image/png;base64,{}", encode_base64(REPORT_LOGO_PNG));
    let html = REPORT_HTML.replace("__TRAMURI_LOGO_DATA_URI__", &logo);
    Ok(html.replace("__TRAMURI_REPORT_DATA__", &json))
}

fn encode_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or_default();
        let third = chunk.get(2).copied().unwrap_or_default();
        encoded.push(ALPHABET[(first >> 2) as usize] as char);
        encoded.push(ALPHABET[(((first & 0b11) << 4) | (second >> 4)) as usize] as char);
        encoded.push(if chunk.len() > 1 {
            ALPHABET[(((second & 0b1111) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            ALPHABET[(third & 0b111111) as usize] as char
        } else {
            '='
        });
    }
    encoded
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
        ActorKind, CriterionInput, ExecutionInput, ExecutionState, IntentRouteSource, MissionInput,
        MissionPerspective, ReadinessAssessmentInput, ReadinessDecision, WorkPerspective,
        capability_snapshot, execution_record, init, mission_create, mission_workflow_set,
        readiness_assess, scan, status_snapshot,
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
        mission_workflow_set(
            project.path(),
            &mission.id,
            MissionPerspective::ProductEngineer,
            IntentRouteSource::Explicit,
            Some("prototype"),
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
        let document = report_document(&snapshot, &capabilities, &[]);
        let report_json = serde_json::to_value(&document).unwrap();
        assert_eq!(report_json["missions"][0]["readiness"]["state"], "blocked");
        assert_eq!(
            report_json["missions"][0]["workflow"]["perspective"],
            "product_engineer"
        );
        assert_eq!(
            report_json["missions"][0]["workflow"]["next_activity"],
            "prototype"
        );
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
        assert!(html.contains("Current focus / next activity"));
        assert!(html.contains("PT-BR"));
        assert!(html.contains("See where the work stands. Continue from here."));
        assert!(html.contains("Veja onde o trabalho está. Continue daqui."));
        assert!(html.contains("Mission Control"));
        assert!(html.contains(&format!(
            "data:image/png;base64,{}",
            encode_base64(REPORT_LOGO_PNG)
        )));
        assert!(!html.contains("__TRAMURI_LOGO_DATA_URI__"));
        assert!(html.contains("id=\"mission-focus\""));
        assert!(html.contains("focusSelectionHelpAria"));
        assert!(html.contains("Go to Product Brief"));
        assert!(html.contains("Ir para o Product Brief"));
        assert!(html.contains("These shortcuts go to sections in this report"));
        assert!(html.contains("Estes atalhos levam a seções deste relatório"));
        assert!(html.contains("focusActivityMayNeedReview"));
        assert!(html.contains("focusBriefRecommendationCaveat"));
        let planned_activity_branch = REPORT_HTML.find("} else if (plannedActivity) {").unwrap();
        let brief_recommendation_branch = REPORT_HTML
            .find("} else if (brief?.next_action && brief.state === \"completed\") {")
            .unwrap();
        assert!(planned_activity_branch < brief_recommendation_branch);
        let mut marker_snapshot = snapshot.clone();
        marker_snapshot.missions[0].objective = "__TRAMURI_LOGO_DATA_URI__".to_owned();
        let marker_html =
            render_html(&report_document(&marker_snapshot, &capabilities, &[])).unwrap();
        assert!(marker_html.contains("\"objective\":\"__TRAMURI_LOGO_DATA_URI__\""));
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
        assert_eq!(json["schema"], "tramuri.status.v4");
        assert_eq!(json["report_schema"], "tramuri.report.v5");
        assert_eq!(json["capabilities"][0]["name"], "test");
        assert_eq!(json["capability_summary"]["total"], 1);
        assert_eq!(json["workspace"]["mission_count"], 1);
    }

    #[test]
    fn base64_encoder_matches_rfc_4648_examples() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"M"), "TQ==");
        assert_eq!(encode_base64(b"Ma"), "TWE=");
        assert_eq!(encode_base64(b"Man"), "TWFu");
    }

    #[test]
    fn report_template_is_standalone_responsive_and_accessible() {
        assert!(REPORT_HTML.contains("<main class=\"content\" id=\"main-content\">"));
        assert!(REPORT_HTML.contains("data-i18n-aria"));
        assert!(REPORT_HTML.contains("@media (max-width: 760px)"));
        assert!(REPORT_HTML.contains("@media (prefers-reduced-motion: reduce)"));
        assert!(REPORT_HTML.contains(":focus-visible"));
        assert!(REPORT_HTML.contains("#00b3a6"));
        assert!(REPORT_HTML.contains("#0b2530"));
        assert!(REPORT_HTML.contains("#006d6a"));
        assert!(REPORT_HTML.contains("#ffa229"));
        assert!(!REPORT_HTML.contains("<script src="));
        assert!(!REPORT_HTML.contains("<link rel="));
        assert!(REPORT_HTML.contains("@media print"));
    }

    #[test]
    fn mission_renderer_uses_shared_define_step_labels() {
        let shared_labels = REPORT_HTML.find("const defineStepNames = {").unwrap();
        let focus_renderer = REPORT_HTML.find("function renderMissionFocus()").unwrap();
        let missions_renderer = REPORT_HTML
            .find("function renderMissions(query, state)")
            .unwrap();

        assert!(shared_labels < focus_renderer);
        assert!(focus_renderer < missions_renderer);
        assert!(
            REPORT_HTML[focus_renderer..missions_renderer]
                .contains("defineStepNames[define.current_step]")
        );
        assert!(REPORT_HTML[missions_renderer..].contains("defineStepNames[step.id]"));
        assert!(REPORT_HTML[missions_renderer..].contains("defineStepNames[event.step]"));
    }
}
