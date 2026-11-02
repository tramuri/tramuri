//! Fixed, host-driven Product Engineering DEFINE workflow.
//!
//! The Core authorizes and validates transitions; it never launches a model, skill, or tool.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::storage::open_database;
use crate::{Result, TramuriError};

pub const DEFINE_WORKFLOW_VERSION: &str = "tramuri.define.v1";
pub const DEFINE_BRIEF_RUBRIC_VERSION: &str = "tramuri.product-brief-rubric.v1";
const MAX_QUESTIONS: u32 = 2;
const BRIEF_ASSESSMENT_CRITERIA: [(&str, &str); 7] = [
    (
        "goal_alignment",
        "The brief objective and main content align with the user's original intent.",
    ),
    (
        "target_consistency",
        "The target audience is consistent between accepted discovery and the Product Brief.",
    ),
    (
        "evidence_assumption_separation",
        "User-provided or observed information is distinguished from hypotheses and assumptions.",
    ),
    (
        "success_criteria_fit",
        "Candidate success criteria meaningfully assess the objective without claiming an outcome.",
    ),
    (
        "requirements_traceability",
        "Brief requirements trace to the accepted requirements artifact and declared dependencies.",
    ),
    (
        "decision_authorization",
        "Only explicitly approved decisions are presented as approved; proposals remain proposals.",
    ),
    (
        "internal_consistency",
        "The Product Brief sections do not materially contradict each other.",
    ),
];
const BRIEF_ASSESSMENT_RESULTS: [&str; 3] = ["pass", "concern", "not_assessed"];
const BRIEF_ASSESSMENT_OVERALL_RESULTS: [&str; 3] = ["pass", "concerns", "incomplete"];
const BRIEF_ASSESSMENT_EVALUATION_BOUNDARY: &str = "host_semantic_assessment_not_core_verified";
const BRIEF_ASSESSMENT_MAX_LIMITATIONS: usize = 5;
const BRIEF_ASSESSMENT_MAX_LIMITATION_CHARACTERS: usize = 500;
const BRIEF_ASSESSMENT_MAX_RATIONALE_CHARACTERS: usize = 1000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DefineStepId {
    ObjectiveContext,
    BusinessDiscovery,
    Requirements,
    KnowledgeSynthesis,
    ProductBrief,
}

impl DefineStepId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ObjectiveContext => "objective_context",
            Self::BusinessDiscovery => "business_discovery",
            Self::Requirements => "requirements",
            Self::KnowledgeSynthesis => "knowledge_synthesis",
            Self::ProductBrief => "product_brief",
        }
    }

    fn order(self) -> u8 {
        match self {
            Self::ObjectiveContext => 1,
            Self::BusinessDiscovery => 2,
            Self::Requirements => 3,
            Self::KnowledgeSynthesis => 4,
            Self::ProductBrief => 5,
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "objective_context" => Some(Self::ObjectiveContext),
            "business_discovery" => Some(Self::BusinessDiscovery),
            "requirements" => Some(Self::Requirements),
            "knowledge_synthesis" => Some(Self::KnowledgeSynthesis),
            "product_brief" => Some(Self::ProductBrief),
            _ => None,
        }
    }

    fn all() -> [Self; 5] {
        [
            Self::ObjectiveContext,
            Self::BusinessDiscovery,
            Self::Requirements,
            Self::KnowledgeSynthesis,
            Self::ProductBrief,
        ]
    }

    fn skill(self) -> &'static str {
        match self {
            Self::ObjectiveContext => "tramuri-pe-objective-context",
            Self::BusinessDiscovery => "tramuri-pe-business-discovery",
            Self::Requirements => "tramuri-pe-requirements",
            Self::KnowledgeSynthesis => "tramuri-pe-evidence-hypotheses",
            Self::ProductBrief => "tramuri-pe-product-brief",
        }
    }

    fn required_fields(self) -> &'static [&'static str] {
        match self {
            Self::ObjectiveContext => &[
                "original_intent",
                "interpreted_objective",
                "known_information",
                "hypotheses",
                "gaps",
            ],
            Self::BusinessDiscovery => &[
                "problem",
                "target_audience",
                "business_objectives",
                "value_proposition",
                "uncertainties",
                "references",
            ],
            Self::Requirements => &[
                "functional_requirements",
                "non_functional_requirements",
                "constraints",
                "dependencies",
                "acceptance_criteria",
                "success_criteria",
                "gaps",
                "references",
            ],
            Self::KnowledgeSynthesis => &["classifications", "references", "open_questions"],
            Self::ProductBrief => &[
                "objective",
                "problem",
                "target_audience",
                "value_proposition",
                "main_journey",
                "requirements",
                "mvp_scope",
                "exclusions",
                "risks",
                "assumptions",
                "approved_decisions",
                "candidate_success_criteria",
                "next_action",
                "references",
            ],
        }
    }

    fn string_fields(self) -> &'static [&'static str] {
        match self {
            Self::ObjectiveContext => &["original_intent", "interpreted_objective"],
            Self::BusinessDiscovery => &["problem", "value_proposition"],
            Self::Requirements | Self::KnowledgeSynthesis => &[],
            Self::ProductBrief => &["objective", "problem", "value_proposition", "next_action"],
        }
    }

    fn validation_rules(self) -> Vec<String> {
        let string_fields = self.string_fields();
        let array_fields = self
            .required_fields()
            .iter()
            .filter(|field| !string_fields.contains(field))
            .copied()
            .collect::<Vec<_>>();
        let mut rules = Vec::new();
        if !string_fields.is_empty() {
            rules.push(format!(
                "these fields must be non-empty JSON strings: {}",
                string_fields.join(", ")
            ));
        }
        if !array_fields.is_empty() {
            rules.push(format!(
                "these fields must be JSON arrays (empty arrays are structurally accepted): {}",
                array_fields.join(", ")
            ));
        }
        if self == Self::KnowledgeSynthesis {
            rules.push(
                "each classifications item must have a supported `kind` and non-empty `statement`; kinds: user_provided, observed, hypothesis, recommendation, approved_decision, open_question, risk".to_owned(),
            );
        }
        if self.order() == 1 {
            rules.push(
                "references may be omitted at objective_context; references are reserved for earlier accepted DEFINE artifacts, while external source metadata belongs in provenance".to_owned(),
            );
        } else {
            let previous = Self::all()[usize::from(self.order() - 2)].as_str();
            rules.push(format!(
                "references must include the latest accepted `{previous}` artifact as {{\"step\":\"{previous}\",\"revision\":<revision>}}; every supplied reference must point to an accepted earlier step"
            ));
        }
        rules
    }

    fn contract_version(self) -> String {
        format!("tramuri.define.{}.v1", self.as_str().replace('_', "-"))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DefineStepState {
    Pending,
    InProgress,
    Blocked,
    ValidationFailed,
    Completed,
    Stale,
}

impl DefineStepState {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "in_progress" => Some(Self::InProgress),
            "blocked" => Some(Self::Blocked),
            "validation_failed" => Some(Self::ValidationFailed),
            "completed" => Some(Self::Completed),
            "stale" => Some(Self::Stale),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DefineArtifact {
    pub id: String,
    pub step: DefineStepId,
    pub revision: u32,
    pub contract_version: String,
    pub payload: Value,
    pub provenance: Value,
    pub dependencies: Value,
    pub expected_skill_id: String,
    pub host_reported_skill_id: Option<String>,
    pub created_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineArtifactReference {
    pub step: DefineStepId,
    pub revision: u32,
    pub field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineBriefCriterionAssessment {
    pub criterion: String,
    pub result: String,
    pub rationale: String,
    pub references: Vec<DefineArtifactReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineBriefAssessment {
    pub schema: String,
    #[serde(default)]
    pub mission_id: String,
    pub brief_artifact_id: String,
    pub brief_revision: u32,
    pub evaluator: String,
    pub rubric_version: String,
    pub overall: String,
    pub criteria: Vec<DefineBriefCriterionAssessment>,
    pub limitations: Vec<String>,
    pub evaluation_boundary: String,
    #[serde(default)]
    pub occurred_at_unix_ms: i64,
}

/// Read-only contract and draft returned to a host before it reviews the current Product Brief.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineBriefAssessmentContract {
    pub contract_schema: String,
    pub assessment_schema: String,
    pub mission_id: String,
    pub brief_artifact_id: String,
    pub brief_revision: u32,
    pub rubric_version: String,
    pub criteria: Vec<DefineBriefAssessmentCriterionContract>,
    pub criterion_results: Vec<String>,
    pub overall_results: Vec<String>,
    pub reference_shape: Value,
    pub maximum_limitations: usize,
    pub maximum_limitation_characters: usize,
    pub maximum_rationale_characters: usize,
    pub evaluation_boundary: String,
    pub assessment_template: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineBriefAssessmentCriterionContract {
    pub id: String,
    pub guidance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineStep {
    pub id: DefineStepId,
    pub order: u8,
    pub state: DefineStepState,
    pub contract_version: String,
    pub attempts: u32,
    pub artifact_id: Option<String>,
    pub validation_errors: Vec<DefineValidationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineValidationError {
    pub code: String,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineStatus {
    pub schema: String,
    pub mission_id: String,
    pub workflow_version: String,
    pub state: DefineState,
    pub current_step: Option<DefineStepId>,
    pub steps: Vec<DefineStep>,
    pub events: Vec<DefineEvent>,
    pub completed: u8,
    pub total: u8,
    pub clarification_count: u32,
    pub clarification_maximum: u32,
    pub revision: u64,
    pub started_at_unix_ms: i64,
    pub updated_at_unix_ms: i64,
    pub completed_at_unix_ms: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DefineState {
    InProgress,
    Blocked,
    ValidationFailed,
    Stale,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DefineNext {
    pub schema: String,
    pub mission_id: String,
    pub workflow_version: String,
    pub state: DefineState,
    pub current_step: Option<DefineStepContract>,
    pub accepted_inputs: Vec<DefineArtifact>,
    pub brief_assessment_contract: Option<DefineBriefAssessmentContract>,
    pub questions: Vec<DefineQuestion>,
    pub progress: DefineProgress,
    pub clarification_budget: DefineClarificationBudget,
    pub next_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineStepContract {
    pub id: DefineStepId,
    pub order: u8,
    pub contract_version: String,
    pub skill: String,
    pub skill_execution_verification: String,
    pub required_fields: Vec<String>,
    pub preconditions: Vec<String>,
    pub validation_rules: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineProgress {
    pub completed: u8,
    pub total: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineClarificationBudget {
    pub used: u32,
    pub maximum: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineSubmitResult {
    pub schema: String,
    pub accepted: bool,
    pub mission_id: String,
    pub step: DefineStepId,
    pub step_state: DefineStepState,
    pub accepted_artifact_id: Option<String>,
    pub progress: DefineProgress,
    pub next_step: Option<DefineStepId>,
    pub workflow_state: DefineState,
    pub errors: Vec<DefineValidationError>,
    pub retry_allowed: bool,
    pub next_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineQuestion {
    pub id: String,
    pub step: DefineStepId,
    pub rationale: String,
    pub alternatives: Vec<String>,
    pub answer: Option<String>,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefineEvent {
    pub id: String,
    pub event_type: String,
    pub step: Option<DefineStepId>,
    pub occurred_at_unix_ms: i64,
}

pub fn define_start(path: impl AsRef<Path>, mission_id: &str) -> Result<DefineStatus> {
    let mut connection = open_database(path.as_ref())?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let perspective: Option<String> = transaction
        .query_row(
            "SELECT perspective FROM mission_workflows WHERE mission_id=?1",
            [mission_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    if perspective.as_deref() != Some("product_engineer") {
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM missions WHERE id=?1)",
            [mission_id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(TramuriError::MissionNotFound(mission_id.to_owned()));
        }
        return Err(TramuriError::DefineRequiresProductEngineer(
            mission_id.to_owned(),
        ));
    }
    let existing: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM define_workflows WHERE mission_id=?1)",
        [mission_id],
        |r| r.get(0),
    )?;
    if !existing {
        let now = now_ms()?;
        let workflow_id = Uuid::new_v4().to_string();
        let source: Option<String> = transaction.query_row(
            "SELECT perspective_source FROM mission_workflows WHERE mission_id=?1",
            [mission_id],
            |r| r.get(0),
        )?;
        transaction.execute("INSERT INTO define_workflows (id, mission_id, workflow_version, perspective_source, state, current_step_id, started_at_unix_ms, updated_at_unix_ms) VALUES (?1,?2,?3,?4,'in_progress','objective_context',?5,?5)", params![workflow_id, mission_id, DEFINE_WORKFLOW_VERSION, source, now])?;
        for step in DefineStepId::all() {
            let state = if step == DefineStepId::ObjectiveContext {
                "in_progress"
            } else {
                "pending"
            };
            transaction.execute("INSERT INTO define_steps (workflow_id, step_id, position, state, contract_version, updated_at_unix_ms) VALUES (?1,?2,?3,?4,?5,?6)", params![workflow_id, step.as_str(), step.order(), state, step.contract_version(), now])?;
        }
        insert_event(
            &transaction,
            &workflow_id,
            Some(DefineStepId::ObjectiveContext),
            "define_started",
            None,
            json!({"workflow_version": DEFINE_WORKFLOW_VERSION}),
            now,
        )?;
        insert_event(
            &transaction,
            &workflow_id,
            Some(DefineStepId::ObjectiveContext),
            "step_started",
            None,
            json!({}),
            now,
        )?;
    }
    transaction.commit()?;
    define_status(path, mission_id)
}

pub fn define_status(path: impl AsRef<Path>, mission_id: &str) -> Result<DefineStatus> {
    let connection = open_database(path.as_ref())?;
    load_status(&connection, mission_id)
}

pub(crate) fn define_status_optional(
    path: impl AsRef<Path>,
    mission_id: &str,
) -> Result<Option<DefineStatus>> {
    let connection = open_database(path.as_ref())?;
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM define_workflows WHERE mission_id=?1)",
        [mission_id],
        |row| row.get(0),
    )?;
    if exists {
        load_status(&connection, mission_id).map(Some)
    } else {
        Ok(None)
    }
}

pub fn define_next(path: impl AsRef<Path>, mission_id: &str) -> Result<DefineNext> {
    let connection = open_database(path.as_ref())?;
    let status = load_status(&connection, mission_id)?;
    let current_step = status.current_step.map(|id| DefineStepContract {
        id,
        order: id.order(),
        contract_version: id.contract_version(),
        skill: id.skill().to_owned(),
        skill_execution_verification: "host_reported_only".to_owned(),
        required_fields: id
            .required_fields()
            .iter()
            .map(|s| (*s).to_owned())
            .collect(),
        preconditions: if id.order() == 1 {
            vec![]
        } else {
            vec![format!(
                "{} must be completed",
                DefineStepId::all()[usize::from(id.order() - 2)].as_str()
            )]
        },
        validation_rules: id.validation_rules(),
        limitations: vec![
            "Structural validation does not establish the truth of product or market claims"
                .to_owned(),
            "Skill use is reported by the host and is not independently verified".to_owned(),
        ],
    });
    let accepted_inputs = load_current_artifacts(&connection, mission_id)?;
    let brief_assessment_contract = if status.state == DefineState::Completed {
        Some(build_brief_assessment_contract(&connection, mission_id)?)
    } else {
        None
    };
    let questions = load_questions(&connection, mission_id)?;
    Ok(DefineNext {
        schema: "tramuri.define.step.v3".to_owned(),
        mission_id: mission_id.to_owned(),
        workflow_version: status.workflow_version.clone(),
        state: status.state,
        current_step,
        accepted_inputs,
        brief_assessment_contract,
        questions,
        progress: DefineProgress {
            completed: status.completed,
            total: status.total,
        },
        clarification_budget: DefineClarificationBudget {
            used: status.clarification_count,
            maximum: MAX_QUESTIONS,
        },
        next_action: if status.state == DefineState::Completed {
            "define_complete".to_owned()
        } else if status.state == DefineState::Blocked {
            "resolve_blocking_decision".to_owned()
        } else {
            "produce_and_submit_step_result".to_owned()
        },
    })
}

/// Return every accepted artifact revision, including historical and stale versions.
pub fn define_artifact_history(
    path: impl AsRef<Path>,
    mission_id: &str,
    step: Option<DefineStepId>,
) -> Result<Vec<DefineArtifact>> {
    let connection = open_database(path.as_ref())?;
    load_artifacts(&connection, mission_id).map(|artifacts| {
        artifacts
            .into_iter()
            .filter(|artifact| step.is_none_or(|filter| artifact.step == filter))
            .collect()
    })
}

/// Persist a host-produced semantic assessment of the current Product Brief.
/// The Core validates the rubric structure and references, not the semantic truth of the review.
pub fn define_brief_assess(
    path: impl AsRef<Path>,
    mission_id: &str,
    assessment: Value,
) -> Result<DefineBriefAssessment> {
    let mut connection = open_database(path.as_ref())?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (workflow_id, brief_id, brief_revision): (String, String, i64) = transaction
        .query_row(
            "SELECT w.id,a.id,a.revision FROM define_workflows w
             JOIN define_steps s ON s.workflow_id=w.id AND s.step_id='product_brief' AND s.state='completed'
             JOIN define_artifacts a ON a.id=s.current_artifact_id AND a.accepted=1
             WHERE w.mission_id=?1",
            [mission_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| TramuriError::InvalidDefine(
            "semantic assessment requires a current, completed Product Brief".to_owned(),
        ))?;

    let mut parsed: DefineBriefAssessment =
        serde_json::from_value(assessment.clone()).map_err(|error| {
            TramuriError::InvalidDefine(format!("invalid brief assessment: {error}"))
        })?;
    if parsed.schema != "tramuri.product-brief-assessment.v1"
        || parsed.rubric_version != DEFINE_BRIEF_RUBRIC_VERSION
        || parsed.brief_artifact_id != brief_id
        || i64::from(parsed.brief_revision) != brief_revision
        || parsed.evaluator.trim().is_empty()
        || parsed.evaluator.chars().count() > 120
    {
        return Err(TramuriError::InvalidDefine(
            "assessment schema, rubric, evaluator, or current Brief reference is invalid"
                .to_owned(),
        ));
    }
    if !BRIEF_ASSESSMENT_OVERALL_RESULTS.contains(&parsed.overall.as_str()) {
        return Err(TramuriError::InvalidDefine(
            "overall must be one of: pass, concerns, incomplete".to_owned(),
        ));
    }
    let expected_criteria = BRIEF_ASSESSMENT_CRITERIA
        .iter()
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    if parsed.criteria.len() != expected_criteria.len() {
        return Err(TramuriError::InvalidDefine(format!(
            "criteria must contain exactly {} entries, one for each v1 rubric ID: {}",
            expected_criteria.len(),
            expected_criteria.join(", ")
        )));
    }
    if parsed.limitations.len() > BRIEF_ASSESSMENT_MAX_LIMITATIONS {
        return Err(TramuriError::InvalidDefine(format!(
            "limitations accepts at most {} entries",
            BRIEF_ASSESSMENT_MAX_LIMITATIONS
        )));
    }
    if parsed.limitations.iter().any(|item| {
        item.trim().is_empty() || item.chars().count() > BRIEF_ASSESSMENT_MAX_LIMITATION_CHARACTERS
    }) {
        return Err(TramuriError::InvalidDefine(format!(
            "each limitation must be non-empty and at most {} characters",
            BRIEF_ASSESSMENT_MAX_LIMITATION_CHARACTERS
        )));
    }
    let names: std::collections::HashSet<&str> = parsed
        .criteria
        .iter()
        .map(|item| item.criterion.as_str())
        .collect();
    if names.len() != expected_criteria.len()
        || expected_criteria.iter().any(|name| !names.contains(name))
    {
        return Err(TramuriError::InvalidDefine(format!(
            "criteria must use each v1 rubric ID exactly once: {}",
            expected_criteria.join(", ")
        )));
    }
    let mut has_concern = false;
    let mut has_not_assessed = false;
    for criterion in &parsed.criteria {
        if !BRIEF_ASSESSMENT_RESULTS.contains(&criterion.result.as_str())
            || criterion.rationale.trim().is_empty()
            || criterion.rationale.chars().count() > BRIEF_ASSESSMENT_MAX_RATIONALE_CHARACTERS
        {
            return Err(TramuriError::InvalidDefine(format!(
                "criterion {} result must be pass, concern, or not_assessed; rationale must be non-empty and at most {} characters",
                criterion.criterion, BRIEF_ASSESSMENT_MAX_RATIONALE_CHARACTERS
            )));
        }
        has_concern |= criterion.result == "concern";
        has_not_assessed |= criterion.result == "not_assessed";
        for reference in &criterion.references {
            let referenced: Option<String> = transaction
                .query_row(
                    "SELECT payload_json FROM define_artifacts WHERE workflow_id=?1 AND step_id=?2 AND revision=?3 AND accepted=1",
                    params![workflow_id, reference.step.as_str(), reference.revision],
                    |row| row.get(0),
                )
                .optional()?;
            let Some(payload) = referenced else {
                return Err(TramuriError::InvalidDefine(format!(
                    "assessment reference {} revision {} does not exist",
                    reference.step.as_str(),
                    reference.revision
                )));
            };
            if reference.field.trim().is_empty()
                || !json_field_path_exists(&payload, &reference.field)?
            {
                return Err(TramuriError::InvalidDefine(format!(
                    "assessment reference field {} is not present in {} revision {}",
                    reference.field,
                    reference.step.as_str(),
                    reference.revision
                )));
            }
            if reference.step == DefineStepId::ProductBrief
                && reference.revision != parsed.brief_revision
            {
                return Err(TramuriError::InvalidDefine(
                    "assessment must not cite an obsolete Product Brief revision".to_owned(),
                ));
            }
        }
        if criterion.result == "concern" && criterion.references.is_empty() {
            return Err(TramuriError::InvalidDefine(format!(
                "concern {} must include at least one artifact reference",
                criterion.criterion
            )));
        }
        if criterion.result == "pass" && criterion.references.is_empty() {
            return Err(TramuriError::InvalidDefine(format!(
                "pass {} must cite at least one artifact field",
                criterion.criterion
            )));
        }
    }
    if (has_concern || has_not_assessed) && parsed.overall == "pass" {
        return Err(TramuriError::InvalidDefine(
            "overall cannot be pass when any criterion is concern or not_assessed".to_owned(),
        ));
    }
    if parsed.evaluation_boundary != BRIEF_ASSESSMENT_EVALUATION_BOUNDARY {
        return Err(TramuriError::InvalidDefine(format!(
            "evaluation_boundary must be `{BRIEF_ASSESSMENT_EVALUATION_BOUNDARY}`; this report is not Core verification"
        )));
    }

    let now = now_ms()?;
    parsed.mission_id = mission_id.to_owned();
    parsed.occurred_at_unix_ms = now;
    let payload = serde_json::to_value(&parsed)
        .map_err(|error| TramuriError::InvalidDefine(error.to_string()))?;
    insert_event(
        &transaction,
        &workflow_id,
        Some(DefineStepId::ProductBrief),
        "step_submitted",
        Some(&brief_id),
        json!({"operation":"brief_semantic_assessment","assessment":payload}),
        now,
    )?;
    transaction.commit()?;
    Ok(parsed)
}

/// Return the exact rubric and a non-submittable draft for the current accepted Product Brief.
pub fn define_brief_assessment_contract(
    path: impl AsRef<Path>,
    mission_id: &str,
) -> Result<DefineBriefAssessmentContract> {
    let connection = open_database(path.as_ref())?;
    build_brief_assessment_contract(&connection, mission_id)
}

fn build_brief_assessment_contract(
    connection: &Connection,
    mission_id: &str,
) -> Result<DefineBriefAssessmentContract> {
    let (brief_artifact_id, brief_revision): (String, i64) = connection
        .query_row(
            "SELECT a.id,a.revision FROM define_workflows w
             JOIN define_steps s ON s.workflow_id=w.id AND s.step_id='product_brief' AND s.state='completed'
             JOIN define_artifacts a ON a.id=s.current_artifact_id AND a.accepted=1
             WHERE w.mission_id=?1",
            [mission_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| {
            TramuriError::InvalidDefine(
                "assessment contract requires a current, completed Product Brief".to_owned(),
            )
        })?;
    let brief_revision = u32::try_from(brief_revision).map_err(|_| {
        TramuriError::InvalidDefine("current Product Brief revision is invalid".to_owned())
    })?;
    let criteria = BRIEF_ASSESSMENT_CRITERIA
        .iter()
        .map(|(id, guidance)| DefineBriefAssessmentCriterionContract {
            id: (*id).to_owned(),
            guidance: (*guidance).to_owned(),
        })
        .collect::<Vec<_>>();
    let assessment_template = json!({
        "schema": "tramuri.product-brief-assessment.v1",
        "mission_id": mission_id,
        "brief_artifact_id": brief_artifact_id,
        "brief_revision": brief_revision,
        "evaluator": "<declared-host-or-reviewer; overwritten by --evaluator>",
        "rubric_version": DEFINE_BRIEF_RUBRIC_VERSION,
        "overall": "<pass|concerns|incomplete>",
        "criteria": BRIEF_ASSESSMENT_CRITERIA.iter().map(|(criterion, _)| json!({
            "criterion": criterion,
            "result": "<pass|concern|not_assessed>",
            "rationale": "<non-empty rationale, at most 1000 characters>",
            "references": [{"step":"product_brief","revision":brief_revision,"field":"<accepted JSON field path>"}]
        })).collect::<Vec<_>>(),
        "limitations": [],
        "evaluation_boundary": BRIEF_ASSESSMENT_EVALUATION_BOUNDARY
    });
    Ok(DefineBriefAssessmentContract {
        contract_schema: "tramuri.product-brief-assessment-contract.v1".to_owned(),
        assessment_schema: "tramuri.product-brief-assessment.v1".to_owned(),
        mission_id: mission_id.to_owned(),
        brief_artifact_id,
        brief_revision,
        rubric_version: DEFINE_BRIEF_RUBRIC_VERSION.to_owned(),
        criteria,
        criterion_results: BRIEF_ASSESSMENT_RESULTS
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
        overall_results: BRIEF_ASSESSMENT_OVERALL_RESULTS
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
        reference_shape: json!({
            "step": "<accepted DEFINE step>",
            "revision": "<accepted revision number>",
            "field": "<existing JSON field path>"
        }),
        maximum_limitations: BRIEF_ASSESSMENT_MAX_LIMITATIONS,
        maximum_limitation_characters: BRIEF_ASSESSMENT_MAX_LIMITATION_CHARACTERS,
        maximum_rationale_characters: BRIEF_ASSESSMENT_MAX_RATIONALE_CHARACTERS,
        evaluation_boundary: BRIEF_ASSESSMENT_EVALUATION_BOUNDARY.to_owned(),
        assessment_template,
    })
}

/// Read the newest host-produced semantic assessment without treating it as Core verification.
pub fn define_brief_assessment_get(
    path: impl AsRef<Path>,
    mission_id: &str,
) -> Result<Option<DefineBriefAssessment>> {
    let connection = open_database(path.as_ref())?;
    let payload: Option<String> = connection
        .query_row(
            "SELECT e.payload_json FROM define_events e JOIN define_workflows w ON w.id=e.workflow_id
             WHERE w.mission_id=?1 AND e.event_type='step_submitted'
               AND json_extract(e.payload_json,'$.operation')='brief_semantic_assessment'
             ORDER BY e.occurred_at_unix_ms DESC,e.id DESC LIMIT 1",
            [mission_id],
            |row| row.get(0),
        )
        .optional()?;
    payload
        .map(|value| {
            let event: Value = serde_json::from_str(&value).map_err(|error| {
                TramuriError::InvalidDefine(format!(
                    "invalid stored brief assessment event: {error}"
                ))
            })?;
            serde_json::from_value(event.get("assessment").cloned().ok_or_else(|| {
                TramuriError::InvalidDefine(
                    "stored brief assessment event is missing assessment".to_owned(),
                )
            })?)
            .map_err(|error| {
                TramuriError::InvalidDefine(format!("invalid stored brief assessment: {error}"))
            })
        })
        .transpose()
}

fn json_field_path_exists(payload: &str, path: &str) -> Result<bool> {
    let value: Value = serde_json::from_str(payload).map_err(|error| {
        TramuriError::InvalidDefine(format!("invalid referenced artifact JSON: {error}"))
    })?;
    Ok(path
        .split('.')
        .try_fold(&value, |current, part| current.get(part))
        .is_some())
}

pub fn define_submit(
    path: impl AsRef<Path>,
    mission_id: &str,
    step: DefineStepId,
    payload: Value,
    expected_revision: Option<u64>,
    host_reported_skill_id: Option<&str>,
) -> Result<DefineSubmitResult> {
    let mut connection = open_database(path.as_ref())?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (workflow_id, current, state, revision): (String, Option<String>, String, i64) = transaction.query_row(
        "SELECT id,current_step_id,state,revision FROM define_workflows WHERE mission_id=?1", [mission_id],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))
    ).optional()?.ok_or_else(|| TramuriError::DefineWorkflowNotFound(mission_id.to_owned()))?;
    if state == "completed" {
        if step != DefineStepId::ProductBrief {
            return Err(TramuriError::InvalidDefine(
                "DEFINE is already complete".to_owned(),
            ));
        }
        let accepted:Option<(String,String)>=transaction.query_row(
            "SELECT id,payload_json FROM define_artifacts WHERE workflow_id=?1 AND step_id=?2 AND accepted=1 ORDER BY revision DESC LIMIT 1",
            params![workflow_id,step.as_str()],|row|Ok((row.get(0)?,row.get(1)?))
        ).optional()?;
        if let Some((artifact_id, stored)) = accepted {
            let stored: Value = serde_json::from_str(&stored).map_err(|error| {
                TramuriError::InvalidDefine(format!("invalid stored Product Brief JSON: {error}"))
            })?;
            if stored == payload {
                transaction.commit()?;
                let status = load_status(&connection, mission_id)?;
                return Ok(submit_result(
                    mission_id,
                    step,
                    (true, Some(artifact_id), vec![]),
                    status,
                    false,
                    "define_complete",
                ));
            }
        }
        return Err(TramuriError::InvalidDefine("DEFINE is already complete with a different Product Brief; start a related Mission to revise it".to_owned()));
    }
    if expected_revision.is_some_and(|v| i64::try_from(v).ok() != Some(revision)) {
        return Err(TramuriError::DefineRevisionConflict);
    }
    if current.as_deref() != Some(step.as_str()) {
        return Err(TramuriError::InvalidDefine(format!(
            "step {} is not the currently authorized step (current: {})",
            step.as_str(),
            current.as_deref().unwrap_or("none")
        )));
    }
    if state == "blocked" {
        return Err(TramuriError::InvalidDefine(
            "a blocking decision must be answered before submitting this step".to_owned(),
        ));
    }
    let mut errors = validate_payload(step, &payload);
    errors.extend(validate_references(
        &transaction,
        &workflow_id,
        step,
        &payload,
    )?);
    errors.extend(validate_latest_references(
        &transaction,
        &workflow_id,
        step,
        &payload,
    )?);
    let now = now_ms()?;
    if !errors.is_empty() {
        let err_json = serde_json::to_string(&errors)
            .map_err(|e| TramuriError::InvalidDefine(e.to_string()))?;
        transaction.execute("UPDATE define_steps SET state='validation_failed',attempt_count=attempt_count+1,validation_json=?3,updated_at_unix_ms=?4 WHERE workflow_id=?1 AND step_id=?2", params![workflow_id, step.as_str(), err_json, now])?;
        transaction.execute("UPDATE define_workflows SET state='validation_failed',revision=revision+1,updated_at_unix_ms=?2 WHERE id=?1", params![workflow_id,now])?;
        insert_event(
            &transaction,
            &workflow_id,
            Some(step),
            "step_validation_failed",
            None,
            json!({"errors": errors}),
            now,
        )?;
        transaction.commit()?;
        let status = load_status(&connection, mission_id)?;
        return Ok(submit_result(
            mission_id,
            step,
            (false, None, errors),
            status,
            true,
            "revise_current_step",
        ));
    }
    let previous_revision: i64 = transaction.query_row("SELECT COALESCE(MAX(revision),0) FROM define_artifacts WHERE workflow_id=?1 AND step_id=?2 AND accepted=1",params![workflow_id,step.as_str()],|r|r.get(0))?;
    let artifact_id = Uuid::new_v4().to_string();
    let artifact_revision = previous_revision + 1;
    let provenance = payload.get("provenance").cloned().unwrap_or_else(
        || json!({"submission":"host_structured_result","skill_execution":"host_reported_only"}),
    );
    let dependencies = payload
        .get("references")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let payload_json =
        serde_json::to_string(&payload).map_err(|e| TramuriError::InvalidDefine(e.to_string()))?;
    transaction.execute("INSERT INTO define_artifacts (id,workflow_id,step_id,revision,contract_version,payload_json,provenance_json,dependencies_json,validation_json,expected_skill_id,host_reported_skill_id,accepted,created_at_unix_ms) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'{}',?9,?10,1,?11)",params![artifact_id,workflow_id,step.as_str(),artifact_revision,step.contract_version(),payload_json,provenance.to_string(),dependencies.to_string(),step.skill(),host_reported_skill_id,now])?;
    let next = DefineStepId::all().get(usize::from(step.order())).copied();
    transaction.execute("UPDATE define_steps SET state='completed',attempt_count=attempt_count+1,current_artifact_id=?3,validation_json='[]',updated_at_unix_ms=?4,completed_at_unix_ms=?4 WHERE workflow_id=?1 AND step_id=?2",params![workflow_id,step.as_str(),artifact_id,now])?;
    if let Some(next_step) = next {
        transaction.execute("UPDATE define_steps SET state='in_progress',started_at_unix_ms=COALESCE(started_at_unix_ms,?3),updated_at_unix_ms=?3 WHERE workflow_id=?1 AND step_id=?2",params![workflow_id,next_step.as_str(),now])?;
        transaction.execute("UPDATE define_workflows SET current_step_id=?2,state='in_progress',revision=revision+1,updated_at_unix_ms=?3 WHERE id=?1",params![workflow_id,next_step.as_str(),now])?;
        insert_event(
            &transaction,
            &workflow_id,
            Some(next_step),
            "step_started",
            None,
            json!({}),
            now,
        )?;
    } else {
        transaction.execute("UPDATE define_workflows SET current_step_id=NULL,state='completed',revision=revision+1,updated_at_unix_ms=?2,completed_at_unix_ms=?2 WHERE id=?1",params![workflow_id,now])?;
        insert_event(
            &transaction,
            &workflow_id,
            Some(step),
            "define_completed",
            Some(&artifact_id),
            json!({"workflow_version":DEFINE_WORKFLOW_VERSION}),
            now,
        )?;
    }
    insert_event(
        &transaction,
        &workflow_id,
        Some(step),
        "step_submitted",
        Some(&artifact_id),
        json!({"artifact_revision":artifact_revision}),
        now,
    )?;
    insert_event(
        &transaction,
        &workflow_id,
        Some(step),
        "step_completed",
        Some(&artifact_id),
        json!({}),
        now,
    )?;
    transaction.commit()?;
    let status = load_status(&connection, mission_id)?;
    Ok(submit_result(
        mission_id,
        step,
        (true, Some(artifact_id), vec![]),
        status,
        false,
        if next.is_some() {
            "continue_to_next_step"
        } else {
            "define_complete"
        },
    ))
}

/// Revise a previously accepted DEFINE artifact without deleting its prior version.
///
/// The declared author and origin are provenance supplied by the caller; they are not
/// authenticated identities. Downstream accepted artifacts that reference the replaced
/// revision are marked stale in the same transaction.
#[allow(clippy::too_many_arguments)]
pub fn define_review(
    path: impl AsRef<Path>,
    mission_id: &str,
    step: DefineStepId,
    payload: Value,
    reason: &str,
    declared_author: &str,
    origin: &str,
    expected_revision: Option<u64>,
    authorize_decision_changes: bool,
    host_reported_skill_id: Option<&str>,
) -> Result<DefineSubmitResult> {
    if reason.trim().is_empty() || declared_author.trim().is_empty() || origin.trim().is_empty() {
        return Err(TramuriError::InvalidDefine(
            "review requires a non-empty reason, declared author, and origin".to_owned(),
        ));
    }

    let mut connection = open_database(path.as_ref())?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (workflow_id, state, current, workflow_revision):
        (String, String, Option<String>, i64) = transaction.query_row(
        "SELECT id,state,current_step_id,revision FROM define_workflows WHERE mission_id=?1",
        [mission_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).optional()?.ok_or_else(|| TramuriError::DefineWorkflowNotFound(mission_id.to_owned()))?;
    if expected_revision.is_some_and(|value| i64::try_from(value).ok() != Some(workflow_revision)) {
        return Err(TramuriError::DefineRevisionConflict);
    }
    if state == "blocked" {
        return Err(TramuriError::InvalidDefine(
            "a blocking decision must be answered before reviewing this workflow".to_owned(),
        ));
    }

    let previous: Option<(String, i64, String, String)> = transaction.query_row(
        "SELECT a.id,a.revision,a.payload_json,a.provenance_json
         FROM define_steps s JOIN define_artifacts a ON a.id=s.current_artifact_id
         WHERE s.workflow_id=?1 AND s.step_id=?2 AND s.state IN ('completed','stale') AND a.accepted=1",
        params![workflow_id, step.as_str()],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).optional()?;
    let Some((previous_id, previous_revision, previous_payload_json, _)) = previous else {
        return Err(TramuriError::InvalidDefine(format!(
            "step {} has no accepted artifact to review",
            step.as_str()
        )));
    };
    let previous_payload: Value =
        serde_json::from_str(&previous_payload_json).map_err(|error| {
            TramuriError::InvalidDefine(format!("invalid stored artifact JSON: {error}"))
        })?;
    if step == DefineStepId::ProductBrief
        && previous_payload.get("approved_decisions") != payload.get("approved_decisions")
        && !authorize_decision_changes
    {
        return Err(TramuriError::InvalidDefine(
            "changing approved decisions requires explicit authorization".to_owned(),
        ));
    }

    let mut errors = validate_payload(step, &payload);
    errors.extend(validate_references(
        &transaction,
        &workflow_id,
        step,
        &payload,
    )?);
    errors.extend(validate_latest_references(
        &transaction,
        &workflow_id,
        step,
        &payload,
    )?);
    if !errors.is_empty() {
        transaction.commit()?;
        let status = load_status(&connection, mission_id)?;
        return Ok(submit_result(
            mission_id,
            step,
            (false, None, errors),
            status,
            true,
            "revise_current_artifact",
        ));
    }

    let now = now_ms()?;
    let artifact_id = Uuid::new_v4().to_string();
    let next_artifact_revision = previous_revision + 1;
    let host_provenance = payload
        .get("provenance")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let provenance = json!({
        "submission": "explicit_artifact_review",
        "host_provenance": host_provenance,
        "review": {
            "reason": reason,
            "declared_author": declared_author,
            "origin": origin,
            "occurred_at_unix_ms": now,
            "authorized_approved_decision_changes": authorize_decision_changes
        }
    });
    let dependencies = payload
        .get("references")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let payload_json = serde_json::to_string(&payload)
        .map_err(|error| TramuriError::InvalidDefine(error.to_string()))?;
    transaction.execute(
        "INSERT INTO define_artifacts (id,workflow_id,step_id,revision,contract_version,payload_json,provenance_json,dependencies_json,validation_json,expected_skill_id,host_reported_skill_id,accepted,created_at_unix_ms)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'{}',?9,?10,1,?11)",
        params![artifact_id, workflow_id, step.as_str(), next_artifact_revision, step.contract_version(), payload_json, provenance.to_string(), dependencies.to_string(), step.skill(), host_reported_skill_id, now],
    )?;

    let stale_steps =
        invalidate_review_dependents(&transaction, &workflow_id, step, previous_revision, now)?;
    transaction.execute(
        "UPDATE define_steps SET state='completed',current_artifact_id=?3,validation_json='[]',updated_at_unix_ms=?4,completed_at_unix_ms=?4 WHERE workflow_id=?1 AND step_id=?2",
        params![workflow_id, step.as_str(), artifact_id, now],
    )?;

    let first_stale = stale_steps.first().copied();
    let next_after_review = if first_stale.is_some() {
        first_stale
    } else if step.order() < 5 && current.is_some() {
        current.as_deref().and_then(DefineStepId::from_str)
    } else if step.order() == 5 {
        None
    } else {
        DefineStepId::all().get(usize::from(step.order())).copied()
    };
    let next_state = if first_stale.is_some() {
        "stale"
    } else if next_after_review.is_none() {
        "completed"
    } else {
        "in_progress"
    };
    transaction.execute(
        "UPDATE define_workflows SET current_step_id=?2,state=?3,revision=revision+1,updated_at_unix_ms=?4,completed_at_unix_ms=CASE WHEN ?3='completed' THEN COALESCE(completed_at_unix_ms,?4) ELSE NULL END WHERE id=?1",
        params![workflow_id, next_after_review.map(DefineStepId::as_str), next_state, now],
    )?;
    insert_event(
        &transaction,
        &workflow_id,
        Some(step),
        "step_submitted",
        Some(&artifact_id),
        json!({"operation":"review","reason":reason,"declared_author":declared_author,"origin":origin,"previous_artifact_id":previous_id,"previous_revision":previous_revision,"artifact_revision":next_artifact_revision}),
        now,
    )?;
    for stale_step in stale_steps {
        insert_event(
            &transaction,
            &workflow_id,
            Some(stale_step),
            "step_invalidated",
            None,
            json!({"reason":"dependency_revision_changed","source_step":step.as_str(),"source_artifact_id":previous_id}),
            now,
        )?;
    }
    transaction.commit()?;
    let status = load_status(&connection, mission_id)?;
    Ok(submit_result(
        mission_id,
        step,
        (true, Some(artifact_id), vec![]),
        status,
        false,
        if first_stale.is_some() {
            "revalidate_stale_steps"
        } else if next_after_review.is_some() {
            "continue_workflow"
        } else {
            "define_complete"
        },
    ))
}

fn validate_latest_references(
    transaction: &Transaction<'_>,
    workflow_id: &str,
    step: DefineStepId,
    payload: &Value,
) -> Result<Vec<DefineValidationError>> {
    let mut errors = Vec::new();
    let Some(references) = payload.get("references").and_then(Value::as_array) else {
        return Ok(errors);
    };
    for (index, reference) in references.iter().enumerate() {
        let Some(referenced_step_name) = reference.get("step").and_then(Value::as_str) else {
            continue;
        };
        let Some(referenced_step) = DefineStepId::from_str(referenced_step_name) else {
            continue;
        };
        if referenced_step.order() >= step.order() {
            continue;
        }
        let supplied_revision = reference.get("revision").and_then(Value::as_i64);
        let latest_revision: Option<i64> = transaction.query_row(
            "SELECT MAX(revision) FROM define_artifacts WHERE workflow_id=?1 AND step_id=?2 AND accepted=1",
            params![workflow_id, referenced_step.as_str()],
            |row| row.get(0),
        )?;
        if supplied_revision.is_some() && supplied_revision != latest_revision {
            errors.push(validation_error(
                "stale_reference",
                &format!("references[{index}].revision"),
                "review must reference the latest accepted revision of an earlier step",
            ));
        }
    }
    Ok(errors)
}

fn invalidate_review_dependents(
    transaction: &Transaction<'_>,
    workflow_id: &str,
    revised_step: DefineStepId,
    revised_revision: i64,
    now: i64,
) -> Result<Vec<DefineStepId>> {
    let mut invalidated_revisions = vec![(revised_step, revised_revision)];
    let mut stale = Vec::new();
    for candidate in DefineStepId::all()
        .into_iter()
        .filter(|id| id.order() > revised_step.order())
    {
        let current: Option<(String, i64, String)> = transaction.query_row(
            "SELECT a.id,a.revision,a.dependencies_json FROM define_steps s JOIN define_artifacts a ON a.id=s.current_artifact_id WHERE s.workflow_id=?1 AND s.step_id=?2 AND s.state='completed' AND a.accepted=1",
            params![workflow_id, candidate.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).optional()?;
        let Some((artifact_id, revision, dependencies_json)) = current else {
            continue;
        };
        let dependencies: Value = serde_json::from_str(&dependencies_json).map_err(|error| {
            TramuriError::InvalidDefine(format!("invalid stored dependencies JSON: {error}"))
        })?;
        let depends_on_stale = dependencies.as_array().is_some_and(|references| {
            references.iter().any(|reference| {
                let reference_step = reference
                    .get("step")
                    .and_then(Value::as_str)
                    .and_then(DefineStepId::from_str);
                let reference_revision = reference.get("revision").and_then(Value::as_i64);
                reference_step
                    .zip(reference_revision)
                    .is_some_and(|pair| invalidated_revisions.contains(&pair))
            })
        });
        if depends_on_stale {
            transaction.execute(
                "UPDATE define_steps SET state='stale',updated_at_unix_ms=?3 WHERE workflow_id=?1 AND step_id=?2",
                params![workflow_id, candidate.as_str(), now],
            )?;
            invalidated_revisions.push((candidate, revision));
            stale.push(candidate);
            let _ = artifact_id;
        }
    }
    Ok(stale)
}

pub fn define_question_request(
    path: impl AsRef<Path>,
    mission_id: &str,
    step: DefineStepId,
    rationale: &str,
    alternatives: &[String],
    material_decision: bool,
) -> Result<DefineQuestion> {
    let rationale = rationale.trim();
    if rationale.is_empty() || rationale.len() > 2000 {
        return Err(TramuriError::InvalidDefine(
            "question rationale must contain 1..=2000 characters".to_owned(),
        ));
    }
    if alternatives.len() > 10
        || alternatives.iter().any(|item| {
            item.trim().is_empty() || item.len() > 500 || item.chars().any(char::is_control)
        })
    {
        return Err(TramuriError::InvalidDefine("provide at most 10 non-empty alternatives, each up to 500 characters without control characters".to_owned()));
    }
    let mut connection = open_database(path.as_ref())?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (workflow_id,current,count):(String,Option<String>,u32)=transaction.query_row("SELECT id,current_step_id,clarification_count FROM define_workflows WHERE mission_id=?1",[mission_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?.ok_or_else(||TramuriError::DefineWorkflowNotFound(mission_id.to_owned()))?;
    if current.as_deref() != Some(step.as_str()) {
        return Err(TramuriError::InvalidDefine(
            "questions can only be requested for the currently authorized step".to_owned(),
        ));
    }
    if count >= MAX_QUESTIONS {
        return Err(TramuriError::InvalidDefine("clarification budget exhausted; record a reversible assumption or keep the step blocked".to_owned()));
    }
    let now = now_ms()?;
    let id = Uuid::new_v4().to_string();
    let state = if material_decision {
        "blocked"
    } else {
        "requested"
    };
    let alts = serde_json::to_string(alternatives)
        .map_err(|e| TramuriError::InvalidDefine(e.to_string()))?;
    transaction.execute("INSERT INTO define_questions (id,workflow_id,step_id,rationale,alternatives_json,state,created_at_unix_ms,updated_at_unix_ms) VALUES (?1,?2,?3,?4,?5,?6,?7,?7)",params![id,workflow_id,step.as_str(),rationale,alts,state,now])?;
    transaction.execute("UPDATE define_workflows SET clarification_count=clarification_count+1,state=?2,revision=revision+1,updated_at_unix_ms=?3 WHERE id=?1",params![workflow_id,if material_decision{"blocked"}else{"in_progress"},now])?;
    if material_decision {
        transaction.execute("UPDATE define_steps SET state='blocked',updated_at_unix_ms=?3 WHERE workflow_id=?1 AND step_id=?2",params![workflow_id,step.as_str(),now])?;
    }
    insert_event(
        &transaction,
        &workflow_id,
        Some(step),
        "decision_requested",
        None,
        json!({"question_id":id,"material_decision":material_decision}),
        now,
    )?;
    if material_decision {
        insert_event(
            &transaction,
            &workflow_id,
            Some(step),
            "step_blocked",
            None,
            json!({"question_id":id}),
            now,
        )?;
    }
    transaction.commit()?;
    Ok(DefineQuestion {
        id,
        step,
        rationale: rationale.to_owned(),
        alternatives: alternatives.to_vec(),
        answer: None,
        state: state.to_owned(),
    })
}

pub fn define_question_answer(
    path: impl AsRef<Path>,
    mission_id: &str,
    question_id: &str,
    answer: &str,
    explicit_user_answer: bool,
) -> Result<DefineQuestion> {
    let answer = answer.trim();
    if answer.is_empty() || answer.len() > 4000 {
        return Err(TramuriError::InvalidDefine(
            "answer must contain 1..=4000 characters".to_owned(),
        ));
    }
    let mut connection = open_database(path.as_ref())?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (workflow_id, step, state): (String, String, String) = transaction
        .query_row(
            "SELECT q.workflow_id,q.step_id,q.state FROM define_questions q JOIN define_workflows w ON w.id=q.workflow_id WHERE w.mission_id=?1 AND q.id=?2",
            params![mission_id, question_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| TramuriError::InvalidDefine(format!("question not found: {question_id}")))?;
    if state != "requested" && state != "blocked" {
        return Err(TramuriError::InvalidDefine(
            "question is already answered or deferred".to_owned(),
        ));
    }
    if state == "blocked" && !explicit_user_answer {
        return Err(TramuriError::InvalidDefine(
            "a blocking decision requires an explicitly declared user answer".to_owned(),
        ));
    }
    let now = now_ms()?;
    transaction.execute("UPDATE define_questions SET answer=?2,answer_source=?3,state='answered',updated_at_unix_ms=?4 WHERE id=?1",params![question_id,answer,if explicit_user_answer{"user_explicit"}else{"declared_other"},now])?;
    let blocking_questions_remaining: u32 = transaction.query_row(
        "SELECT COUNT(*) FROM define_questions WHERE workflow_id=?1 AND step_id=?2 AND state='blocked'",
        params![workflow_id, step],
        |row| row.get(0),
    )?;
    if blocking_questions_remaining == 0 {
        transaction.execute("UPDATE define_steps SET state='in_progress',updated_at_unix_ms=?3 WHERE workflow_id=?1 AND step_id=?2 AND state='blocked'",params![workflow_id,step,now])?;
        transaction.execute("UPDATE define_workflows SET state='in_progress',revision=revision+1,updated_at_unix_ms=?2 WHERE id=?1",params![workflow_id,now])?;
    } else {
        transaction.execute(
            "UPDATE define_workflows SET revision=revision+1,updated_at_unix_ms=?2 WHERE id=?1",
            params![workflow_id, now],
        )?;
    }
    insert_event(
        &transaction,
        &workflow_id,
        DefineStepId::from_str(&step),
        "decision_recorded",
        None,
        json!({"question_id":question_id,"answer_source":if explicit_user_answer{"user_explicit"}else{"declared_other"}}),
        now,
    )?;
    let (rationale, alternatives): (String, String) = transaction.query_row(
        "SELECT rationale,alternatives_json FROM define_questions WHERE id=?1",
        [question_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    transaction.commit()?;
    let alternatives = serde_json::from_str(&alternatives)
        .map_err(|e| TramuriError::InvalidDefine(e.to_string()))?;
    Ok(DefineQuestion {
        id: question_id.to_owned(),
        step: DefineStepId::from_str(&step)
            .ok_or_else(|| TramuriError::InvalidDefine("invalid stored step".to_owned()))?,
        rationale,
        alternatives,
        answer: Some(answer.to_owned()),
        state: "answered".to_owned(),
    })
}

fn validate_payload(step: DefineStepId, payload: &Value) -> Vec<DefineValidationError> {
    let mut errors = Vec::new();
    if !payload.is_object() {
        return vec![validation_error(
            "invalid_object",
            "$",
            "result must be a JSON object",
        )];
    }
    for field in step.required_fields() {
        let value = payload.get(field);
        let valid = if step.string_fields().contains(field) {
            value
                .and_then(Value::as_str)
                .is_some_and(|s| !s.trim().is_empty())
        } else {
            // Empty arrays are explicit declarations that no items were identified. The field
            // still has to use its contract type; arbitrary scalars and objects are rejected.
            value.is_some_and(Value::is_array)
        };
        if !valid {
            errors.push(validation_error(
                if value.is_some() {
                    "invalid_field_type"
                } else {
                    "missing_required_field"
                },
                field,
                &format!(
                    "required field `{field}` must be a {}",
                    if step.string_fields().contains(field) {
                        "non-empty string"
                    } else {
                        "JSON array (empty arrays are structurally accepted)"
                    }
                ),
            ));
        }
    }
    if step == DefineStepId::KnowledgeSynthesis {
        if let Some(items) = payload.get("classifications").and_then(Value::as_array) {
            for (i, item) in items.iter().enumerate() {
                let kind = item.get("kind").and_then(Value::as_str).unwrap_or("");
                if ![
                    "user_provided",
                    "observed",
                    "hypothesis",
                    "recommendation",
                    "approved_decision",
                    "open_question",
                    "risk",
                ]
                .contains(&kind)
                {
                    errors.push(validation_error(
                        "invalid_classification",
                        &format!("classifications[{i}].kind"),
                        "classification kind must be one of: user_provided, observed, hypothesis, recommendation, approved_decision, open_question, risk",
                    ));
                }
                if item
                    .get("statement")
                    .and_then(Value::as_str)
                    .is_none_or(|s| s.trim().is_empty())
                {
                    errors.push(validation_error(
                        "missing_statement",
                        &format!("classifications[{i}].statement"),
                        "statement is required",
                    ));
                }
            }
        }
    }
    errors
}

fn validate_references(
    transaction: &Transaction<'_>,
    workflow_id: &str,
    step: DefineStepId,
    payload: &Value,
) -> Result<Vec<DefineValidationError>> {
    let mut errors = Vec::new();
    let references = match payload.get("references").and_then(Value::as_array) {
        Some(references) => references,
        None if step.order() == 1 => return Ok(errors),
        None => {
            return Ok(vec![validation_error(
                "missing_previous_step_reference",
                "references",
                "include the accepted artifact from the immediately preceding step",
            )]);
        }
    };
    let mut previous_step_referenced = false;
    for (index, reference) in references.iter().enumerate() {
        let field = format!("references[{index}]");
        let (Some(step_name), Some(revision)) = (
            reference.get("step").and_then(Value::as_str),
            reference.get("revision").and_then(Value::as_i64),
        ) else {
            errors.push(validation_error(
                "invalid_reference",
                &field,
                "reference must include a prior step and revision",
            ));
            continue;
        };
        let Some(referenced_step) = DefineStepId::from_str(step_name) else {
            errors.push(validation_error(
                "unknown_reference_step",
                &format!("{field}.step"),
                "reference points to an unknown DEFINE step",
            ));
            continue;
        };
        if referenced_step.order() >= step.order() {
            errors.push(validation_error(
                "forward_reference",
                &format!("{field}.step"),
                "a step may reference only an earlier accepted step",
            ));
            continue;
        }
        let exists:bool=transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM define_artifacts WHERE workflow_id=?1 AND step_id=?2 AND revision=?3 AND accepted=1)",
            params![workflow_id,referenced_step.as_str(),revision],|row|row.get(0)
        )?;
        if !exists {
            errors.push(validation_error(
                "unresolved_reference",
                &field,
                "reference does not identify an accepted artifact revision",
            ));
        }
        if referenced_step.order() + 1 == step.order() && exists {
            previous_step_referenced = true;
        }
    }
    if step.order() > 1 && !previous_step_referenced {
        errors.push(validation_error(
            "missing_previous_step_reference",
            "references",
            "include the accepted artifact from the immediately preceding step",
        ));
    }
    Ok(errors)
}

fn load_status(connection: &rusqlite::Connection, mission_id: &str) -> Result<DefineStatus> {
    let (workflow_id,workflow_version,state,current,clarifications,revision,started,updated,completed):(String,String,String,Option<String>,u32,i64,i64,i64,Option<i64>)=connection.query_row("SELECT id,workflow_version,state,current_step_id,clarification_count,revision,started_at_unix_ms,updated_at_unix_ms,completed_at_unix_ms FROM define_workflows WHERE mission_id=?1",[mission_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?))).optional()?.ok_or_else(||TramuriError::DefineWorkflowNotFound(mission_id.to_owned()))?;
    let mut stmt=connection.prepare("SELECT step_id,state,contract_version,attempt_count,current_artifact_id,validation_json FROM define_steps WHERE workflow_id=?1 ORDER BY position")?;
    let rows = stmt.query_map([&workflow_id], |r| {
        let id: String = r.get(0)?;
        let state: String = r.get(1)?;
        let errors: String = r.get(5)?;
        Ok((
            id,
            state,
            r.get::<_, String>(2)?,
            r.get::<_, u32>(3)?,
            r.get::<_, Option<String>>(4)?,
            errors,
        ))
    })?;
    let mut steps = Vec::new();
    for row in rows {
        let (id, s, contract_version, attempts, artifact_id, errors) = row?;
        steps.push(DefineStep {
            id: DefineStepId::from_str(&id)
                .ok_or_else(|| TramuriError::InvalidDefine(format!("unknown stored step {id}")))?,
            order: DefineStepId::from_str(&id).map_or(0, DefineStepId::order),
            state: DefineStepState::from_str(&s)
                .ok_or_else(|| TramuriError::InvalidDefine(format!("unknown stored state {s}")))?,
            contract_version,
            attempts,
            artifact_id,
            validation_errors: serde_json::from_str(&errors).map_err(|e| {
                TramuriError::InvalidDefine(format!("invalid stored validation JSON: {e}"))
            })?,
        });
    }
    let completed_count = steps
        .iter()
        .filter(|s| s.state == DefineStepState::Completed)
        .count() as u8;
    let state = match state.as_str() {
        "in_progress" => DefineState::InProgress,
        "blocked" => DefineState::Blocked,
        "validation_failed" => DefineState::ValidationFailed,
        "stale" => DefineState::Stale,
        "completed" => DefineState::Completed,
        _ => {
            return Err(TramuriError::InvalidDefine(format!(
                "unknown stored workflow state {state}"
            )));
        }
    };
    let mut event_stmt=connection.prepare("SELECT id,event_type,step_id,occurred_at_unix_ms FROM define_events WHERE workflow_id=?1 ORDER BY occurred_at_unix_ms,id")?;
    let event_rows = event_stmt.query_map([&workflow_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, i64>(3)?,
        ))
    })?;
    let mut events = Vec::new();
    for event in event_rows {
        let (id, event_type, step, occurred_at_unix_ms) = event?;
        events.push(DefineEvent {
            id,
            event_type,
            step: step
                .map(|value| {
                    DefineStepId::from_str(&value).ok_or_else(|| {
                        TramuriError::InvalidDefine(format!("unknown event step {value}"))
                    })
                })
                .transpose()?,
            occurred_at_unix_ms,
        });
    }
    Ok(DefineStatus {
        schema: "tramuri.define.status.v1".to_owned(),
        mission_id: mission_id.to_owned(),
        workflow_version,
        state,
        current_step: current
            .map(|v| {
                DefineStepId::from_str(&v)
                    .ok_or_else(|| TramuriError::InvalidDefine(format!("unknown current step {v}")))
            })
            .transpose()?,
        steps,
        events,
        completed: completed_count,
        total: 5,
        clarification_count: clarifications,
        clarification_maximum: MAX_QUESTIONS,
        revision: u64::try_from(revision)
            .map_err(|_| TramuriError::InvalidDefine("negative workflow revision".to_owned()))?,
        started_at_unix_ms: started,
        updated_at_unix_ms: updated,
        completed_at_unix_ms: completed,
    })
}

fn load_artifacts(
    connection: &rusqlite::Connection,
    mission_id: &str,
) -> Result<Vec<DefineArtifact>> {
    let mut stmt=connection.prepare("SELECT a.id,a.step_id,a.revision,a.contract_version,a.payload_json,a.provenance_json,a.dependencies_json,a.expected_skill_id,a.host_reported_skill_id,a.created_at_unix_ms FROM define_artifacts a JOIN define_workflows w ON w.id=a.workflow_id WHERE w.mission_id=?1 AND a.accepted=1 ORDER BY a.step_id,a.revision")?;
    let rows = stmt.query_map([mission_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, u32>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, String>(7)?,
            r.get::<_, Option<String>>(8)?,
            r.get::<_, i64>(9)?,
        ))
    })?;
    rows.map(|row| {
        let (
            id,
            step,
            revision,
            contract,
            payload,
            provenance,
            dependencies,
            expected,
            host,
            created,
        ) = row?;
        Ok(DefineArtifact {
            id,
            step: DefineStepId::from_str(&step).ok_or_else(|| {
                TramuriError::InvalidDefine(format!("unknown stored artifact step {step}"))
            })?,
            revision,
            contract_version: contract,
            payload: serde_json::from_str(&payload)
                .map_err(|e| TramuriError::InvalidDefine(e.to_string()))?,
            provenance: serde_json::from_str(&provenance)
                .map_err(|e| TramuriError::InvalidDefine(e.to_string()))?,
            dependencies: serde_json::from_str(&dependencies)
                .map_err(|e| TramuriError::InvalidDefine(e.to_string()))?,
            expected_skill_id: expected,
            host_reported_skill_id: host,
            created_at_unix_ms: created,
        })
    })
    .collect()
}

fn load_current_artifacts(
    connection: &rusqlite::Connection,
    mission_id: &str,
) -> Result<Vec<DefineArtifact>> {
    let current_ids: std::collections::HashSet<String> = load_status(connection, mission_id)?
        .steps
        .into_iter()
        .filter(|step| step.state == DefineStepState::Completed)
        .filter_map(|step| step.artifact_id)
        .collect();
    Ok(load_artifacts(connection, mission_id)?
        .into_iter()
        .filter(|artifact| current_ids.contains(&artifact.id))
        .collect())
}

fn load_questions(
    connection: &rusqlite::Connection,
    mission_id: &str,
) -> Result<Vec<DefineQuestion>> {
    let mut stmt=connection.prepare("SELECT q.id,q.step_id,q.rationale,q.alternatives_json,q.answer,q.state FROM define_questions q JOIN define_workflows w ON w.id=q.workflow_id WHERE w.mission_id=?1 ORDER BY q.created_at_unix_ms,q.id")?;
    let rows = stmt.query_map([mission_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;
    rows.map(|row| {
        let (id, step, rationale, alternatives, answer, state) = row?;
        Ok(DefineQuestion {
            id,
            step: DefineStepId::from_str(&step).ok_or_else(|| {
                TramuriError::InvalidDefine(format!("unknown question step {step}"))
            })?,
            rationale,
            alternatives: serde_json::from_str(&alternatives).map_err(|error| {
                TramuriError::InvalidDefine(format!("invalid stored alternatives: {error}"))
            })?,
            answer,
            state,
        })
    })
    .collect()
}

fn submit_result(
    mission: &str,
    step: DefineStepId,
    result: (bool, Option<String>, Vec<DefineValidationError>),
    status: DefineStatus,
    retry: bool,
    next_action: &str,
) -> DefineSubmitResult {
    let (accepted, artifact, errors) = result;
    DefineSubmitResult {
        schema: "tramuri.define.submit.v1".to_owned(),
        accepted,
        mission_id: mission.to_owned(),
        step,
        step_state: status
            .steps
            .iter()
            .find(|s| s.id == step)
            .map_or(DefineStepState::Pending, |s| s.state),
        accepted_artifact_id: artifact,
        progress: DefineProgress {
            completed: status.completed,
            total: status.total,
        },
        next_step: status.current_step,
        workflow_state: status.state,
        errors,
        retry_allowed: retry,
        next_action: next_action.to_owned(),
    }
}
fn validation_error(code: &str, field: &str, message: &str) -> DefineValidationError {
    DefineValidationError {
        code: code.to_owned(),
        field: field.to_owned(),
        message: message.to_owned(),
    }
}
fn insert_event(
    tx: &Transaction<'_>,
    workflow: &str,
    step: Option<DefineStepId>,
    kind: &str,
    artifact: Option<&str>,
    payload: Value,
    now: i64,
) -> Result<()> {
    tx.execute("INSERT INTO define_events(id,workflow_id,step_id,event_type,artifact_id,payload_json,occurred_at_unix_ms) VALUES (?1,?2,?3,?4,?5,?6,?7)",params![Uuid::new_v4().to_string(),workflow,step.map(DefineStepId::as_str),kind,artifact,payload.to_string(),now])?;
    Ok(())
}
fn now_ms() -> Result<i64> {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TramuriError::Clock)?
            .as_millis(),
    )
    .map_err(|_| TramuriError::Clock)
}
