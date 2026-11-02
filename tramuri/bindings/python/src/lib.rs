use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use serde::Deserialize;
use tramuri_core::{
    ActorKind, AskWorkPerspective, CriterionAssessmentInput, CriterionAssessmentResult,
    CriterionInput, DefineStepId, EvidenceInput, EvidenceKind, ExecutionInput, ExecutionState,
    KnowledgeFilter, KnowledgeInput, KnowledgeType, MissionInput, MissionPerspective,
    ReadinessAssessmentInput, ReadinessDecision, TramuriError as CoreError, VerifierKind,
    WorkPerspective, WorkProfile, WorkProfilePreference, capability_list, define_artifact_history,
    define_brief_assess, define_brief_assessment_get, define_next, define_question_answer,
    define_question_request, define_review, define_start, define_status, define_submit,
    evidence_add, evidence_get, evidence_list, execution_get, execution_list, execution_record,
    execution_transition, init, knowledge_add, knowledge_list, knowledge_withdraw,
    mission_capability_link, mission_capability_unlink, mission_close, mission_contract_revise,
    mission_create, mission_get, mission_knowledge_link, mission_knowledge_unlink, mission_list,
    mission_relationships, mission_resume, mission_start, mission_workflow_activity_set,
    mission_workflow_get, mission_workflow_set, outcome_assess, outcome_get, profile_get,
    profile_set, readiness_assess, readiness_get, readiness_history, scan, status_snapshot,
};

create_exception!(_native, TramuriError, PyException);

#[pyfunction]
fn _cli_main(args: Vec<String>) -> i32 {
    tramuri_cli::run_from(args)
}

fn map_core_error(error: CoreError) -> PyErr {
    TramuriError::new_err(error.to_string())
}

fn map_serialization_error(error: serde_json::Error) -> PyErr {
    TramuriError::new_err(format!("failed to serialize Tramuri result: {error}"))
}

#[pyfunction]
fn _init_json(path: &str) -> PyResult<String> {
    let report = init(path).map_err(map_core_error)?;
    serde_json::to_string(&report).map_err(map_serialization_error)
}

#[pyfunction]
fn _scan_json(path: &str) -> PyResult<String> {
    let report = scan(path).map_err(map_core_error)?;
    serde_json::to_string(&report).map_err(map_serialization_error)
}

#[pyfunction]
fn _capability_snapshot_json(path: &str, kind: Option<&str>) -> PyResult<String> {
    let kind = match kind {
        Some("agent") => Some(tramuri_core::CapabilityKind::Agent),
        Some("skill") => Some(tramuri_core::CapabilityKind::Skill),
        Some("rule") => Some(tramuri_core::CapabilityKind::Rule),
        Some("hook") => Some(tramuri_core::CapabilityKind::Hook),
        Some("mcp_server") | Some("mcp-server") => Some(tramuri_core::CapabilityKind::McpServer),
        Some(value) => {
            return Err(TramuriError::new_err(format!(
                "unknown capability type: {value}"
            )));
        }
        None => None,
    };
    let snapshot = capability_list(path, kind).map_err(map_core_error)?;
    serde_json::to_string(&snapshot).map_err(map_serialization_error)
}

fn parse_knowledge_type(value: &str) -> PyResult<KnowledgeType> {
    match value {
        "fact" => Ok(KnowledgeType::Fact),
        "decision" => Ok(KnowledgeType::Decision),
        "inference" => Ok(KnowledgeType::Inference),
        "question" => Ok(KnowledgeType::Question),
        _ => Err(TramuriError::new_err(format!(
            "unknown knowledge type: {value}"
        ))),
    }
}

#[pyfunction]
#[pyo3(signature = (path, kind, statement, sources, human_input=false, supersedes=None))]
fn _knowledge_add_json(
    path: &str,
    kind: &str,
    statement: &str,
    sources: Vec<String>,
    human_input: bool,
    supersedes: Option<String>,
) -> PyResult<String> {
    let record = knowledge_add(
        path,
        KnowledgeInput {
            kind: parse_knowledge_type(kind)?,
            statement: statement.to_owned(),
            sources,
            human_input,
            supersedes,
        },
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&record).map_err(map_serialization_error)
}

#[pyfunction]
#[pyo3(signature = (path, kind=None, include_history=false))]
fn _knowledge_list_json(path: &str, kind: Option<&str>, include_history: bool) -> PyResult<String> {
    let records = knowledge_list(
        path,
        KnowledgeFilter {
            kind: kind.map(parse_knowledge_type).transpose()?,
            include_history,
        },
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&records).map_err(map_serialization_error)
}

#[pyfunction]
fn _knowledge_withdraw_json(path: &str, id: &str) -> PyResult<String> {
    let record = knowledge_withdraw(path, id).map_err(map_core_error)?;
    serde_json::to_string(&record).map_err(map_serialization_error)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MissionCriterionJson {
    description: String,
    #[serde(default = "required_by_default")]
    required: bool,
    #[serde(default)]
    evidence_expectation: Option<String>,
}

#[derive(Deserialize)]
struct ReadinessAssessmentJson {
    mission_id: String,
    activity: String,
    decision: ReadinessDecision,
    #[serde(default)]
    criteria_applied: Vec<String>,
    #[serde(default)]
    gaps: Vec<String>,
    #[serde(default)]
    blockers: Vec<String>,
    #[serde(default)]
    accepted_uncertainties: Vec<String>,
    actor_reference: Option<String>,
}

fn required_by_default() -> bool {
    true
}

fn parse_criteria_json(value: &str) -> PyResult<Vec<CriterionInput>> {
    let criteria: Vec<MissionCriterionJson> = serde_json::from_str(value)
        .map_err(|error| TramuriError::new_err(format!("invalid criteria JSON: {error}")))?;
    Ok(criteria
        .into_iter()
        .map(|criterion| CriterionInput {
            description: criterion.description,
            required: criterion.required,
            evidence_expectation: criterion.evidence_expectation,
        })
        .collect())
}

#[pyfunction]
fn _mission_create_json(
    path: &str,
    objective: &str,
    context: Option<&str>,
    criteria_json: &str,
) -> PyResult<String> {
    let mission = mission_create(
        path,
        MissionInput {
            objective: objective.to_owned(),
            context: context.map(str::to_owned),
            criteria: parse_criteria_json(criteria_json)?,
        },
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&mission).map_err(map_serialization_error)
}

#[pyfunction]
#[pyo3(signature = (path, objective, perspective=None, context=None))]
fn _mission_start_json(
    path: &str,
    objective: &str,
    perspective: Option<&str>,
    context: Option<&str>,
) -> PyResult<String> {
    let explicit = perspective.map(parse_mission_perspective).transpose()?;
    let result = mission_start(path, objective, context, explicit).map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

fn parse_define_step(value: &str) -> PyResult<DefineStepId> {
    match value {
        "objective_context" => Ok(DefineStepId::ObjectiveContext),
        "business_discovery" => Ok(DefineStepId::BusinessDiscovery),
        "requirements" => Ok(DefineStepId::Requirements),
        "knowledge_synthesis" => Ok(DefineStepId::KnowledgeSynthesis),
        "product_brief" => Ok(DefineStepId::ProductBrief),
        _ => Err(TramuriError::new_err("unknown DEFINE step")),
    }
}

#[pyfunction]
fn _define_start_json(path: &str, mission_id: &str) -> PyResult<String> {
    serde_json::to_string(&define_start(path, mission_id).map_err(map_core_error)?)
        .map_err(map_serialization_error)
}

#[pyfunction]
fn _define_status_json(path: &str, mission_id: &str) -> PyResult<String> {
    serde_json::to_string(&define_status(path, mission_id).map_err(map_core_error)?)
        .map_err(map_serialization_error)
}

#[pyfunction]
fn _define_next_json(path: &str, mission_id: &str) -> PyResult<String> {
    serde_json::to_string(&define_next(path, mission_id).map_err(map_core_error)?)
        .map_err(map_serialization_error)
}

#[pyfunction]
#[pyo3(signature = (path, mission_id, step, payload_json, expected_revision=None, host_reported_skill_id=None))]
fn _define_submit_json(
    path: &str,
    mission_id: &str,
    step: &str,
    payload_json: &str,
    expected_revision: Option<u64>,
    host_reported_skill_id: Option<&str>,
) -> PyResult<String> {
    let payload: serde_json::Value = serde_json::from_str(payload_json)
        .map_err(|error| TramuriError::new_err(format!("invalid DEFINE payload JSON: {error}")))?;
    let result = define_submit(
        path,
        mission_id,
        parse_define_step(step)?,
        payload,
        expected_revision,
        host_reported_skill_id,
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
#[pyo3(signature = (path, mission_id, step, payload_json, reason, declared_author, origin, expected_revision=None, authorize_decision_changes=false, host_reported_skill_id=None))]
#[allow(clippy::too_many_arguments)] // Keep the review provenance explicit in the SDK API.
fn _define_review_json(
    path: &str,
    mission_id: &str,
    step: &str,
    payload_json: &str,
    reason: &str,
    declared_author: &str,
    origin: &str,
    expected_revision: Option<u64>,
    authorize_decision_changes: bool,
    host_reported_skill_id: Option<&str>,
) -> PyResult<String> {
    let payload: serde_json::Value = serde_json::from_str(payload_json)
        .map_err(|error| TramuriError::new_err(format!("invalid DEFINE review JSON: {error}")))?;
    let result = define_review(
        path,
        mission_id,
        parse_define_step(step)?,
        payload,
        reason,
        declared_author,
        origin,
        expected_revision,
        authorize_decision_changes,
        host_reported_skill_id,
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
fn _define_history_json(path: &str, mission_id: &str, step: Option<&str>) -> PyResult<String> {
    let step = step.map(parse_define_step).transpose()?;
    let result = define_artifact_history(path, mission_id, step).map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
fn _define_brief_assess_json(
    path: &str,
    mission_id: &str,
    assessment_json: &str,
) -> PyResult<String> {
    let assessment: serde_json::Value = serde_json::from_str(assessment_json).map_err(|error| {
        TramuriError::new_err(format!("invalid brief assessment JSON: {error}"))
    })?;
    let result = define_brief_assess(path, mission_id, assessment).map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
fn _define_brief_assessment_get_json(path: &str, mission_id: &str) -> PyResult<String> {
    let result = define_brief_assessment_get(path, mission_id).map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
fn _define_question_request_json(
    path: &str,
    mission_id: &str,
    step: &str,
    rationale: &str,
    alternatives: Vec<String>,
    material_decision: bool,
) -> PyResult<String> {
    let result = define_question_request(
        path,
        mission_id,
        parse_define_step(step)?,
        rationale,
        &alternatives,
        material_decision,
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
fn _define_question_answer_json(
    path: &str,
    mission_id: &str,
    question_id: &str,
    answer: &str,
    explicit_user_answer: bool,
) -> PyResult<String> {
    let result =
        define_question_answer(path, mission_id, question_id, answer, explicit_user_answer)
            .map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

fn parse_mission_perspective(value: &str) -> PyResult<MissionPerspective> {
    MissionPerspective::parse(value).ok_or_else(|| {
        TramuriError::new_err("perspective must be product_engineer, fde, mixed, or other")
    })
}

#[pyfunction]
fn _profile_get_json(path: &str) -> PyResult<String> {
    serde_json::to_string(&profile_get(path).map_err(map_core_error)?)
        .map_err(map_serialization_error)
}

#[pyfunction]
fn _profile_set_json(path: &str, perspective: &str, ask: &str) -> PyResult<String> {
    let default_work_perspective = match perspective {
        "product_engineer" => WorkProfilePreference::ProductEngineer,
        "fde" => WorkProfilePreference::Fde,
        "mixed" => WorkProfilePreference::Mixed,
        "other" => WorkProfilePreference::Other,
        "auto" => WorkProfilePreference::Auto,
        _ => return Err(TramuriError::new_err("invalid work perspective preference")),
    };
    let ask_work_perspective = match ask {
        "ambiguous" => AskWorkPerspective::Ambiguous,
        "always" => AskWorkPerspective::Always,
        _ => return Err(TramuriError::new_err("ask must be ambiguous or always")),
    };
    let profile = profile_set(
        path,
        WorkProfile {
            default_work_perspective,
            ask_work_perspective,
        },
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&profile).map_err(map_serialization_error)
}

#[pyfunction]
fn _mission_workflow_get_json(path: &str, mission_id: &str) -> PyResult<String> {
    serde_json::to_string(&mission_workflow_get(path, mission_id).map_err(map_core_error)?)
        .map_err(map_serialization_error)
}

#[pyfunction]
fn _mission_workflow_set_json(
    path: &str,
    mission_id: &str,
    perspective: &str,
    source: &str,
    next_activity: Option<&str>,
) -> PyResult<String> {
    let perspective = parse_mission_perspective(perspective)?;
    let source = match source {
        "explicit" => tramuri_core::IntentRouteSource::Explicit,
        "current_intent" => tramuri_core::IntentRouteSource::CurrentIntent,
        "profile" => tramuri_core::IntentRouteSource::Profile,
        "inferred" => tramuri_core::IntentRouteSource::Inferred,
        _ => return Err(TramuriError::new_err("invalid perspective source")),
    };
    let workflow = mission_workflow_set(path, mission_id, perspective, source, next_activity)
        .map_err(map_core_error)?;
    serde_json::to_string(&workflow).map_err(map_serialization_error)
}

#[pyfunction]
fn _mission_workflow_activity_set_json(
    path: &str,
    mission_id: &str,
    next_activity: Option<&str>,
) -> PyResult<String> {
    let workflow =
        mission_workflow_activity_set(path, mission_id, next_activity).map_err(map_core_error)?;
    serde_json::to_string(&workflow).map_err(map_serialization_error)
}

#[pyfunction]
fn _execution_transition_json(path: &str, execution_id: &str, state: &str) -> PyResult<String> {
    let state = match state {
        "completed" => ExecutionState::Completed,
        "blocked" => ExecutionState::Blocked,
        "failed" => ExecutionState::Failed,
        "cancelled" => ExecutionState::Cancelled,
        _ => {
            return Err(TramuriError::new_err(
                "terminal state must be completed, blocked, failed, or cancelled",
            ));
        }
    };
    serde_json::to_string(&execution_transition(path, execution_id, state).map_err(map_core_error)?)
        .map_err(map_serialization_error)
}

#[pyfunction]
fn _readiness_assess_json(path: &str, input_json: &str) -> PyResult<String> {
    let input: ReadinessAssessmentJson = serde_json::from_str(input_json)
        .map_err(|error| TramuriError::new_err(format!("invalid readiness JSON: {error}")))?;
    let assessment = readiness_assess(
        path,
        ReadinessAssessmentInput {
            mission_id: input.mission_id,
            activity: input.activity,
            decision: input.decision,
            criteria_applied: input.criteria_applied,
            gaps: input.gaps,
            blockers: input.blockers,
            accepted_uncertainties: input.accepted_uncertainties,
            actor_reference: input.actor_reference,
        },
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&assessment).map_err(map_serialization_error)
}

#[pyfunction]
fn _readiness_get_json(path: &str, mission_id: &str) -> PyResult<String> {
    let readiness = readiness_get(path, mission_id).map_err(map_core_error)?;
    serde_json::to_string(&readiness).map_err(map_serialization_error)
}

#[pyfunction]
fn _readiness_history_json(path: &str, mission_id: &str) -> PyResult<String> {
    let history = readiness_history(path, mission_id).map_err(map_core_error)?;
    serde_json::to_string(&history).map_err(map_serialization_error)
}

#[pyfunction]
fn _mission_get_json(path: &str, mission_id: &str) -> PyResult<String> {
    let mission = mission_get(path, mission_id).map_err(map_core_error)?;
    serde_json::to_string(&mission).map_err(map_serialization_error)
}

#[pyfunction]
fn _mission_list_json(path: &str) -> PyResult<String> {
    let missions = mission_list(path).map_err(map_core_error)?;
    serde_json::to_string(&missions).map_err(map_serialization_error)
}

fn parse_link_actor_kind(value: &str) -> PyResult<ActorKind> {
    match value {
        "human" => Ok(ActorKind::Human),
        "agent" => Ok(ActorKind::Agent),
        "tool" => Ok(ActorKind::Tool),
        _ => Err(TramuriError::new_err(
            "actor_kind must be human, agent, or tool",
        )),
    }
}

#[pyfunction]
#[pyo3(signature = (path, mission_id, knowledge_id, actor_kind="human", actor_reference=None))]
fn _mission_knowledge_link_json(
    path: &str,
    mission_id: &str,
    knowledge_id: &str,
    actor_kind: &str,
    actor_reference: Option<&str>,
) -> PyResult<String> {
    let relationships = mission_knowledge_link(
        path,
        mission_id,
        knowledge_id,
        parse_link_actor_kind(actor_kind)?,
        actor_reference,
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&relationships).map_err(map_serialization_error)
}

#[pyfunction]
fn _mission_knowledge_unlink_json(
    path: &str,
    mission_id: &str,
    knowledge_id: &str,
) -> PyResult<String> {
    let removed =
        mission_knowledge_unlink(path, mission_id, knowledge_id).map_err(map_core_error)?;
    serde_json::to_string(&removed).map_err(map_serialization_error)
}

#[pyfunction]
#[pyo3(signature = (path, mission_id, capability_id, actor_kind="human", actor_reference=None))]
fn _mission_capability_link_json(
    path: &str,
    mission_id: &str,
    capability_id: &str,
    actor_kind: &str,
    actor_reference: Option<&str>,
) -> PyResult<String> {
    let relationships = mission_capability_link(
        path,
        mission_id,
        capability_id,
        parse_link_actor_kind(actor_kind)?,
        actor_reference,
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&relationships).map_err(map_serialization_error)
}

#[pyfunction]
fn _mission_capability_unlink_json(
    path: &str,
    mission_id: &str,
    capability_id: &str,
) -> PyResult<String> {
    let removed =
        mission_capability_unlink(path, mission_id, capability_id).map_err(map_core_error)?;
    serde_json::to_string(&removed).map_err(map_serialization_error)
}

#[pyfunction]
fn _mission_relationships_json(path: &str, mission_id: &str) -> PyResult<String> {
    let relationships = mission_relationships(path, mission_id).map_err(map_core_error)?;
    serde_json::to_string(&relationships).map_err(map_serialization_error)
}

#[pyfunction]
fn _mission_resume_json(path: &str, mission_id: &str) -> PyResult<String> {
    let result = mission_resume(path, mission_id).map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
fn _status_json(path: &str) -> PyResult<String> {
    let status = status_snapshot(path).map_err(map_core_error)?;
    serde_json::to_string(&status).map_err(map_serialization_error)
}

#[pyfunction]
fn _mission_close_json(path: &str, mission_id: &str) -> PyResult<String> {
    let mission = mission_close(path, mission_id).map_err(map_core_error)?;
    serde_json::to_string(&mission).map_err(map_serialization_error)
}

#[pyfunction]
fn _mission_contract_revise_json(
    path: &str,
    mission_id: &str,
    criteria_json: &str,
) -> PyResult<String> {
    let mission = mission_contract_revise(path, mission_id, parse_criteria_json(criteria_json)?)
        .map_err(map_core_error)?;
    serde_json::to_string(&mission).map_err(map_serialization_error)
}

#[pyfunction]
#[pyo3(signature = (path, mission_id, contract_revision, actor_kind, state, actor_reference=None, notes=None, work_perspective=None))]
#[allow(clippy::too_many_arguments)] // Keep the Python SDK's explicit, keyword-friendly API.
fn _execution_record_json(
    path: &str,
    mission_id: &str,
    contract_revision: u32,
    actor_kind: &str,
    state: &str,
    actor_reference: Option<&str>,
    notes: Option<&str>,
    work_perspective: Option<&str>,
) -> PyResult<String> {
    let actor_kind = match actor_kind {
        "human" => ActorKind::Human,
        "agent" => ActorKind::Agent,
        "tool" => ActorKind::Tool,
        _ => {
            return Err(TramuriError::new_err(
                "actor_kind must be human, agent, or tool",
            ));
        }
    };
    let state = match state {
        "in_progress" => ExecutionState::InProgress,
        "completed" => ExecutionState::Completed,
        "blocked" => ExecutionState::Blocked,
        "failed" => ExecutionState::Failed,
        "cancelled" => ExecutionState::Cancelled,
        _ => return Err(TramuriError::new_err("invalid execution state")),
    };
    let work_perspective = match work_perspective {
        None => None,
        Some("product_engineer") => Some(WorkPerspective::ProductEngineer),
        Some("fde") => Some(WorkPerspective::Fde),
        Some("other") => Some(WorkPerspective::Other),
        Some(_) => {
            return Err(TramuriError::new_err(
                "work_perspective must be product_engineer, fde, or other",
            ));
        }
    };
    let result = execution_record(
        path,
        ExecutionInput {
            mission_id: mission_id.to_owned(),
            contract_revision,
            actor_kind,
            actor_reference: actor_reference.map(str::to_owned),
            work_perspective,
            state,
            notes: notes.map(str::to_owned),
        },
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
fn _evidence_add_json(
    path: &str,
    mission_id: &str,
    execution_id: &str,
    kind: &str,
    locator: &str,
) -> PyResult<String> {
    let kind = match kind {
        "project_file" => EvidenceKind::ProjectFile,
        "external_reference" => EvidenceKind::ExternalReference,
        "human_attestation" => EvidenceKind::HumanAttestation,
        _ => return Err(TramuriError::new_err("invalid evidence kind")),
    };
    let result = evidence_add(
        path,
        EvidenceInput {
            mission_id: mission_id.to_owned(),
            execution_id: execution_id.to_owned(),
            kind,
            locator: locator.to_owned(),
        },
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
fn _execution_list_json(path: &str, mission_id: &str) -> PyResult<String> {
    let result = execution_list(path, mission_id).map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
fn _execution_get_json(path: &str, execution_id: &str) -> PyResult<String> {
    let result = execution_get(path, execution_id).map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
fn _evidence_list_json(path: &str, mission_id: &str) -> PyResult<String> {
    let result = evidence_list(path, mission_id).map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[pyfunction]
fn _evidence_get_json(path: &str, evidence_id: &str) -> PyResult<String> {
    let result = evidence_get(path, evidence_id).map_err(map_core_error)?;
    serde_json::to_string(&result).map_err(map_serialization_error)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CriterionAssessmentJson {
    mission_id: String,
    contract_revision: u32,
    criterion_id: String,
    result: String,
    evidence_ids: Vec<String>,
    verifier_kind: String,
    verifier_reference: String,
    explanation: Option<String>,
}

#[pyfunction]
fn _outcome_assess_json(path: &str, input_json: &str) -> PyResult<String> {
    let input: CriterionAssessmentJson = serde_json::from_str(input_json)
        .map_err(|error| TramuriError::new_err(format!("invalid assessment JSON: {error}")))?;
    let result = match input.result.as_str() {
        "met" => CriterionAssessmentResult::Met,
        "not_met" => CriterionAssessmentResult::NotMet,
        "unverified" => CriterionAssessmentResult::Unverified,
        _ => {
            return Err(TramuriError::new_err(
                "result must be met, not_met, or unverified",
            ));
        }
    };
    let verifier_kind = match input.verifier_kind.as_str() {
        "human" => VerifierKind::Human,
        "mechanism" => VerifierKind::Mechanism,
        _ => {
            return Err(TramuriError::new_err(
                "verifier_kind must be human or mechanism",
            ));
        }
    };
    let assessment = outcome_assess(
        path,
        CriterionAssessmentInput {
            mission_id: input.mission_id,
            contract_revision: input.contract_revision,
            criterion_id: input.criterion_id,
            result,
            evidence_ids: input.evidence_ids,
            verifier_kind,
            verifier_reference: input.verifier_reference,
            explanation: input.explanation,
        },
    )
    .map_err(map_core_error)?;
    serde_json::to_string(&assessment).map_err(map_serialization_error)
}

#[pyfunction]
fn _outcome_get_json(
    path: &str,
    mission_id: &str,
    contract_revision: Option<u32>,
) -> PyResult<String> {
    let outcome = outcome_get(path, mission_id, contract_revision).map_err(map_core_error)?;
    serde_json::to_string(&outcome).map_err(map_serialization_error)
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_cli_main, module)?)?;
    module.add_function(wrap_pyfunction!(_init_json, module)?)?;
    module.add_function(wrap_pyfunction!(_scan_json, module)?)?;
    module.add_function(wrap_pyfunction!(_capability_snapshot_json, module)?)?;
    module.add_function(wrap_pyfunction!(_knowledge_add_json, module)?)?;
    module.add_function(wrap_pyfunction!(_knowledge_list_json, module)?)?;
    module.add_function(wrap_pyfunction!(_knowledge_withdraw_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_create_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_start_json, module)?)?;
    module.add_function(wrap_pyfunction!(_define_start_json, module)?)?;
    module.add_function(wrap_pyfunction!(_define_status_json, module)?)?;
    module.add_function(wrap_pyfunction!(_define_next_json, module)?)?;
    module.add_function(wrap_pyfunction!(_define_submit_json, module)?)?;
    module.add_function(wrap_pyfunction!(_define_review_json, module)?)?;
    module.add_function(wrap_pyfunction!(_define_history_json, module)?)?;
    module.add_function(wrap_pyfunction!(_define_brief_assess_json, module)?)?;
    module.add_function(wrap_pyfunction!(_define_brief_assessment_get_json, module)?)?;
    module.add_function(wrap_pyfunction!(_define_question_request_json, module)?)?;
    module.add_function(wrap_pyfunction!(_define_question_answer_json, module)?)?;
    module.add_function(wrap_pyfunction!(_profile_get_json, module)?)?;
    module.add_function(wrap_pyfunction!(_profile_set_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_workflow_get_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_workflow_set_json, module)?)?;
    module.add_function(wrap_pyfunction!(
        _mission_workflow_activity_set_json,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(_execution_transition_json, module)?)?;
    module.add_function(wrap_pyfunction!(_readiness_assess_json, module)?)?;
    module.add_function(wrap_pyfunction!(_readiness_get_json, module)?)?;
    module.add_function(wrap_pyfunction!(_readiness_history_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_get_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_list_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_knowledge_link_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_knowledge_unlink_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_capability_link_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_capability_unlink_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_relationships_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_resume_json, module)?)?;
    module.add_function(wrap_pyfunction!(_status_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_close_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_contract_revise_json, module)?)?;
    module.add_function(wrap_pyfunction!(_execution_record_json, module)?)?;
    module.add_function(wrap_pyfunction!(_evidence_add_json, module)?)?;
    module.add_function(wrap_pyfunction!(_execution_list_json, module)?)?;
    module.add_function(wrap_pyfunction!(_execution_get_json, module)?)?;
    module.add_function(wrap_pyfunction!(_evidence_list_json, module)?)?;
    module.add_function(wrap_pyfunction!(_evidence_get_json, module)?)?;
    module.add_function(wrap_pyfunction!(_outcome_assess_json, module)?)?;
    module.add_function(wrap_pyfunction!(_outcome_get_json, module)?)?;
    module.add("TramuriError", module.py().get_type::<TramuriError>())?;
    Ok(())
}
