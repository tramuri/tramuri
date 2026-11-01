use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::storage::open_database;
use crate::{
    Criterion, CriterionAssessmentResult, EvidenceKind, ExecutionState, Mission, MissionState,
    OutcomeState, Result, TramuriError, VerifierKind, evidence_list, execution_list, mission_get,
    outcome_get,
};

/// Compact Mission record for discovery without a known identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionListItem {
    pub id: String,
    pub objective: String,
    pub state: MissionState,
    pub created_at_unix_ms: i64,
    pub contract_revision: u32,
}

/// One recent, timestamped domain event derived from a persisted record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentActivity {
    pub mission_id: String,
    pub record_id: String,
    pub kind: String,
    pub occurred_at_unix_ms: i64,
}

/// Privacy-minimized execution projection for a local status report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionSummary {
    pub id: String,
    pub state: ExecutionState,
    pub actor_kind: crate::ActorKind,
    pub contract_revision: u32,
    pub started_at_unix_ms: i64,
    pub ended_at_unix_ms: Option<i64>,
}

/// Privacy-minimized evidence projection. Locators are intentionally omitted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceSummary {
    pub id: String,
    pub execution_id: String,
    pub kind: EvidenceKind,
    pub availability: String,
    pub captured_at_unix_ms: i64,
}

/// Assessment attribution without user-entered verifier references or explanations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssessmentSummary {
    pub id: String,
    pub result: CriterionAssessmentResult,
    pub evidence_ids: Vec<String>,
    pub verifier_kind: VerifierKind,
    pub assessed_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CriterionReview {
    pub criterion: Criterion,
    pub assessments: Vec<AssessmentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutcomeSummary {
    pub state: OutcomeState,
    pub contract_revision: u32,
    pub criteria: Vec<CriterionReview>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionStatus {
    pub id: String,
    pub objective: String,
    pub state: MissionState,
    pub created_at_unix_ms: i64,
    pub contract: crate::SuccessContract,
    pub contract_history: Vec<OutcomeSummary>,
    pub executions: Vec<ExecutionSummary>,
    pub evidence: Vec<EvidenceSummary>,
    pub outcome: OutcomeSummary,
}

/// Explicit, Mission-scoped context for a deliberate resume operation.
/// Unlike workspace status, this includes only Knowledge and Capabilities linked by the user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionResume {
    #[serde(flatten)]
    pub mission: MissionStatus,
    pub relationships: crate::MissionRelationships,
}

/// Versioned, read-only projection of the local Tramuri workspace.
///
/// Mission context, execution notes, evidence locators, verifier references, explanations, and
/// workspace Knowledge text are deliberately excluded. Explicit Mission-linked Knowledge is
/// available only through the deliberate Mission resume/relationships operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusSnapshot {
    pub schema: String,
    pub generated_at_unix_ms: i64,
    pub workspace: WorkspaceSummary,
    pub capability_summary: crate::CapabilitySummary,
    pub missions: Vec<MissionStatus>,
    pub recent_activity: Vec<RecentActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceSummary {
    pub mission_count: usize,
    pub open_mission_count: usize,
    pub closed_mission_count: usize,
}

/// List Missions in a stable order: open first, newest first, then ID.
///
/// Opening the database may migrate a supported older schema before this read.
pub fn mission_list(path: impl AsRef<Path>) -> Result<Vec<MissionListItem>> {
    let workspace = std::fs::canonicalize(path.as_ref()).map_err(|source| TramuriError::Io {
        operation: "resolve project directory",
        path: path.as_ref().to_path_buf(),
        source,
    })?;
    let connection = open_database(&workspace)?;
    let mut statement = connection.prepare(
        "SELECT id FROM missions
         ORDER BY CASE state WHEN 'open' THEN 0 ELSE 1 END, created_at_unix_ms DESC, id ASC",
    )?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ids.into_iter()
        .map(|id| {
            let mission: Mission = mission_get(&workspace, &id)?;
            Ok(MissionListItem {
                id: mission.id,
                objective: mission.objective,
                state: mission.state,
                created_at_unix_ms: mission.created_at_unix_ms,
                contract_revision: mission.success_contract.revision,
            })
        })
        .collect()
}

/// Build a versioned local status snapshot from persisted records without following evidence
/// locators or scanning source files. Opening a supported old database may run its existing
/// schema migration; the read does not change Mission, Execution, Evidence, or Outcome records.
pub fn status_snapshot(path: impl AsRef<Path>) -> Result<StatusSnapshot> {
    let workspace = std::fs::canonicalize(path.as_ref()).map_err(|source| TramuriError::Io {
        operation: "resolve project directory",
        path: path.as_ref().to_path_buf(),
        source,
    })?;
    let listed = mission_list(&workspace)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TramuriError::Clock)?
        .as_millis();
    let generated_at_unix_ms = i64::try_from(now).map_err(|_| TramuriError::Clock)?;
    let capability_summary = crate::capability::capability_summary(&workspace)?;
    let mut missions = Vec::with_capacity(listed.len());
    let mut recent_activity = Vec::new();

    for item in listed {
        let mission: Mission = mission_get(&workspace, &item.id)?;
        let executions = execution_list(&workspace, &item.id)?;
        let evidence = evidence_list(&workspace, &item.id)?;
        let connection = open_database(&workspace)?;
        let mut revision_statement = connection.prepare(
            "SELECT revision, created_at_unix_ms FROM success_contracts WHERE mission_id = ?1 ORDER BY revision",
        )?;
        let revisions = revision_statement
            .query_map([&item.id], |row| {
                let value: i64 = row.get(0)?;
                let revision = u32::try_from(value)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, value))?;
                Ok((revision, row.get::<_, i64>(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        drop(revision_statement);
        drop(connection);
        let outcomes = revisions
            .iter()
            .map(|(revision, _)| outcome_get(&workspace, &item.id, Some(*revision)))
            .collect::<Result<Vec<_>>>()?;
        let outcome = outcomes
            .last()
            .ok_or_else(|| TramuriError::ContractNotFound {
                mission_id: item.id.clone(),
                revision: mission.success_contract.revision,
            })?;

        for execution in &executions {
            recent_activity.push(RecentActivity {
                mission_id: item.id.clone(),
                record_id: execution.id.clone(),
                kind: format!("execution_{:?}", execution.state).to_lowercase(),
                occurred_at_unix_ms: execution
                    .ended_at_unix_ms
                    .unwrap_or(execution.started_at_unix_ms),
            });
        }
        for (index, (revision, created_at_unix_ms)) in revisions.iter().enumerate() {
            recent_activity.push(RecentActivity {
                mission_id: item.id.clone(),
                record_id: outcomes[index].contract_id.clone(),
                kind: if *revision == 1 {
                    "success_contract_created".to_owned()
                } else {
                    "success_contract_revised".to_owned()
                },
                occurred_at_unix_ms: *created_at_unix_ms,
            });
        }
        for reference in &evidence {
            recent_activity.push(RecentActivity {
                mission_id: item.id.clone(),
                record_id: reference.id.clone(),
                kind: "evidence_recorded".to_owned(),
                occurred_at_unix_ms: reference.captured_at_unix_ms,
            });
        }
        for revision_outcome in &outcomes {
            for criterion in &revision_outcome.criteria {
                for assessment in &criterion.assessment_history {
                    recent_activity.push(RecentActivity {
                        mission_id: item.id.clone(),
                        record_id: assessment.id.clone(),
                        kind: format!("criterion_{:?}", assessment.result).to_lowercase(),
                        occurred_at_unix_ms: assessment.assessed_at_unix_ms,
                    });
                }
            }
        }

        let executions = executions
            .into_iter()
            .map(|execution| ExecutionSummary {
                id: execution.id,
                state: execution.state,
                actor_kind: execution.actor_kind,
                contract_revision: execution.contract_revision,
                started_at_unix_ms: execution.started_at_unix_ms,
                ended_at_unix_ms: execution.ended_at_unix_ms,
            })
            .collect();
        let evidence = evidence
            .into_iter()
            .map(|reference| EvidenceSummary {
                id: reference.id,
                execution_id: reference.execution_id,
                kind: reference.kind,
                availability: reference.availability,
                captured_at_unix_ms: reference.captured_at_unix_ms,
            })
            .collect();
        let outcome_summary = summarize_outcome(outcome);
        let contract_history = outcomes.iter().map(summarize_outcome).collect();
        missions.push(MissionStatus {
            id: mission.id,
            objective: mission.objective,
            state: mission.state,
            created_at_unix_ms: mission.created_at_unix_ms,
            contract: mission.success_contract,
            executions,
            evidence,
            outcome: outcome_summary,
            contract_history,
        });
    }

    recent_activity.sort_by(|a, b| {
        b.occurred_at_unix_ms
            .cmp(&a.occurred_at_unix_ms)
            .then_with(|| a.mission_id.cmp(&b.mission_id))
            .then_with(|| a.record_id.cmp(&b.record_id))
    });
    recent_activity.truncate(20);
    let open_mission_count = missions
        .iter()
        .filter(|mission| mission.state == MissionState::Open)
        .count();
    let mission_count = missions.len();

    Ok(StatusSnapshot {
        schema: "tramuri.status.v1".to_owned(),
        generated_at_unix_ms,
        workspace: WorkspaceSummary {
            mission_count,
            open_mission_count,
            closed_mission_count: mission_count - open_mission_count,
        },
        capability_summary,
        missions,
        recent_activity,
    })
}

/// Return a Mission's normal status projection plus its explicitly associated context.
/// No workspace-wide Knowledge is attached and capability freshness is checked against the
/// latest local registry snapshot without following source locators.
pub fn mission_resume(path: impl AsRef<Path>, mission_id: &str) -> Result<MissionResume> {
    let snapshot = status_snapshot(path.as_ref())?;
    let mission = snapshot
        .missions
        .into_iter()
        .find(|item| item.id == mission_id)
        .ok_or_else(|| TramuriError::MissionNotFound(mission_id.to_owned()))?;
    let relationships = crate::mission_relationships(path, mission_id)?;
    Ok(MissionResume {
        mission,
        relationships,
    })
}

fn summarize_outcome(outcome: &crate::VerifiedOutcome) -> OutcomeSummary {
    OutcomeSummary {
        state: outcome.state,
        contract_revision: outcome.contract_revision,
        criteria: outcome
            .criteria
            .iter()
            .map(|criterion| CriterionReview {
                criterion: criterion.criterion.clone(),
                assessments: criterion
                    .assessment_history
                    .iter()
                    .map(|assessment| AssessmentSummary {
                        id: assessment.id.clone(),
                        result: assessment.result,
                        evidence_ids: assessment.evidence_ids.clone(),
                        verifier_kind: assessment.verifier_kind,
                        assessed_at_unix_ms: assessment.assessed_at_unix_ms,
                    })
                    .collect(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ActorKind, CriterionAssessmentInput, CriterionInput, EvidenceInput, ExecutionInput,
        MissionInput, VerifierKind, evidence_add, execution_record, init, mission_contract_revise,
        mission_create, outcome_assess,
    };
    use tempfile::tempdir;

    #[test]
    fn status_snapshot_has_version_and_empty_workspace_counts() {
        let project = tempdir().unwrap();
        init(project.path()).unwrap();

        let snapshot = status_snapshot(project.path()).unwrap();

        assert_eq!(snapshot.schema, "tramuri.status.v1");
        assert_eq!(snapshot.workspace.mission_count, 0);
        assert_eq!(snapshot.workspace.open_mission_count, 0);
        assert_eq!(snapshot.workspace.closed_mission_count, 0);
        assert!(snapshot.missions.is_empty());
    }

    #[test]
    fn status_snapshot_does_not_promote_execution_or_export_freeform_fields() {
        let project = tempdir().unwrap();
        init(project.path()).unwrap();
        let mission = mission_create(
            project.path(),
            MissionInput {
                objective: "Prepare synthetic feature rollout".to_owned(),
                context: Some("context-secret-947".to_owned()),
                criteria: vec![CriterionInput {
                    description: "Synthetic acceptance check".to_owned(),
                    required: true,
                    evidence_expectation: None,
                }],
            },
        )
        .unwrap();
        let execution = execution_record(
            project.path(),
            ExecutionInput {
                mission_id: mission.id.clone(),
                contract_revision: 1,
                actor_kind: ActorKind::Human,
                actor_reference: Some("actor-secret-947".to_owned()),
                state: ExecutionState::Completed,
                notes: Some("notes-secret-947".to_owned()),
            },
        )
        .unwrap();
        let evidence = evidence_add(
            project.path(),
            EvidenceInput {
                mission_id: mission.id.clone(),
                execution_id: execution.id,
                kind: EvidenceKind::ExternalReference,
                locator: "https://example.test/evidence".to_owned(),
            },
        )
        .unwrap();
        outcome_assess(
            project.path(),
            CriterionAssessmentInput {
                mission_id: mission.id,
                contract_revision: 1,
                criterion_id: mission.success_contract.criteria[0].id.clone(),
                result: CriterionAssessmentResult::Unverified,
                evidence_ids: vec![evidence.id],
                verifier_kind: VerifierKind::Human,
                verifier_reference: "verifier-secret-947".to_owned(),
                explanation: Some("explanation-secret-947".to_owned()),
            },
        )
        .unwrap();

        let snapshot = status_snapshot(project.path()).unwrap();
        let serialized = serde_json::to_string(&snapshot).unwrap();
        let mission = &snapshot.missions[0];
        assert_eq!(mission.outcome.state, OutcomeState::Pending);
        assert_eq!(mission.executions[0].state, ExecutionState::Completed);
        assert_eq!(
            mission.outcome.criteria[0].assessments[0].result,
            CriterionAssessmentResult::Unverified
        );
        for private_value in [
            "context-secret-947",
            "actor-secret-947",
            "notes-secret-947",
            "https://example.test/evidence",
            "verifier-secret-947",
            "explanation-secret-947",
        ] {
            assert!(
                !serialized.contains(private_value),
                "serialized report leaked {private_value}"
            );
        }
    }

    #[test]
    fn mission_list_orders_open_first_then_recent_and_id() {
        let project = tempdir().unwrap();
        init(project.path()).unwrap();
        let first = mission_create(
            project.path(),
            MissionInput {
                objective: "First synthetic Mission".to_owned(),
                context: None,
                criteria: vec![CriterionInput {
                    description: "required".to_owned(),
                    required: true,
                    evidence_expectation: None,
                }],
            },
        )
        .unwrap();
        let second = mission_create(
            project.path(),
            MissionInput {
                objective: "Second synthetic Mission".to_owned(),
                context: None,
                criteria: vec![CriterionInput {
                    description: "required".to_owned(),
                    required: true,
                    evidence_expectation: None,
                }],
            },
        )
        .unwrap();
        crate::mission_close(project.path(), &second.id).unwrap();

        let missions = mission_list(project.path()).unwrap();

        assert_eq!(missions[0].id, first.id);
        assert_eq!(missions[1].id, second.id);
    }

    #[test]
    fn snapshot_keeps_current_and_previous_contract_revisions() {
        let project = tempdir().unwrap();
        init(project.path()).unwrap();
        let mission = mission_create(
            project.path(),
            MissionInput {
                objective: "Synthetic revision history".to_owned(),
                context: None,
                criteria: vec![CriterionInput {
                    description: "Initial criterion".to_owned(),
                    required: true,
                    evidence_expectation: None,
                }],
            },
        )
        .unwrap();
        mission_contract_revise(
            project.path(),
            &mission.id,
            vec![CriterionInput {
                description: "Revised criterion".to_owned(),
                required: true,
                evidence_expectation: None,
            }],
        )
        .unwrap();

        let snapshot = status_snapshot(project.path()).unwrap();
        let item = &snapshot.missions[0];

        assert_eq!(item.contract.revision, 2);
        assert_eq!(item.contract_history.len(), 2);
        assert_eq!(item.contract_history[0].contract_revision, 1);
        assert_eq!(
            item.contract_history[0].criteria[0].criterion.description,
            "Initial criterion"
        );
        assert_eq!(item.contract_history[1].contract_revision, 2);
        assert_eq!(
            item.outcome.criteria[0].criterion.description,
            "Revised criterion"
        );
    }
}
