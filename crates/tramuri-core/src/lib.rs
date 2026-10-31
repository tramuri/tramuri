//! Shared domain API for Tramuri's local-first missions, knowledge, and project discovery.

mod discovery;
mod error;
mod hash_status;
mod knowledge;
mod mission;
mod storage;

pub use discovery::{FileRecord, InitReport, ScanReport, ScanWarning, init, scan};
pub use error::{Result, TramuriError};
pub use hash_status::HashStatus;
pub use knowledge::{
    KnowledgeFilter, KnowledgeFreshness, KnowledgeInput, KnowledgeProvenance,
    KnowledgeProvenanceKind, KnowledgeRecord, KnowledgeStatus, KnowledgeType, knowledge_add,
    knowledge_list, knowledge_withdraw,
};
pub use mission::{
    ActorKind, Criterion, CriterionInput, EvidenceInput, EvidenceKind, EvidenceReference,
    Execution, ExecutionInput, ExecutionState, Mission, MissionInput, MissionState,
    SuccessContract, evidence_add, execution_record, mission_close, mission_contract_revise,
    mission_create, mission_get,
};

/// The current on-disk SQLite schema version.
pub const SCHEMA_VERSION: u32 = 4;
