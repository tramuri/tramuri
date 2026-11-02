//! Shared domain API for Tramuri's local-first missions, knowledge, and project discovery.

mod capability;
mod define;
mod discovery;
mod error;
mod hash_status;
mod knowledge;
mod mission;
mod profile;
mod readiness;
mod relationships;
mod status;
mod storage;

pub use capability::{
    Capability, CapabilityKind, CapabilityRelationship, CapabilitySnapshot, CapabilityState,
    CapabilitySummary, capability_list, capability_snapshot,
};
pub use define::{
    DefineArtifact, DefineArtifactReference, DefineBriefAssessment, DefineBriefCriterionAssessment,
    DefineEvent, DefineNext, DefineQuestion, DefineState, DefineStatus, DefineStep, DefineStepId,
    DefineStepState, DefineSubmitResult, define_artifact_history, define_brief_assess,
    define_brief_assessment_get, define_next, define_question_answer, define_question_request,
    define_review, define_start, define_status, define_submit,
};
pub use discovery::{FileRecord, InitReport, ScanReport, ScanWarning, init, scan};
pub use error::{Result, TramuriError};
pub use hash_status::HashStatus;
pub use knowledge::{
    KnowledgeFilter, KnowledgeFreshness, KnowledgeInput, KnowledgeProvenance,
    KnowledgeProvenanceKind, KnowledgeRecord, KnowledgeStatus, KnowledgeType, knowledge_add,
    knowledge_list, knowledge_withdraw,
};
pub use mission::{
    ActorKind, Criterion, CriterionAssessment, CriterionAssessmentInput, CriterionAssessmentResult,
    CriterionInput, CriterionOutcome, EvidenceInput, EvidenceKind, EvidenceReference, Execution,
    ExecutionInput, ExecutionState, Mission, MissionInput, MissionState, OutcomeState,
    SuccessContract, VerifiedOutcome, VerifierKind, WorkPerspective, evidence_add, evidence_get,
    evidence_list, execution_get, execution_list, execution_record, execution_transition,
    mission_close, mission_contract_revise, mission_create, mission_get, outcome_assess,
    outcome_get,
};
pub use profile::{
    AskWorkPerspective, IntentRoute, IntentRouteSource, MissionPerspective, MissionStartResult,
    MissionWorkflow, WorkPerspectiveChange, WorkProfile, WorkProfilePreference, mission_start,
    mission_workflow_activity_set, mission_workflow_get, mission_workflow_set, profile_get,
    profile_set, route_intent,
};
pub use readiness::{
    MissionReadiness, ReadinessAssessment, ReadinessAssessmentInput, ReadinessDecision,
    ReadinessState, readiness_assess, readiness_get, readiness_history,
};
pub use relationships::{
    MissionCapabilityRelationship, MissionKnowledgeRelationship, MissionRelationships,
    RelationshipFreshness, RelationshipProvenance, mission_capability_link,
    mission_capability_unlink, mission_knowledge_link, mission_knowledge_unlink,
    mission_relationships,
};
pub use status::{
    MissionListItem, MissionResume, MissionStatus, OutcomeSummary, RecentActivity, StatusSnapshot,
    WorkspaceSummary, mission_list, mission_resume, status_snapshot,
};
pub use storage::workspace_is_initialized;

/// The current on-disk SQLite schema version.
pub const SCHEMA_VERSION: u32 = 10;
