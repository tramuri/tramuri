use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::knowledge::{SourceMetadata, normalize_source_path, safe_source_metadata};
use crate::storage::{latest_scan, load_inventory_file, open_database};
use crate::{HashStatus, Result, TramuriError};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionState {
    Open,
    Closed,
}

impl MissionState {
    fn as_storage_value(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }

    fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "open" => Some(Self::Open),
            "closed" => Some(Self::Closed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Criterion {
    pub id: String,
    pub description: String,
    pub required: bool,
    pub evidence_expectation: Option<String>,
    pub position: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionInput {
    pub description: String,
    pub required: bool,
    pub evidence_expectation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuccessContract {
    pub id: String,
    pub mission_id: String,
    pub revision: u32,
    pub created_at_unix_ms: i64,
    pub criteria: Vec<Criterion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mission {
    pub id: String,
    pub objective: String,
    pub context: Option<String>,
    pub state: MissionState,
    pub created_at_unix_ms: i64,
    pub success_contract: SuccessContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionInput {
    pub objective: String,
    pub context: Option<String>,
    pub criteria: Vec<CriterionInput>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    Human,
    Agent,
    Tool,
}

impl ActorKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Agent => "agent",
            Self::Tool => "tool",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionState {
    InProgress,
    Completed,
    Blocked,
    Failed,
    Cancelled,
}

impl ExecutionState {
    fn as_str(self) -> &'static str {
        match self {
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Execution {
    pub id: String,
    pub mission_id: String,
    pub contract_id: String,
    pub contract_revision: u32,
    pub actor_kind: ActorKind,
    pub actor_reference: Option<String>,
    pub state: ExecutionState,
    pub started_at_unix_ms: i64,
    pub ended_at_unix_ms: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionInput {
    pub mission_id: String,
    pub contract_revision: u32,
    pub actor_kind: ActorKind,
    pub actor_reference: Option<String>,
    pub state: ExecutionState,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    ProjectFile,
    ExternalReference,
    HumanAttestation,
}

impl EvidenceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::ProjectFile => "project_file",
            Self::ExternalReference => "external_reference",
            Self::HumanAttestation => "human_attestation",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceReference {
    pub id: String,
    pub mission_id: String,
    pub execution_id: String,
    pub kind: EvidenceKind,
    pub locator: String,
    pub sha256: Option<String>,
    pub hash_status: Option<HashStatus>,
    pub availability: String,
    pub scan_id: Option<i64>,
    pub captured_at_unix_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceInput {
    pub mission_id: String,
    pub execution_id: String,
    pub kind: EvidenceKind,
    pub locator: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CriterionAssessmentResult {
    Met,
    NotMet,
    Unverified,
}

impl CriterionAssessmentResult {
    fn as_str(self) -> &'static str {
        match self {
            Self::Met => "met",
            Self::NotMet => "not_met",
            Self::Unverified => "unverified",
        }
    }

    fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "met" => Some(Self::Met),
            "not_met" => Some(Self::NotMet),
            "unverified" => Some(Self::Unverified),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerifierKind {
    Human,
    Mechanism,
}

impl VerifierKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Mechanism => "mechanism",
        }
    }

    fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "human" => Some(Self::Human),
            "mechanism" => Some(Self::Mechanism),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionAssessmentInput {
    pub mission_id: String,
    pub contract_revision: u32,
    pub criterion_id: String,
    pub result: CriterionAssessmentResult,
    pub evidence_ids: Vec<String>,
    pub verifier_kind: VerifierKind,
    pub verifier_reference: String,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CriterionAssessment {
    pub id: String,
    pub criterion_id: String,
    pub contract_revision: u32,
    pub result: CriterionAssessmentResult,
    pub evidence_ids: Vec<String>,
    pub verifier_kind: VerifierKind,
    pub verifier_reference: String,
    pub assessed_at_unix_ms: i64,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeState {
    Pending,
    NotAchieved,
    Verified,
}

impl std::fmt::Display for OutcomeState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Pending => "pending",
            Self::NotAchieved => "not_achieved",
            Self::Verified => "verified",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CriterionOutcome {
    pub criterion: Criterion,
    pub assessment_history: Vec<CriterionAssessment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifiedOutcome {
    pub mission_id: String,
    pub contract_id: String,
    pub contract_revision: u32,
    pub state: OutcomeState,
    pub criteria: Vec<CriterionOutcome>,
}

pub fn execution_record(path: impl AsRef<Path>, input: ExecutionInput) -> Result<Execution> {
    let workspace = canonical_workspace(path.as_ref())?;
    let connection = open_database(&workspace)?;
    let contract: Option<(String, i64)> = connection
        .query_row(
            "SELECT id, revision FROM success_contracts WHERE mission_id = ?1 AND revision = ?2",
            params![input.mission_id, i64::from(input.contract_revision)],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((contract_id, revision)) = contract else {
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM missions WHERE id = ?1)",
            [&input.mission_id],
            |row| row.get(0),
        )?;
        return if exists {
            Err(TramuriError::ContractNotFound {
                mission_id: input.mission_id,
                revision: input.contract_revision,
            })
        } else {
            Err(TramuriError::MissionNotFound(input.mission_id))
        };
    };
    let now = now_unix_ms()?;
    let id = Uuid::new_v4().to_string();
    let end = (input.state != ExecutionState::InProgress).then_some(now);
    connection.execute(
        "INSERT INTO executions (id, mission_id, contract_id, contract_revision, actor_kind, actor_reference, state, started_at_unix_ms, ended_at_unix_ms, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![id, input.mission_id, contract_id, revision, input.actor_kind.as_str(), input.actor_reference.as_deref().map(str::trim).filter(|s| !s.is_empty()), input.state.as_str(), now, end, input.notes.as_deref().map(str::trim).filter(|s| !s.is_empty())],
    )?;
    Ok(Execution {
        id,
        mission_id: input.mission_id,
        contract_id,
        contract_revision: input.contract_revision,
        actor_kind: input.actor_kind,
        actor_reference: input.actor_reference,
        state: input.state,
        started_at_unix_ms: now,
        ended_at_unix_ms: end,
        notes: input.notes,
    })
}

pub fn evidence_add(path: impl AsRef<Path>, input: EvidenceInput) -> Result<EvidenceReference> {
    let workspace = canonical_workspace(path.as_ref())?;
    let connection = open_database(&workspace)?;
    let belongs: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM executions WHERE id = ?1 AND mission_id = ?2)",
        params![input.execution_id, input.mission_id],
        |row| row.get(0),
    )?;
    if !belongs {
        return Err(TramuriError::ExecutionNotFound(input.execution_id));
    }
    let mut locator = input.locator.trim().to_owned();
    if locator.is_empty() {
        return Err(TramuriError::InvalidEvidenceReference(locator));
    }
    let mut scan_id = None;
    let mut sha256 = None;
    let mut hash_status = None;
    let mut availability = "unverified";
    if input.kind == EvidenceKind::ProjectFile {
        locator = normalize_source_path(&locator)
            .map_err(|_| TramuriError::InvalidEvidenceReference(input.locator.clone()))?;
        let latest = latest_scan(&connection)?
            .ok_or_else(|| TramuriError::InvalidEvidenceReference(locator.clone()))?;
        let file = load_inventory_file(&connection, &locator)?
            .filter(|item| item.last_seen_scan_id == Some(latest.id))
            .ok_or_else(|| TramuriError::InvalidEvidenceReference(locator.clone()))?;
        let current_metadata = safe_source_metadata(&workspace, &locator);
        match current_metadata {
            SourceMetadata::Unavailable => {
                return Err(TramuriError::InvalidEvidenceReference(locator));
            }
            SourceMetadata::Missing => availability = "missing",
            SourceMetadata::Available => {
                let source_path = workspace.join(&locator);
                let metadata = fs::symlink_metadata(&source_path)
                    .map_err(|_| TramuriError::InvalidEvidenceReference(locator.clone()))?;
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    return Err(TramuriError::InvalidEvidenceReference(locator));
                }
                let modified_unix_ms = metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .and_then(|d| i64::try_from(d.as_millis()).ok());
                if metadata.len() != file.size_bytes || modified_unix_ms != file.modified_unix_ms {
                    availability = "changed";
                }
            }
        }
        sha256 = file.sha256;
        if file.hash_status != HashStatus::ReadError {
            hash_status = Some(file.hash_status.clone());
        }
        scan_id = Some(latest.id);
        if latest.complete && file.hash_status == HashStatus::Hashed && availability == "unverified"
        {
            availability = "observed";
        }
    } else if input.kind == EvidenceKind::ExternalReference {
        validate_external_reference(&locator)?;
    }
    let now = now_unix_ms()?;
    let id = Uuid::new_v4().to_string();
    connection.execute(
        "INSERT INTO mission_evidence (id, mission_id, execution_id, kind, locator, sha256, hash_status, scan_id, availability, captured_at_unix_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![id, input.mission_id, input.execution_id, input.kind.as_str(), locator, sha256, hash_status.as_ref().map(HashStatus::as_storage_value), scan_id, availability, now],
    )?;
    Ok(EvidenceReference {
        id,
        mission_id: input.mission_id,
        execution_id: input.execution_id,
        kind: input.kind,
        locator,
        sha256,
        hash_status,
        availability: availability.to_owned(),
        scan_id,
        captured_at_unix_ms: now,
    })
}

pub fn outcome_assess(
    path: impl AsRef<Path>,
    input: CriterionAssessmentInput,
) -> Result<CriterionAssessment> {
    let verifier_reference = input.verifier_reference.trim();
    if verifier_reference.is_empty() {
        return Err(TramuriError::EmptyVerifierReference);
    }
    if input.result != CriterionAssessmentResult::Unverified && input.evidence_ids.is_empty() {
        return Err(TramuriError::MissingAssessmentEvidence);
    }
    let mut evidence_ids = input.evidence_ids;
    evidence_ids.sort();
    evidence_ids.dedup();

    let workspace = canonical_workspace(path.as_ref())?;
    let mut connection = open_database(&workspace)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let contract_id: Option<String> = transaction
        .query_row(
            "SELECT id FROM success_contracts WHERE mission_id = ?1 AND revision = ?2",
            params![input.mission_id, i64::from(input.contract_revision)],
            |row| row.get(0),
        )
        .optional()?;
    let Some(contract_id) = contract_id else {
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM missions WHERE id = ?1)",
            [&input.mission_id],
            |row| row.get(0),
        )?;
        return if exists {
            Err(TramuriError::ContractNotFound {
                mission_id: input.mission_id,
                revision: input.contract_revision,
            })
        } else {
            Err(TramuriError::MissionNotFound(input.mission_id))
        };
    };
    let criterion_exists: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM success_criteria
         WHERE id = ?1 AND contract_id = ?2 AND contract_revision = ?3)",
        params![
            input.criterion_id,
            contract_id,
            i64::from(input.contract_revision)
        ],
        |row| row.get(0),
    )?;
    if !criterion_exists {
        return Err(TramuriError::CriterionNotFound(input.criterion_id));
    }
    for evidence_id in &evidence_ids {
        let belongs: bool = transaction.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM mission_evidence AS evidence
                JOIN executions AS execution ON execution.id = evidence.execution_id
                WHERE evidence.id = ?1 AND evidence.mission_id = ?2
                  AND execution.contract_id = ?3 AND execution.contract_revision = ?4
            )",
            params![
                evidence_id,
                input.mission_id,
                contract_id,
                i64::from(input.contract_revision)
            ],
            |row| row.get(0),
        )?;
        if !belongs {
            return Err(TramuriError::AssessmentEvidenceNotFound(
                evidence_id.clone(),
            ));
        }
    }

    let id = Uuid::new_v4().to_string();
    let assessed_at_unix_ms = now_unix_ms()?;
    let explanation = input
        .explanation
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    transaction.execute(
        "INSERT INTO criterion_assessments
            (id, contract_id, contract_revision, criterion_id, result, verifier_kind,
             verifier_reference, assessed_at_unix_ms, explanation)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            contract_id,
            i64::from(input.contract_revision),
            input.criterion_id,
            input.result.as_str(),
            input.verifier_kind.as_str(),
            verifier_reference,
            assessed_at_unix_ms,
            explanation
        ],
    )?;
    for evidence_id in &evidence_ids {
        transaction.execute(
            "INSERT INTO criterion_assessment_evidence (assessment_id, evidence_id)
             VALUES (?1, ?2)",
            params![id, evidence_id],
        )?;
    }
    transaction.commit()?;

    Ok(CriterionAssessment {
        id,
        criterion_id: input.criterion_id,
        contract_revision: input.contract_revision,
        result: input.result,
        evidence_ids,
        verifier_kind: input.verifier_kind,
        verifier_reference: verifier_reference.to_owned(),
        assessed_at_unix_ms,
        explanation: explanation.map(str::to_owned),
    })
}

pub fn outcome_get(
    path: impl AsRef<Path>,
    mission_id: &str,
    contract_revision: Option<u32>,
) -> Result<VerifiedOutcome> {
    let workspace = canonical_workspace(path.as_ref())?;
    let connection = open_database(&workspace)?;
    let stored_contract: Option<(String, i64)> = connection
        .query_row(
            "SELECT id, revision FROM success_contracts
             WHERE mission_id = ?1 AND (?2 IS NULL OR revision = ?2)
             ORDER BY revision DESC LIMIT 1",
            params![mission_id, contract_revision.map(i64::from)],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((contract_id, revision)) = stored_contract else {
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM missions WHERE id = ?1)",
            [mission_id],
            |row| row.get(0),
        )?;
        return if exists {
            Err(TramuriError::ContractNotFound {
                mission_id: mission_id.to_owned(),
                revision: contract_revision.unwrap_or_default(),
            })
        } else {
            Err(TramuriError::MissionNotFound(mission_id.to_owned()))
        };
    };
    let revision = u32::try_from(revision).map_err(|_| TramuriError::ContractRevisionOverflow)?;
    let mut statement = connection.prepare(
        "SELECT id, description, required, evidence_expectation, position
         FROM success_criteria WHERE contract_id = ?1 AND contract_revision = ?2 ORDER BY position",
    )?;
    let criteria = statement
        .query_map(params![contract_id, i64::from(revision)], |row| {
            let required: i64 = row.get(2)?;
            let position: i64 = row.get(4)?;
            Ok(Criterion {
                id: row.get(0)?,
                description: row.get(1)?,
                required: required != 0,
                evidence_expectation: row.get(3)?,
                position: u32::try_from(position)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(4, position))?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    drop(statement);

    let mut criterion_outcomes = Vec::with_capacity(criteria.len());
    for criterion in criteria {
        let mut assessment_statement = connection.prepare(
            "SELECT id, result, verifier_kind, verifier_reference, assessed_at_unix_ms, explanation
             FROM criterion_assessments
             WHERE contract_id = ?1 AND contract_revision = ?2 AND criterion_id = ?3
             ORDER BY assessed_at_unix_ms, rowid",
        )?;
        let stored_assessments = assessment_statement
            .query_map(
                params![contract_id, i64::from(revision), criterion.id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )?
            .collect::<std::result::Result<Vec<StoredAssessment>, _>>()?;
        drop(assessment_statement);

        let mut assessment_history = Vec::with_capacity(stored_assessments.len());
        for (id, result, verifier_kind, verifier_reference, assessed_at_unix_ms, explanation) in
            stored_assessments
        {
            let mut evidence_statement = connection.prepare(
                "SELECT evidence_id FROM criterion_assessment_evidence
                 WHERE assessment_id = ?1 ORDER BY evidence_id",
            )?;
            let evidence_ids = evidence_statement
                .query_map([&id], |row| row.get(0))?
                .collect::<std::result::Result<Vec<String>, _>>()?;
            assessment_history.push(CriterionAssessment {
                id,
                criterion_id: criterion.id.clone(),
                contract_revision: revision,
                result: CriterionAssessmentResult::from_storage_value(&result).ok_or_else(
                    || {
                        rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            format!("unknown assessment result: {result}").into(),
                        )
                    },
                )?,
                evidence_ids,
                verifier_kind: VerifierKind::from_storage_value(&verifier_kind).ok_or_else(
                    || {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            format!("unknown verifier kind: {verifier_kind}").into(),
                        )
                    },
                )?,
                verifier_reference,
                assessed_at_unix_ms,
                explanation,
            });
        }
        criterion_outcomes.push(CriterionOutcome {
            criterion,
            assessment_history,
        });
    }

    let mut has_pending_required = false;
    let mut has_failed_required = false;
    for criterion in &criterion_outcomes {
        if !criterion.criterion.required {
            continue;
        }
        match criterion.assessment_history.last() {
            Some(assessment)
                if assessment.result == CriterionAssessmentResult::NotMet
                    && !assessment.evidence_ids.is_empty() =>
            {
                has_failed_required = true;
            }
            Some(assessment) if assessment.result == CriterionAssessmentResult::Met => {
                let mut all_evidence_resolves = !assessment.evidence_ids.is_empty();
                for evidence_id in &assessment.evidence_ids {
                    all_evidence_resolves &=
                        evidence_is_resolvable(&connection, &workspace, evidence_id)?;
                }
                has_pending_required |= !all_evidence_resolves;
            }
            _ => has_pending_required = true,
        }
    }
    let state = if has_failed_required {
        OutcomeState::NotAchieved
    } else if has_pending_required {
        OutcomeState::Pending
    } else {
        OutcomeState::Verified
    };

    Ok(VerifiedOutcome {
        mission_id: mission_id.to_owned(),
        contract_id,
        contract_revision: revision,
        state,
        criteria: criterion_outcomes,
    })
}

type StoredAssessment = (String, String, String, String, i64, Option<String>);
type EvidenceResolutionRecord = (String, String, Option<String>, Option<String>, String);

fn evidence_is_resolvable(
    connection: &rusqlite::Connection,
    workspace: &Path,
    evidence_id: &str,
) -> Result<bool> {
    let stored: Option<EvidenceResolutionRecord> = connection
        .query_row(
            "SELECT kind, locator, sha256, hash_status, availability
             FROM mission_evidence WHERE id = ?1",
            [evidence_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()?;
    let Some((kind, locator, sha256, hash_status, availability)) = stored else {
        return Ok(false);
    };
    if kind != "project_file" {
        // External references are deliberately never fetched; their persisted, validated locator
        // remains the evidence, and human attestations are provenance records rather than identity.
        return Ok(true);
    }
    if availability != "observed" || hash_status.as_deref() != Some("hashed") || sha256.is_none() {
        return Ok(false);
    }
    let Some(latest) = latest_scan(connection)? else {
        return Ok(false);
    };
    if !latest.complete {
        return Ok(false);
    }
    let Some(file) = load_inventory_file(connection, &locator)? else {
        return Ok(false);
    };
    if file.last_seen_scan_id != Some(latest.id)
        || file.hash_status != HashStatus::Hashed
        || file.sha256 != sha256
    {
        return Ok(false);
    }
    if !matches!(
        safe_source_metadata(workspace, &locator),
        SourceMetadata::Available
    ) {
        return Ok(false);
    }
    let path = workspace.join(&locator);
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => metadata,
        _ => return Ok(false),
    };
    let modified_unix_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .and_then(|duration| i64::try_from(duration.as_millis()).ok());
    Ok(metadata.len() == file.size_bytes && modified_unix_ms == file.modified_unix_ms)
}

fn validate_external_reference(value: &str) -> Result<()> {
    let Some((scheme, rest)) = value.split_once("://") else {
        return Err(TramuriError::InvalidEvidenceReference(value.to_owned()));
    };
    let authority = rest.split('/').next().unwrap_or_default();
    if !matches!(scheme.to_ascii_lowercase().as_str(), "http" | "https")
        || authority.is_empty()
        || authority.contains('@')
        || value.chars().any(char::is_whitespace)
        || value.contains('?')
        || value.contains('#')
    {
        return Err(TramuriError::InvalidEvidenceReference(value.to_owned()));
    }
    Ok(())
}

pub fn mission_create(path: impl AsRef<Path>, input: MissionInput) -> Result<Mission> {
    let objective = input.objective.trim();
    if objective.is_empty() {
        return Err(TramuriError::EmptyMissionObjective);
    }
    validate_criteria(&input.criteria)?;

    let workspace = canonical_workspace(path.as_ref())?;
    let mut connection = open_database(&workspace)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let now = now_unix_ms()?;
    let mission_id = Uuid::new_v4().to_string();
    let contract_id = Uuid::new_v4().to_string();
    transaction.execute(
        "INSERT INTO missions (id, objective, context, state, created_at_unix_ms)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            mission_id,
            objective,
            input
                .context
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
            MissionState::Open.as_storage_value(),
            now,
        ],
    )?;
    insert_contract(
        &transaction,
        &contract_id,
        &mission_id,
        1,
        now,
        &input.criteria,
    )?;
    transaction.commit()?;

    mission_get(&workspace, &mission_id)
}

pub fn mission_get(path: impl AsRef<Path>, mission_id: &str) -> Result<Mission> {
    let workspace = canonical_workspace(path.as_ref())?;
    let connection = open_database(&workspace)?;
    let stored: Option<(String, Option<String>, String, i64)> = connection
        .query_row(
            "SELECT objective, context, state, created_at_unix_ms FROM missions WHERE id = ?1",
            [mission_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()?;
    let Some((objective, context, state, created_at_unix_ms)) = stored else {
        return Err(TramuriError::MissionNotFound(mission_id.to_owned()));
    };
    let state = MissionState::from_storage_value(&state).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            format!("unknown mission state: {state}").into(),
        )
    })?;

    let (contract_id, revision, contract_created_at_unix_ms): (String, i64, i64) = connection
        .query_row(
            "SELECT id, revision, created_at_unix_ms FROM success_contracts
             WHERE mission_id = ?1 ORDER BY revision DESC LIMIT 1",
            [mission_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
    let mut statement = connection.prepare(
        "SELECT id, description, required, evidence_expectation, position
         FROM success_criteria WHERE contract_id = ?1 AND contract_revision = ?2 ORDER BY position",
    )?;
    let criteria = statement
        .query_map(params![&contract_id, revision], |row| {
            let required: i64 = row.get(2)?;
            let position: i64 = row.get(4)?;
            Ok(Criterion {
                id: row.get(0)?,
                description: row.get(1)?,
                required: required != 0,
                evidence_expectation: row.get(3)?,
                position: u32::try_from(position)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(4, position))?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let revision = u32::try_from(revision).map_err(|_| TramuriError::ContractRevisionOverflow)?;

    Ok(Mission {
        id: mission_id.to_owned(),
        objective,
        context,
        state,
        created_at_unix_ms,
        success_contract: SuccessContract {
            id: contract_id,
            mission_id: mission_id.to_owned(),
            revision,
            created_at_unix_ms: contract_created_at_unix_ms,
            criteria,
        },
    })
}

pub fn mission_close(path: impl AsRef<Path>, mission_id: &str) -> Result<Mission> {
    let workspace = canonical_workspace(path.as_ref())?;
    let connection = open_database(&workspace)?;
    let changed = connection.execute(
        "UPDATE missions SET state = ?2 WHERE id = ?1",
        params![mission_id, MissionState::Closed.as_storage_value()],
    )?;
    if changed == 0 {
        return Err(TramuriError::MissionNotFound(mission_id.to_owned()));
    }
    mission_get(&workspace, mission_id)
}

pub fn mission_contract_revise(
    path: impl AsRef<Path>,
    mission_id: &str,
    criteria: Vec<CriterionInput>,
) -> Result<Mission> {
    validate_criteria(&criteria)?;
    let workspace = canonical_workspace(path.as_ref())?;
    let mut connection = open_database(&workspace)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current: Option<(String, i64)> = transaction
        .query_row(
            "SELECT id, revision FROM success_contracts
             WHERE mission_id = ?1 ORDER BY revision DESC LIMIT 1",
            [mission_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((contract_id, current_revision)) = current else {
        return Err(TramuriError::MissionNotFound(mission_id.to_owned()));
    };
    let revision = current_revision
        .checked_add(1)
        .ok_or(TramuriError::ContractRevisionOverflow)?;
    let revision = u32::try_from(revision).map_err(|_| TramuriError::ContractRevisionOverflow)?;
    let now = now_unix_ms()?;
    insert_contract(
        &transaction,
        &contract_id,
        mission_id,
        revision,
        now,
        &criteria,
    )?;
    transaction.commit()?;

    mission_get(&workspace, mission_id)
}

fn validate_criteria(criteria: &[CriterionInput]) -> Result<()> {
    if !criteria.iter().any(|criterion| criterion.required) {
        return Err(TramuriError::NoRequiredCriterion);
    }
    if criteria
        .iter()
        .any(|criterion| criterion.description.trim().is_empty())
    {
        return Err(TramuriError::EmptySuccessCriterion);
    }
    Ok(())
}

fn insert_contract(
    transaction: &rusqlite::Transaction<'_>,
    contract_id: &str,
    mission_id: &str,
    revision: u32,
    created_at_unix_ms: i64,
    criteria: &[CriterionInput],
) -> Result<()> {
    transaction.execute(
        "INSERT INTO success_contracts (id, mission_id, revision, created_at_unix_ms)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            contract_id,
            mission_id,
            i64::from(revision),
            created_at_unix_ms
        ],
    )?;
    for (position, criterion) in criteria.iter().enumerate() {
        let criterion_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO success_criteria
                (id, contract_id, contract_revision, description, required, evidence_expectation, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                criterion_id,
                contract_id,
                i64::from(revision),
                criterion.description.trim(),
                criterion.required,
                criterion
                    .evidence_expectation
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty()),
                i64::try_from(position).map_err(|_| TramuriError::ContractRevisionOverflow)?,
            ],
        )?;
    }
    Ok(())
}

fn canonical_workspace(path: &Path) -> Result<std::path::PathBuf> {
    fs::canonicalize(path).map_err(|source| TramuriError::Io {
        operation: "resolve Tramuri workspace",
        path: path.to_path_buf(),
        source,
    })
}

fn now_unix_ms() -> Result<i64> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TramuriError::Clock)?;
    i64::try_from(elapsed.as_millis()).map_err(|_| TramuriError::Clock)
}
