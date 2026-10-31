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
