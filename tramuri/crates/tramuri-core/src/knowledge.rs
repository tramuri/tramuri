use std::fs;
use std::path::{Component, Path};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::discovery::is_sensitive_name;
use crate::storage::{latest_scan, load_inventory_file, open_database};
use crate::{HashStatus, Result, TramuriError};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeType {
    Fact,
    Decision,
    Inference,
    Question,
}

impl KnowledgeType {
    fn as_storage_value(self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Decision => "decision",
            Self::Inference => "inference",
            Self::Question => "question",
        }
    }

    fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "fact" => Some(Self::Fact),
            "decision" => Some(Self::Decision),
            "inference" => Some(Self::Inference),
            "question" => Some(Self::Question),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeStatus {
    Current,
    Superseded,
    Withdrawn,
}

impl KnowledgeStatus {
    fn as_storage_value(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Superseded => "superseded",
            Self::Withdrawn => "withdrawn",
        }
    }

    fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "current" => Some(Self::Current),
            "superseded" => Some(Self::Superseded),
            "withdrawn" => Some(Self::Withdrawn),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeFreshness {
    Current,
    Changed,
    Missing,
    Unverified,
    Unavailable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeProvenanceKind {
    ProjectFile,
    HumanInput,
}

impl KnowledgeProvenanceKind {
    fn as_storage_value(self) -> &'static str {
        match self {
            Self::ProjectFile => "project_file",
            Self::HumanInput => "human_input",
        }
    }

    fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "project_file" => Some(Self::ProjectFile),
            "human_input" => Some(Self::HumanInput),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeProvenance {
    pub kind: KnowledgeProvenanceKind,
    pub path: Option<String>,
    pub sha256: Option<String>,
    pub hash_status: Option<HashStatus>,
    pub captured_at_unix_ms: i64,
    pub freshness: KnowledgeFreshness,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeRecord {
    pub id: String,
    pub project_id: String,
    #[serde(rename = "type")]
    pub kind: KnowledgeType,
    pub statement: String,
    pub status: KnowledgeStatus,
    pub created_at_unix_ms: i64,
    pub supersedes: Option<String>,
    pub provenance: Vec<KnowledgeProvenance>,
    pub freshness: KnowledgeFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeInput {
    pub kind: KnowledgeType,
    pub statement: String,
    pub sources: Vec<String>,
    pub human_input: bool,
    pub supersedes: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KnowledgeFilter {
    pub kind: Option<KnowledgeType>,
    pub include_history: bool,
}

pub fn knowledge_add(path: impl AsRef<Path>, input: KnowledgeInput) -> Result<KnowledgeRecord> {
    if input.statement.trim().is_empty() {
        return Err(TramuriError::EmptyKnowledgeStatement);
    }
    if input.sources.is_empty() && !input.human_input {
        return Err(TramuriError::MissingKnowledgeProvenance);
    }
    if input.human_input
        && !matches!(
            input.kind,
            KnowledgeType::Decision | KnowledgeType::Question
        )
    {
        return Err(TramuriError::InvalidKnowledgeSource(
            "human-input provenance is supported for decisions and questions".to_owned(),
        ));
    }

    let project_root = fs::canonicalize(path.as_ref()).map_err(|source| TramuriError::Io {
        operation: "resolve project directory",
        path: path.as_ref().to_path_buf(),
        source,
    })?;
    let connection = open_database(&project_root)?;
    let observed_scan = if input.sources.is_empty() {
        None
    } else {
        Some(latest_scan(&connection)?.ok_or_else(|| {
            TramuriError::InvalidKnowledgeSource(
                "run `tramuri scan` before adding file-backed knowledge".to_owned(),
            )
        })?)
    };
    let mut sources = Vec::with_capacity(input.sources.len());
    for source in input.sources {
        let observed_scan = observed_scan.ok_or_else(|| {
            TramuriError::InvalidKnowledgeSource(
                "run `tramuri scan` before adding file-backed knowledge".to_owned(),
            )
        })?;
        let source = normalize_source_path(&source)?;
        let file = load_inventory_file(&connection, &source)?;
        let Some(file) = file else {
            return Err(TramuriError::InvalidKnowledgeSource(source));
        };
        if file.last_seen_scan_id != Some(observed_scan.id) {
            return Err(TramuriError::InvalidKnowledgeSource(format!(
                "`{source}` was not observed in the latest scan"
            )));
        }
        sources.push(file);
    }

    let record_id = Uuid::new_v4().to_string();
    let created_at = now_unix_ms()?;
    let transaction = connection.unchecked_transaction()?;
    if let Some(supersedes) = &input.supersedes {
        let status: Option<String> = transaction
            .query_row(
                "SELECT status FROM knowledge_records WHERE id = ?1",
                [supersedes],
                |row| row.get(0),
            )
            .optional()?;
        match status.as_deref() {
            Some("current") => {
                transaction.execute(
                    "UPDATE knowledge_records SET status = ?2 WHERE id = ?1",
                    params![supersedes, KnowledgeStatus::Superseded.as_storage_value()],
                )?;
            }
            Some(_) => return Err(TramuriError::InvalidSupersession(supersedes.clone())),
            None => return Err(TramuriError::KnowledgeNotFound(supersedes.clone())),
        }
    }
    transaction.execute(
        "INSERT INTO knowledge_records
            (id, record_type, statement, status, created_at_unix_ms, supersedes_id)
         VALUES (?1, ?2, ?3, 'current', ?4, ?5)",
        params![
            record_id,
            input.kind.as_storage_value(),
            input.statement.trim(),
            created_at,
            input.supersedes,
        ],
    )?;

    let mut position = 0_i64;
    for file in sources {
        let evidence_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO evidence_references
                (id, provenance_kind, source_path, sha256, hash_status, captured_at_unix_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                evidence_id,
                KnowledgeProvenanceKind::ProjectFile.as_storage_value(),
                file.path,
                file.sha256,
                match file.hash_status {
                    HashStatus::Hashed => Some("hashed"),
                    HashStatus::TooLarge => Some("too_large"),
                    HashStatus::ReadError => None,
                },
                created_at,
            ],
        )?;
        transaction.execute(
            "INSERT INTO knowledge_evidence (knowledge_id, evidence_id, position)
             VALUES (?1, ?2, ?3)",
            params![record_id, evidence_id, position],
        )?;
        position += 1;
    }
    if input.human_input {
        let evidence_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO evidence_references
                (id, provenance_kind, source_path, sha256, hash_status, captured_at_unix_ms)
             VALUES (?1, ?2, NULL, NULL, NULL, ?3)",
            params![
                evidence_id,
                KnowledgeProvenanceKind::HumanInput.as_storage_value(),
                created_at
            ],
        )?;
        transaction.execute(
            "INSERT INTO knowledge_evidence (knowledge_id, evidence_id, position)
             VALUES (?1, ?2, ?3)",
            params![record_id, evidence_id, position],
        )?;
    }
    transaction.commit()?;

    knowledge_list(
        &project_root,
        KnowledgeFilter {
            kind: Some(input.kind),
            include_history: true,
        },
    )?
    .into_iter()
    .find(|record| record.id == record_id)
    .ok_or_else(|| TramuriError::KnowledgeNotFound(record_id))
}

pub fn knowledge_list(
    path: impl AsRef<Path>,
    filter: KnowledgeFilter,
) -> Result<Vec<KnowledgeRecord>> {
    let project_root = fs::canonicalize(path.as_ref()).map_err(|source| TramuriError::Io {
        operation: "resolve project directory",
        path: path.as_ref().to_path_buf(),
        source,
    })?;
    let connection = open_database(&project_root)?;
    let project_id: String = connection.query_row(
        "SELECT project_id FROM project_metadata WHERE singleton_id = 1",
        [],
        |row| row.get(0),
    )?;
    let mut statement = connection.prepare(
        "SELECT id, record_type, statement, status, created_at_unix_ms, supersedes_id
         FROM knowledge_records
         WHERE (?1 IS NULL OR record_type = ?1)
           AND (?2 = 1 OR status = 'current')
         ORDER BY created_at_unix_ms, id",
    )?;
    let type_filter = filter.kind.map(KnowledgeType::as_storage_value);
    let rows = statement.query_map(
        params![
            type_filter,
            if filter.include_history { 1_i64 } else { 0_i64 }
        ],
        |row| {
            let type_value: String = row.get(1)?;
            let kind = KnowledgeType::from_storage_value(&type_value).ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("unknown stored knowledge type: {type_value}"),
                    )
                    .into(),
                )
            })?;
            let status_value: String = row.get(3)?;
            let status = KnowledgeStatus::from_storage_value(&status_value).ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("unknown stored knowledge status: {status_value}"),
                    )
                    .into(),
                )
            })?;
            Ok((
                row.get::<_, String>(0)?,
                kind,
                row.get::<_, String>(2)?,
                status,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        },
    )?;
    let mut records = Vec::new();
    for row in rows {
        let (id, kind, statement, status, created_at_unix_ms, supersedes) = row?;
        let provenance = load_provenance(&connection, &project_root, &id)?;
        let freshness = aggregate_freshness(&provenance);
        records.push(KnowledgeRecord {
            id,
            project_id: project_id.clone(),
            kind,
            statement,
            status,
            created_at_unix_ms,
            supersedes,
            provenance,
            freshness,
        });
    }
    Ok(records)
}

pub(crate) fn knowledge_record_by_id(
    connection: &rusqlite::Connection,
    project_root: &Path,
    id: &str,
) -> Result<KnowledgeRecord> {
    let project_id: String = connection.query_row(
        "SELECT project_id FROM project_metadata WHERE singleton_id = 1",
        [],
        |row| row.get(0),
    )?;
    let stored: Option<(KnowledgeType, String, KnowledgeStatus, i64, Option<String>)> = connection
        .query_row(
            "SELECT record_type, statement, status, created_at_unix_ms, supersedes_id
             FROM knowledge_records WHERE id = ?1",
            [id],
            |row| {
                let kind: String = row.get(0)?;
                let kind = KnowledgeType::from_storage_value(&kind).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("unknown stored knowledge type: {kind}"),
                        )
                        .into(),
                    )
                })?;
                let status: String = row.get(2)?;
                let status = KnowledgeStatus::from_storage_value(&status).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("unknown stored knowledge status: {status}"),
                        )
                        .into(),
                    )
                })?;
                Ok((kind, row.get(1)?, status, row.get(3)?, row.get(4)?))
            },
        )
        .optional()?;
    let Some((kind, statement, status, created_at_unix_ms, supersedes)) = stored else {
        return Err(TramuriError::KnowledgeNotFound(id.to_owned()));
    };
    let provenance = load_provenance(connection, project_root, id)?;
    let freshness = aggregate_freshness(&provenance);
    Ok(KnowledgeRecord {
        id: id.to_owned(),
        project_id,
        kind,
        statement,
        status,
        created_at_unix_ms,
        supersedes,
        provenance,
        freshness,
    })
}

pub fn knowledge_withdraw(path: impl AsRef<Path>, id: &str) -> Result<KnowledgeRecord> {
    let project_root = fs::canonicalize(path.as_ref()).map_err(|source| TramuriError::Io {
        operation: "resolve project directory",
        path: path.as_ref().to_path_buf(),
        source,
    })?;
    let mut connection = open_database(&project_root)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let status: Option<String> = transaction
        .query_row(
            "SELECT status FROM knowledge_records WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()?;
    match status.as_deref() {
        Some("current") => {
            transaction.execute(
                "UPDATE knowledge_records SET status = ?2 WHERE id = ?1",
                params![id, KnowledgeStatus::Withdrawn.as_storage_value()],
            )?;
        }
        Some(_) => return Err(TramuriError::InvalidWithdrawal(id.to_owned())),
        None => return Err(TramuriError::KnowledgeNotFound(id.to_owned())),
    }
    transaction.commit()?;
    knowledge_list(
        project_root,
        KnowledgeFilter {
            kind: None,
            include_history: true,
        },
    )?
    .into_iter()
    .find(|record| record.id == id)
    .ok_or_else(|| TramuriError::KnowledgeNotFound(id.to_owned()))
}

fn load_provenance(
    connection: &rusqlite::Connection,
    project_root: &Path,
    knowledge_id: &str,
) -> Result<Vec<KnowledgeProvenance>> {
    let mut statement = connection.prepare(
        "SELECT e.provenance_kind, e.source_path, e.sha256, e.hash_status,
                e.captured_at_unix_ms
         FROM evidence_references e
         INNER JOIN knowledge_evidence ke ON ke.evidence_id = e.id
         WHERE ke.knowledge_id = ?1
         ORDER BY ke.position",
    )?;
    let rows = statement.query_map([knowledge_id], |row| {
        let kind_value: String = row.get(0)?;
        let kind = KnowledgeProvenanceKind::from_storage_value(&kind_value).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("unknown stored provenance kind: {kind_value}"),
                )
                .into(),
            )
        })?;
        let status_value: Option<String> = row.get(3)?;
        let hash_status = status_value
            .as_deref()
            .map(|value| {
                HashStatus::from_storage_value(value).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("unknown stored hash status: {value}"),
                        )
                        .into(),
                    )
                })
            })
            .transpose()?;
        Ok((
            kind,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            hash_status,
            row.get::<_, i64>(4)?,
        ))
    })?;
    let latest = latest_scan(connection)?;
    let mut provenance = Vec::new();
    for row in rows {
        let (kind, path, sha256, hash_status, captured_at_unix_ms) = row?;
        let freshness = match (kind, path.as_deref()) {
            (KnowledgeProvenanceKind::HumanInput, _) => KnowledgeFreshness::Current,
            (KnowledgeProvenanceKind::ProjectFile, Some(path)) => {
                file_freshness(connection, project_root, path, sha256.as_deref(), latest)?
            }
            (KnowledgeProvenanceKind::ProjectFile, None) => KnowledgeFreshness::Unavailable,
        };
        provenance.push(KnowledgeProvenance {
            kind,
            path,
            sha256,
            hash_status,
            captured_at_unix_ms,
            freshness,
        });
    }
    Ok(provenance)
}

fn file_freshness(
    connection: &rusqlite::Connection,
    project_root: &Path,
    path: &str,
    captured_sha256: Option<&str>,
    latest: Option<crate::storage::ScanObservation>,
) -> Result<KnowledgeFreshness> {
    let Some(latest) = latest else {
        return Ok(KnowledgeFreshness::Unverified);
    };
    if let Some(file) = load_inventory_file(connection, path)? {
        if file.last_seen_scan_id == Some(latest.id) {
            if file.hash_status != HashStatus::Hashed {
                return Ok(KnowledgeFreshness::Unverified);
            }
            return Ok(match (captured_sha256, file.sha256.as_deref()) {
                (Some(captured), Some(current)) if captured == current => {
                    KnowledgeFreshness::Current
                }
                (Some(_), Some(_)) => KnowledgeFreshness::Changed,
                _ => KnowledgeFreshness::Unverified,
            });
        }
    }
    if !latest.complete {
        return Ok(KnowledgeFreshness::Unverified);
    }
    Ok(match safe_source_metadata(project_root, path) {
        SourceMetadata::Missing => KnowledgeFreshness::Missing,
        SourceMetadata::Available | SourceMetadata::Unavailable => KnowledgeFreshness::Unavailable,
    })
}

pub(crate) enum SourceMetadata {
    Available,
    Missing,
    Unavailable,
}

pub(crate) fn safe_source_metadata(project_root: &Path, source_path: &str) -> SourceMetadata {
    let mut path = project_root.to_path_buf();
    let components = source_path.split('/').collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        if is_sensitive_name(component) {
            return SourceMetadata::Unavailable;
        }
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return SourceMetadata::Unavailable;
            }
            Ok(metadata) if index + 1 < components.len() && !metadata.is_dir() => {
                return SourceMetadata::Unavailable;
            }
            Ok(metadata) if index + 1 == components.len() && metadata.is_file() => {
                return SourceMetadata::Available;
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return SourceMetadata::Missing;
            }
            Err(_) => return SourceMetadata::Unavailable,
        }
    }
    SourceMetadata::Unavailable
}

fn aggregate_freshness(provenance: &[KnowledgeProvenance]) -> KnowledgeFreshness {
    if provenance.is_empty() {
        return KnowledgeFreshness::Unverified;
    }
    let states = provenance
        .iter()
        .map(|item| item.freshness)
        .collect::<Vec<_>>();
    if states.contains(&KnowledgeFreshness::Changed) {
        KnowledgeFreshness::Changed
    } else if states
        .iter()
        .all(|state| *state == KnowledgeFreshness::Missing)
    {
        KnowledgeFreshness::Missing
    } else if states.contains(&KnowledgeFreshness::Unavailable) {
        KnowledgeFreshness::Unavailable
    } else if states.contains(&KnowledgeFreshness::Missing)
        || states.contains(&KnowledgeFreshness::Unverified)
    {
        KnowledgeFreshness::Unverified
    } else {
        KnowledgeFreshness::Current
    }
}

pub(crate) fn normalize_source_path(source: &str) -> Result<String> {
    if source.is_empty()
        || source.starts_with('/')
        || source.contains('\\')
        || source.contains(':')
        || source
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == ".." || is_sensitive_name(part))
    {
        return Err(TramuriError::InvalidKnowledgeSource(source.to_owned()));
    }
    let path = Path::new(source);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(TramuriError::InvalidKnowledgeSource(source.to_owned()));
    }
    Ok(source.to_owned())
}

fn now_unix_ms() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TramuriError::Clock)?;
    i64::try_from(duration.as_millis()).map_err(|_| TramuriError::Clock)
}
