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

#[derive(serde::Serialize)]
struct ReportDocument<'a> {
    #[serde(flatten)]
    status: &'a StatusSnapshot,
    capability_schema: &'a str,
    capability_scan_complete: bool,
    capabilities: Vec<ReportCapability<'a>>,
    capability_warnings: &'a [String],
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
            "### {} — {}\n\n- State: `{}`\n- Success Contract: revision {}\n- Outcome: `{}`\n- Executions: {}\n- Evidence references: {}\n\n",
            mission.id,
            md_escape(&mission.objective),
            state_name(mission.state),
            mission.contract.revision,
            outcome_name(mission.outcome.state),
            mission.executions.len(),
            mission.evidence.len(),
        ));
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
        "\nKnowledge is not shown under Missions because the current data model does not associate Knowledge records with a Mission. Free-form context, execution notes, evidence locators, verifier references, and assessment explanations are excluded from this report.\n",
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
    Ok(format!(
        r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="referrer" content="no-referrer"><title>Tramuri workspace</title>
<style>
:root{{--bg:#111318;--panel:#191c23;--line:#2b303a;--text:#eceef2;--muted:#a0a7b4;--accent:#8da8ff;--green:#86d6aa;--amber:#f2c879;--red:#f08b8b;font:15px/1.55 system-ui,-apple-system,"Segoe UI",sans-serif;color-scheme:dark}}
*{{box-sizing:border-box}}body{{margin:0;background:var(--bg);color:var(--text)}}header{{position:sticky;top:0;background:#111318ed;backdrop-filter:blur(12px);border-bottom:1px solid var(--line);padding:18px max(22px,calc((100vw - 1160px)/2));display:flex;gap:24px;align-items:center;justify-content:space-between;z-index:2}}h1{{font-size:20px;margin:0}}.muted{{color:var(--muted);font-size:13px}}main{{max-width:1160px;margin:28px auto;padding:0 22px}}.stats{{display:grid;grid-template-columns:repeat(5,1fr);gap:12px;margin-bottom:22px}}.stat,.card{{background:var(--panel);border:1px solid var(--line);border-radius:12px;padding:17px}}.stat strong{{display:block;font-size:25px}}.toolbar{{display:flex;gap:10px;margin:18px 0}}input,select{{background:var(--panel);border:1px solid var(--line);border-radius:8px;padding:10px;color:var(--text)}}input{{flex:1}}.mission{{margin:12px 0}}.mission summary{{cursor:pointer;list-style:none;display:flex;justify-content:space-between;gap:14px;align-items:start}}.mission summary::-webkit-details-marker{{display:none}}.title{{font-size:17px;font-weight:650}}.badge{{border:1px solid var(--line);border-radius:99px;padding:2px 9px;font-size:12px;white-space:nowrap}}.verified{{color:var(--green)}}.pending{{color:var(--amber)}}.not_achieved{{color:var(--red)}}.grid{{display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-top:18px}}h2{{font-size:16px;margin:22px 0 10px}}h3{{font-size:14px;margin:10px 0}}ul{{padding-left:20px}}li{{margin:5px 0}}.event{{border-left:2px solid var(--line);padding:4px 0 6px 12px}}.empty{{color:var(--muted)}}@media(max-width:760px){{.stats{{grid-template-columns:repeat(2,1fr)}}.grid{{grid-template-columns:1fr}}header{{position:static;align-items:flex-start;flex-direction:column}}}}
</style></head><body>
<script type="application/json" id="report-data">{json}</script>
<header><div><h1>Tramuri <span class="muted">Workspace status</span></h1><div class="muted" id="generated"></div></div><span class="muted">Local report · no external services</span></header>
<main><section class="stats" id="stats"></section><div class="toolbar"><input id="search" type="search" placeholder="Search Missions and capabilities"><select id="state"><option value="all">All states</option><option value="open">Open</option><option value="closed">Closed</option></select></div><section id="missions"></section><h2>Recent activity</h2><section id="activity" class="card"></section><h2>Declared capabilities</h2><section id="capabilities" class="card"></section><p class="muted">Capability records describe declarations only; Tramuri does not execute hooks, connect to MCP servers, or verify operational availability. Knowledge is not attached to Missions without an explicit relationship.</p></main>
<script>
const report=JSON.parse(document.getElementById('report-data').textContent);const d=report;const el=(tag,text,cls)=>{{const n=document.createElement(tag);if(text!==undefined)n.textContent=text;if(cls)n.className=cls;return n}};
document.getElementById('generated').textContent='Generated '+new Date(d.generated_at_unix_ms).toLocaleString()+' · Contract '+d.schema;
const stats=document.getElementById('stats');[['Missions',d.workspace.mission_count],['Open',d.workspace.open_mission_count],['Closed',d.workspace.closed_mission_count],['Verified outcomes',d.missions.filter(m=>m.outcome.state==='verified').length],['Capabilities',report.capabilities.length]].forEach(([k,v])=>{{const c=el('div',undefined,'stat');c.append(el('span',k,'muted'),el('strong',String(v)));stats.append(c)}});
function render(){{const q=document.getElementById('search').value.toLowerCase();const state=document.getElementById('state').value;const host=document.getElementById('missions');host.replaceChildren();const list=d.missions.filter(m=>(state==='all'||m.state===state)&&(m.objective.toLowerCase().includes(q)||m.id.toLowerCase().includes(q)));if(!list.length)host.append(el('p','No matching Missions.','empty'));for(const m of list){{const details=el('details',undefined,'card mission');const summary=el('summary');const left=el('div');left.append(el('div',m.objective,'title'),el('div','Mission '+m.id+' · contract revision '+m.contract.revision,'muted'));summary.append(left,el('span',m.outcome.state.replace('_',' '),'badge '+m.outcome.state));details.append(summary);const grid=el('div',undefined,'grid');const c=el('section');c.append(el('h3','Success criteria'));for(const r of m.outcome.criteria){{const a=r.assessments[r.assessments.length-1];const status=a?a.result:'unverified';const li=el('li',r.criterion.description+' — '+status+(r.criterion.required?' · required':' · optional'));c.append(li)}}const w=el('section');w.append(el('h3','Work history'));for(const e of m.executions)w.append(el('div','Execution '+e.id+' · '+e.state+' · '+e.actor_kind,'event'));for(const e of m.evidence)w.append(el('div','Evidence '+e.id+' · '+e.kind+' · '+e.availability,'event'));if(!m.executions.length&&!m.evidence.length)w.append(el('p','No executions or evidence recorded.','empty'));grid.append(c,w);details.append(grid);const footer=el('div',m.outcome.state==='verified'?'Outcome verified by recorded criterion assessments.':m.outcome.state==='not_achieved'?'Outcome not achieved according to current required criteria.':'Outcome pending review.','muted');details.append(footer);host.append(details)}}}}
document.getElementById('search').addEventListener('input',render);document.getElementById('state').addEventListener('change',render);render();
const history=el('section');history.append(el('h2','Success Contract history'));let hasHistory=false;for(const m of d.missions){{if(m.contract_history.length<2)continue;hasHistory=true;const group=el('details','', 'card');group.append(el('summary',m.objective+' · '+m.id));for(const rev of [...m.contract_history].reverse().slice(1)){{const section=el('section');section.append(el('h3','Revision '+rev.contract_revision+' · outcome '+rev.state));for(const c of rev.criteria){{const latest=c.assessments[c.assessments.length-1];section.append(el('div',c.criterion.description+' — '+(latest?latest.result:'unverified')+(c.criterion.required?' · required':' · optional'),'event'));for(const a of c.assessments)section.append(el('div','Review '+a.id+' · '+a.result+' · '+a.verifier_kind+' · '+a.assessed_at_unix_ms+' · evidence '+(a.evidence_ids.join(', ')||'none'),'muted'))}}group.append(section)}}}}if(hasHistory)document.querySelector('main').insertBefore(history,document.getElementById('activity').parentElement.previousElementSibling);
const reviews=el('section');reviews.append(el('h2','Criterion review history'));let hasReviews=false;for(const m of d.missions){{for(const rev of m.contract_history){{for(const c of rev.criteria){{if(!c.assessments.length)continue;hasReviews=true;const block=el('details','', 'card');block.append(el('summary',m.objective+' · revision '+rev.contract_revision+' · '+c.criterion.description));for(const a of c.assessments)block.append(el('div','Review '+a.id+' · '+a.result+' · '+a.verifier_kind+' · '+new Date(a.assessed_at_unix_ms).toLocaleString()+' · evidence '+(a.evidence_ids.join(', ')||'none'),'event'));reviews.append(block)}}}}}}if(hasReviews)document.querySelector('main').insertBefore(reviews,document.getElementById('activity').parentElement.previousElementSibling);
const activity=document.getElementById('activity');if(!d.recent_activity.length)activity.append(el('p','No recent activity recorded.','empty'));for(const a of d.recent_activity)activity.append(el('div',a.kind+' · Mission '+a.mission_id+' · '+a.record_id+' · '+new Date(a.occurred_at_unix_ms).toLocaleString(),'event'));
const caps=document.getElementById('capabilities');if(!report.capabilities.length)caps.append(el('p','No declarative capabilities were recognized. Run `tramuri scan` to refresh discovery.','empty'));for(const c of report.capabilities){{const row=el('div',undefined,'event');row.append(el('strong',c.name+' · '+c.category.replace('_',' ')),el('div',c.ecosystem+' · '+c.state.replaceAll('_',' ')+' · '+c.path,'muted'));if(c.limitation)row.append(el('div',c.limitation,'muted'));caps.append(row)}}if(report.capability_warnings.length)for(const warning of report.capability_warnings)caps.append(el('div','Warning: '+warning,'muted'));
</script></body></html>"#
    ))
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
        CriterionInput, MissionInput, capability_snapshot, init, mission_create, scan,
        status_snapshot,
    };

    #[test]
    fn report_writes_required_filenames_and_escapes_script_payloads() {
        let project = tempdir().unwrap();
        init(project.path()).unwrap();
        mission_create(
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
        let html = render_html(&document).unwrap();
        assert!(!html.contains("<script>alert('unsafe')</script>"));
        assert!(html.contains("\\u003cscript\\u003e"));
        assert!(!html.contains("private-context"));
        let markdown = render_markdown(&snapshot, &capabilities);
        assert!(markdown.contains("&lt;script&gt;"));

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
        assert_eq!(json["schema"], "tramuri.status.v1");
        assert_eq!(json["capabilities"][0]["name"], "test");
        assert_eq!(json["capability_summary"]["total"], 1);
        assert_eq!(json["workspace"]["mission_count"], 1);
    }
}
