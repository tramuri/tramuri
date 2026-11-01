use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{TransactionBehavior, params};
use serde::{Deserialize, Serialize};

use crate::capability::{Capability, CapabilityState};
use crate::discovery::canonical_project_root;
use crate::knowledge::{KnowledgeRecord, knowledge_record_by_id};
use crate::mission::ActorKind;
use crate::storage::{latest_scan, load_inventory_file, open_database};
use crate::{Result, TramuriError, capability_snapshot};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipFreshness {
    Current,
    Changed,
    Missing,
    Unverified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationshipProvenance {
    /// Actor attribution is supplied by the caller and is not authenticated by Tramuri.
    pub actor_kind: ActorKind,
    pub actor_reference: Option<String>,
    pub linked_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionKnowledgeRelationship {
    pub knowledge: KnowledgeRecord,
    pub provenance: RelationshipProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionCapabilityRelationship {
    pub capability_id: String,
    pub name_at_link: String,
    pub ecosystem_at_link: String,
    pub path_at_link: String,
    pub sha256_at_link: String,
    pub adapter_version_at_link: String,
    pub freshness: RelationshipFreshness,
    pub capability: Option<Capability>,
    pub provenance: RelationshipProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionRelationships {
    pub mission_id: String,
    pub knowledge: Vec<MissionKnowledgeRelationship>,
    pub capabilities: Vec<MissionCapabilityRelationship>,
}

/// Explicitly associate an existing Knowledge record with a Mission.
/// Repeating the same association is idempotent and preserves its original attribution.
pub fn mission_knowledge_link(
    path: impl AsRef<Path>,
    mission_id: &str,
    knowledge_id: &str,
    actor_kind: ActorKind,
    actor_reference: Option<&str>,
) -> Result<MissionRelationships> {
    let root = canonical_project_root(path.as_ref())?;
    let actor_reference = validate_actor_reference(actor_reference)?;
    let mut connection = open_database(&root)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    ensure_mission(&transaction, mission_id)?;
    knowledge_record_by_id(&transaction, &root, knowledge_id)?;
    transaction.execute(
        "INSERT INTO mission_knowledge_links
            (mission_id, knowledge_id, actor_kind, actor_reference, created_at_unix_ms)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(mission_id, knowledge_id) DO NOTHING",
        params![
            mission_id,
            knowledge_id,
            actor_kind.as_str(),
            actor_reference,
            now_unix_ms()?
        ],
    )?;
    transaction.commit()?;
    mission_relationships(&root, mission_id)
}

/// Explicitly associate a currently declared, recognized Capability with a Mission.
/// The link captures the exact source identity so later scans can flag it as changed or missing.
pub fn mission_capability_link(
    path: impl AsRef<Path>,
    mission_id: &str,
    capability_id: &str,
    actor_kind: ActorKind,
    actor_reference: Option<&str>,
) -> Result<MissionRelationships> {
    let root = canonical_project_root(path.as_ref())?;
    let actor_reference = validate_actor_reference(actor_reference)?;
    let snapshot = capability_snapshot(&root)?;
    let capability = snapshot
        .capabilities
        .iter()
        .find(|item| item.id == capability_id)
        .ok_or_else(|| TramuriError::CapabilityNotFound(capability_id.to_owned()))?;
    if !matches!(
        capability.state,
        CapabilityState::DeclaredAndRecognized | CapabilityState::Disabled
    ) {
        return Err(TramuriError::InvalidCapabilityLink(
            capability_id.to_owned(),
        ));
    }
    let source_sha256 = capability
        .sha256
        .as_deref()
        .ok_or_else(|| TramuriError::InvalidCapabilityLink(capability_id.to_owned()))?;
    let mut connection = open_database(&root)?;
    let latest = latest_scan(&connection)?;
    let observed_source = load_inventory_file(&connection, &capability.path)?;
    if !latest.is_some_and(|scan| {
        observed_source.as_ref().is_some_and(|source| {
            source.last_seen_scan_id == Some(scan.id)
                && source.hash_status == crate::HashStatus::Hashed
                && source.sha256.as_deref() == Some(source_sha256)
        })
    }) {
        return Err(TramuriError::CapabilityNotFresh(capability_id.to_owned()));
    }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    ensure_mission(&transaction, mission_id)?;
    transaction.execute(
        "INSERT INTO mission_capability_links
            (mission_id, capability_id, capability_name, ecosystem, source_path, source_sha256,
             adapter_version, actor_kind, actor_reference, created_at_unix_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(mission_id, capability_id) DO NOTHING",
        params![
            mission_id,
            capability.id,
            capability.name,
            capability.ecosystem,
            capability.path,
            source_sha256,
            capability.adapter_version,
            actor_kind.as_str(),
            actor_reference,
            now_unix_ms()?
        ],
    )?;
    transaction.commit()?;
    mission_relationships(&root, mission_id)
}

/// Remove an explicit Mission–Knowledge association. The Knowledge record is retained.
pub fn mission_knowledge_unlink(
    path: impl AsRef<Path>,
    mission_id: &str,
    knowledge_id: &str,
) -> Result<bool> {
    let root = canonical_project_root(path.as_ref())?;
    let connection = open_database(&root)?;
    ensure_mission(&connection, mission_id)?;
    Ok(connection.execute(
        "DELETE FROM mission_knowledge_links WHERE mission_id = ?1 AND knowledge_id = ?2",
        params![mission_id, knowledge_id],
    )? > 0)
}

/// Remove an explicit Mission–Capability association. The discovered declaration is retained.
pub fn mission_capability_unlink(
    path: impl AsRef<Path>,
    mission_id: &str,
    capability_id: &str,
) -> Result<bool> {
    let root = canonical_project_root(path.as_ref())?;
    let connection = open_database(&root)?;
    ensure_mission(&connection, mission_id)?;
    Ok(connection.execute(
        "DELETE FROM mission_capability_links WHERE mission_id = ?1 AND capability_id = ?2",
        params![mission_id, capability_id],
    )? > 0)
}

/// Return only relationships explicitly recorded for one Mission.
pub fn mission_relationships(
    path: impl AsRef<Path>,
    mission_id: &str,
) -> Result<MissionRelationships> {
    let root = canonical_project_root(path.as_ref())?;
    let connection = open_database(&root)?;
    ensure_mission(&connection, mission_id)?;
    let knowledge_rows = {
        let mut statement = connection.prepare(
            "SELECT knowledge_id, actor_kind, actor_reference, created_at_unix_ms
             FROM mission_knowledge_links WHERE mission_id = ?1
             ORDER BY created_at_unix_ms, knowledge_id",
        )?;
        statement
            .query_map([mission_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?
    };
    let mut knowledge = Vec::with_capacity(knowledge_rows.len());
    for (knowledge_id, actor_kind, actor_reference, linked_at_unix_ms) in knowledge_rows {
        knowledge.push(MissionKnowledgeRelationship {
            knowledge: knowledge_record_by_id(&connection, &root, &knowledge_id)?,
            provenance: RelationshipProvenance {
                actor_kind: parse_actor_kind(&actor_kind)?,
                actor_reference,
                linked_at_unix_ms,
            },
        });
    }

    let capabilities = capability_snapshot(&root)?;
    let latest = latest_scan(&connection)?;
    let by_id: BTreeMap<_, _> = capabilities
        .capabilities
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();
    let capability_rows = {
        let mut statement = connection.prepare(
            "SELECT capability_id, capability_name, ecosystem, source_path, source_sha256,
                    adapter_version, actor_kind, actor_reference, created_at_unix_ms
             FROM mission_capability_links WHERE mission_id = ?1
             ORDER BY created_at_unix_ms, capability_id",
        )?;
        statement
            .query_map([mission_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?
    };
    let mut linked_capabilities = Vec::with_capacity(capability_rows.len());
    for (
        capability_id,
        name_at_link,
        ecosystem_at_link,
        path_at_link,
        sha256_at_link,
        adapter_version_at_link,
        actor_kind,
        actor_reference,
        linked_at_unix_ms,
    ) in capability_rows
    {
        let current = by_id.get(capability_id.as_str()).copied().cloned();
        let source = load_inventory_file(&connection, &path_at_link)?;
        let source_observed = latest.is_some_and(|scan| {
            source.as_ref().is_some_and(|source| {
                source.last_seen_scan_id == Some(scan.id)
                    && source.hash_status == crate::HashStatus::Hashed
            })
        });
        let source_hash_changed = source_observed
            && source.as_ref().and_then(|source| source.sha256.as_deref())
                != Some(sha256_at_link.as_str());
        let freshness = match current.as_ref() {
            Some(item)
                if item.sha256.as_deref() == Some(sha256_at_link.as_str())
                    && item.adapter_version == adapter_version_at_link
                    && source_observed
                    && !source_hash_changed =>
            {
                RelationshipFreshness::Current
            }
            Some(item)
                if source_hash_changed
                    || (source_observed
                        && (item.sha256.as_deref() != Some(sha256_at_link.as_str())
                            || item.adapter_version != adapter_version_at_link)) =>
            {
                RelationshipFreshness::Changed
            }
            Some(_) => RelationshipFreshness::Unverified,
            None if capabilities.scan_complete => RelationshipFreshness::Missing,
            None => RelationshipFreshness::Unverified,
        };
        linked_capabilities.push(MissionCapabilityRelationship {
            capability_id,
            name_at_link,
            ecosystem_at_link,
            path_at_link,
            sha256_at_link,
            adapter_version_at_link,
            freshness,
            capability: current,
            provenance: RelationshipProvenance {
                actor_kind: parse_actor_kind(&actor_kind)?,
                actor_reference,
                linked_at_unix_ms,
            },
        });
    }
    Ok(MissionRelationships {
        mission_id: mission_id.to_owned(),
        knowledge,
        capabilities: linked_capabilities,
    })
}

fn ensure_mission(connection: &rusqlite::Connection, mission_id: &str) -> Result<()> {
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM missions WHERE id = ?1)",
        [mission_id],
        |row| row.get(0),
    )?;
    if exists {
        Ok(())
    } else {
        Err(TramuriError::MissionNotFound(mission_id.to_owned()))
    }
}

fn parse_actor_kind(value: &str) -> Result<ActorKind> {
    ActorKind::from_storage_value(value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Text,
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown stored relationship actor kind: {value}"),
            )
            .into(),
        )
        .into()
    })
}

fn validate_actor_reference(value: Option<&str>) -> Result<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 256 || trimmed.chars().any(char::is_control) {
        return Err(TramuriError::InvalidActorReference);
    }
    Ok(Some(trimmed.to_owned()))
}

fn now_unix_ms() -> Result<i64> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TramuriError::Clock)?
        .as_millis();
    i64::try_from(now).map_err(|_| TramuriError::Clock)
}
