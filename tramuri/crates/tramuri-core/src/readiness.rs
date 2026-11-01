use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::storage::open_database;
use crate::{Mission, Result, TramuriError, mission_get, mission_relationships};

pub const READINESS_POLICY_VERSION: &str = "tramuri.activity-readiness.v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessState {
    NotAssessed,
    Preparing,
    NeedsReview,
    Ready,
    Stale,
    Blocked,
}

/// A user-recorded readiness decision. `Stale` and `NotAssessed` are derived states.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessDecision {
    Preparing,
    NeedsReview,
    Ready,
    Blocked,
}

impl ReadinessDecision {
    fn as_str(self) -> &'static str {
        match self {
            Self::Preparing => "preparing",
            Self::NeedsReview => "needs_review",
            Self::Ready => "ready",
            Self::Blocked => "blocked",
        }
    }

    fn state(self) -> ReadinessState {
        match self {
            Self::Preparing => ReadinessState::Preparing,
            Self::NeedsReview => ReadinessState::NeedsReview,
            Self::Ready => ReadinessState::Ready,
            Self::Blocked => ReadinessState::Blocked,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessAssessmentInput {
    pub mission_id: String,
    /// Open activity key, such as `discover`, `prototype`, or a user-defined value.
    pub activity: String,
    pub decision: ReadinessDecision,
    pub criteria_applied: Vec<String>,
    pub gaps: Vec<String>,
    pub blockers: Vec<String>,
    pub accepted_uncertainties: Vec<String>,
    /// Caller-supplied attribution; this is not authenticated identity.
    pub actor_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReadinessAssessment {
    pub id: String,
    pub mission_id: String,
    pub activity: String,
    pub contract_revision: u32,
    pub policy_version: String,
    pub decision: ReadinessDecision,
    pub criteria_applied: Vec<String>,
    pub gaps: Vec<String>,
    pub blockers: Vec<String>,
    pub accepted_uncertainties: Vec<String>,
    pub context_references: Vec<String>,
    pub actor_reference: Option<String>,
    pub assessed_at_unix_ms: i64,
    pub context_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionReadiness {
    pub mission_id: String,
    pub activity: Option<String>,
    pub state: ReadinessState,
    /// Original recorded decision, retained when the current readiness is stale.
    pub assessed_state: Option<ReadinessDecision>,
    pub assessment: Option<ReadinessAssessment>,
}

pub fn readiness_assess(
    path: impl AsRef<Path>,
    input: ReadinessAssessmentInput,
) -> Result<ReadinessAssessment> {
    let activity = normalize_activity(&input.activity)?;
    let gaps = normalize_items(input.gaps)?;
    let blockers = normalize_items(input.blockers)?;
    let criteria_applied = normalize_items(input.criteria_applied)?;
    let accepted_uncertainties = normalize_items(input.accepted_uncertainties)?;
    if input.decision == ReadinessDecision::Blocked && blockers.is_empty() {
        return Err(TramuriError::InvalidReadiness(
            "blocked readiness requires at least one recorded blocker".to_owned(),
        ));
    }
    if !blockers.is_empty() && input.decision != ReadinessDecision::Blocked {
        return Err(TramuriError::InvalidReadiness(
            "a readiness assessment with critical blockers must remain blocked".to_owned(),
        ));
    }
    if input.decision == ReadinessDecision::Ready && !gaps.is_empty() {
        return Err(TramuriError::InvalidReadiness(
            "ready assessments cannot contain unresolved gaps; record accepted uncertainties separately".to_owned(),
        ));
    }
    if input.decision == ReadinessDecision::Ready && criteria_applied.is_empty() {
        return Err(TramuriError::InvalidReadiness(
            "ready assessments must record at least one applied readiness check".to_owned(),
        ));
    }
    let actor_reference = normalize_actor(input.actor_reference)?;

    let workspace = std::fs::canonicalize(path.as_ref()).map_err(|source| TramuriError::Io {
        operation: "resolve project directory",
        path: path.as_ref().to_path_buf(),
        source,
    })?;
    let mission = mission_get(&workspace, &input.mission_id)?;
    let (fingerprint, context_references) = context_fingerprint(&workspace, &mission)?;
    let now = now_unix_ms()?;
    let id = Uuid::new_v4().to_string();
    let connection = open_database(&workspace)?;
    connection.execute(
        "INSERT INTO readiness_assessments
         (id, mission_id, activity, contract_revision, policy_version, state,
          criteria_applied_json, gaps_json, blockers_json, accepted_uncertainties_json,
          context_references_json, actor_reference, assessed_at_unix_ms, context_fingerprint)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            id,
            input.mission_id,
            activity,
            i64::from(mission.success_contract.revision),
            READINESS_POLICY_VERSION,
            input.decision.as_str(),
            serialize_items(&criteria_applied)?,
            serialize_items(&gaps)?,
            serialize_items(&blockers)?,
            serialize_items(&accepted_uncertainties)?,
            serialize_items(&context_references)?,
            actor_reference,
            now,
            fingerprint,
        ],
    )?;

    Ok(ReadinessAssessment {
        id,
        mission_id: input.mission_id,
        activity,
        contract_revision: mission.success_contract.revision,
        policy_version: READINESS_POLICY_VERSION.to_owned(),
        decision: input.decision,
        criteria_applied,
        gaps,
        blockers,
        accepted_uncertainties,
        context_references,
        actor_reference,
        assessed_at_unix_ms: now,
        context_fingerprint: fingerprint,
    })
}

pub fn readiness_get(path: impl AsRef<Path>, mission_id: &str) -> Result<MissionReadiness> {
    let workspace = std::fs::canonicalize(path.as_ref()).map_err(|source| TramuriError::Io {
        operation: "resolve project directory",
        path: path.as_ref().to_path_buf(),
        source,
    })?;
    let mission = mission_get(&workspace, mission_id)?;
    let connection = open_database(&workspace)?;
    let stored = connection
        .query_row(
            "SELECT id, mission_id, activity, contract_revision, policy_version, state,
                    criteria_applied_json, gaps_json, blockers_json, accepted_uncertainties_json,
                    context_references_json, actor_reference, assessed_at_unix_ms,
                    context_fingerprint
             FROM readiness_assessments WHERE mission_id = ?1
             ORDER BY assessed_at_unix_ms DESC, id DESC LIMIT 1",
            [mission_id],
            readiness_from_row,
        )
        .optional()?;
    let Some(assessment) = stored else {
        return Ok(MissionReadiness {
            mission_id: mission_id.to_owned(),
            activity: None,
            state: ReadinessState::NotAssessed,
            assessed_state: None,
            assessment: None,
        });
    };
    let (current_fingerprint, _) = context_fingerprint(&workspace, &mission)?;
    let decision = assessment.decision;
    let activity = assessment.activity.clone();
    let is_stale = assessment.context_fingerprint != current_fingerprint;
    Ok(MissionReadiness {
        mission_id: mission_id.to_owned(),
        activity: Some(activity),
        state: if is_stale {
            ReadinessState::Stale
        } else {
            decision.state()
        },
        assessed_state: Some(decision),
        assessment: Some(assessment),
    })
}

pub fn readiness_history(
    path: impl AsRef<Path>,
    mission_id: &str,
) -> Result<Vec<ReadinessAssessment>> {
    let workspace = std::fs::canonicalize(path.as_ref()).map_err(|source| TramuriError::Io {
        operation: "resolve project directory",
        path: path.as_ref().to_path_buf(),
        source,
    })?;
    mission_get(&workspace, mission_id)?;
    let connection = open_database(&workspace)?;
    let mut statement = connection.prepare(
        "SELECT id, mission_id, activity, contract_revision, policy_version, state,
                criteria_applied_json, gaps_json, blockers_json, accepted_uncertainties_json,
                context_references_json, actor_reference, assessed_at_unix_ms,
                context_fingerprint
         FROM readiness_assessments WHERE mission_id = ?1
         ORDER BY assessed_at_unix_ms DESC, id DESC",
    )?;
    let rows = statement.query_map([mission_id], readiness_from_row)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

fn readiness_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReadinessAssessment> {
    let revision: i64 = row.get(3)?;
    let revision = u32::try_from(revision)
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, revision))?;
    let state: String = row.get(5)?;
    let decision = parse_decision_sql(&state)?;
    let criteria_applied: String = row.get(6)?;
    let gaps: String = row.get(7)?;
    let blockers: String = row.get(8)?;
    let uncertainties: String = row.get(9)?;
    let context_references: String = row.get(10)?;
    let parse = |column, value: String, label: &str| {
        serde_json::from_str(&value).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                column,
                rusqlite::types::Type::Text,
                format!("invalid readiness {label} JSON: {error}").into(),
            )
        })
    };
    Ok(ReadinessAssessment {
        id: row.get(0)?,
        mission_id: row.get(1)?,
        activity: row.get(2)?,
        contract_revision: revision,
        policy_version: row.get(4)?,
        decision,
        criteria_applied: parse(6, criteria_applied, "criteria applied")?,
        gaps: parse(7, gaps, "gaps")?,
        blockers: parse(8, blockers, "blockers")?,
        accepted_uncertainties: parse(9, uncertainties, "accepted uncertainties")?,
        context_references: parse(10, context_references, "context references")?,
        actor_reference: row.get(11)?,
        assessed_at_unix_ms: row.get(12)?,
        context_fingerprint: row.get(13)?,
    })
}

fn parse_decision_sql(value: &str) -> rusqlite::Result<ReadinessDecision> {
    match value {
        "preparing" => Ok(ReadinessDecision::Preparing),
        "needs_review" => Ok(ReadinessDecision::NeedsReview),
        "ready" => Ok(ReadinessDecision::Ready),
        "blocked" => Ok(ReadinessDecision::Blocked),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            4,
            rusqlite::types::Type::Text,
            format!("unknown readiness decision: {value}").into(),
        )),
    }
}

fn normalize_activity(value: &str) -> Result<String> {
    let normalized = value.trim();
    if normalized.is_empty() || normalized.len() > 80 || normalized.chars().any(char::is_control) {
        return Err(TramuriError::InvalidReadiness(
            "activity must be 1 to 80 characters without control characters".to_owned(),
        ));
    }
    Ok(normalized.to_owned())
}

fn normalize_items(values: Vec<String>) -> Result<Vec<String>> {
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        let value = value.trim();
        if value.is_empty() || value.len() > 2048 || value.chars().any(char::is_control) {
            return Err(TramuriError::InvalidReadiness(
                "readiness notes must be 1 to 2048 characters without control characters"
                    .to_owned(),
            ));
        }
        normalized.push(value.to_owned());
    }
    Ok(normalized)
}

fn normalize_actor(value: Option<String>) -> Result<Option<String>> {
    let value = value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    if value
        .as_ref()
        .is_some_and(|value| value.len() > 256 || value.chars().any(char::is_control))
    {
        return Err(TramuriError::InvalidActorReference);
    }
    Ok(value)
}

fn context_fingerprint(workspace: &Path, mission: &Mission) -> Result<(String, Vec<String>)> {
    let relationships = mission_relationships(workspace, &mission.id)?;
    let mut context_references: Vec<String> = relationships
        .knowledge
        .iter()
        .map(|link| format!("knowledge:{}", link.knowledge.id))
        .chain(
            relationships
                .capabilities
                .iter()
                .map(|link| format!("capability:{}:{}", link.capability_id, link.sha256_at_link)),
        )
        .collect();
    context_references.sort();
    let serialized = serde_json::to_vec(&(
        mission.objective.as_str(),
        &mission.success_contract,
        relationships,
    ))
    .map_err(|error| {
        TramuriError::InvalidReadiness(format!("cannot fingerprint context: {error}"))
    })?;
    let digest = Sha256::digest(serialized);
    Ok((
        digest.iter().map(|byte| format!("{byte:02x}")).collect(),
        context_references,
    ))
}

fn serialize_items(values: &[String]) -> Result<String> {
    serde_json::to_string(values).map_err(|error| {
        TramuriError::InvalidReadiness(format!("cannot store readiness notes: {error}"))
    })
}

fn now_unix_ms() -> Result<i64> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TramuriError::Clock)?
        .as_millis();
    i64::try_from(now).map_err(|_| TramuriError::Clock)
}
