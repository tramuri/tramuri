use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use serde::Deserialize;
use tramuri_core::{
    ActorKind, CriterionAssessmentInput, CriterionAssessmentResult, CriterionInput, EvidenceInput,
    EvidenceKind, ExecutionInput, ExecutionState, KnowledgeFilter, KnowledgeInput, KnowledgeType,
    MissionInput, TramuriError as CoreError, VerifierKind, evidence_add, evidence_get,
    evidence_list, execution_get, execution_list, execution_record, init, knowledge_add,
    knowledge_list, knowledge_withdraw, mission_close, mission_contract_revise, mission_create,
    mission_get, outcome_assess, outcome_get, scan,
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
fn _mission_get_json(path: &str, mission_id: &str) -> PyResult<String> {
    let mission = mission_get(path, mission_id).map_err(map_core_error)?;
    serde_json::to_string(&mission).map_err(map_serialization_error)
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
#[pyo3(signature = (path, mission_id, contract_revision, actor_kind, state, actor_reference=None, notes=None))]
fn _execution_record_json(
    path: &str,
    mission_id: &str,
    contract_revision: u32,
    actor_kind: &str,
    state: &str,
    actor_reference: Option<&str>,
    notes: Option<&str>,
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
    let result = execution_record(
        path,
        ExecutionInput {
            mission_id: mission_id.to_owned(),
            contract_revision,
            actor_kind,
            actor_reference: actor_reference.map(str::to_owned),
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
    module.add_function(wrap_pyfunction!(_knowledge_add_json, module)?)?;
    module.add_function(wrap_pyfunction!(_knowledge_list_json, module)?)?;
    module.add_function(wrap_pyfunction!(_knowledge_withdraw_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_create_json, module)?)?;
    module.add_function(wrap_pyfunction!(_mission_get_json, module)?)?;
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
