use std::path::PathBuf;

/// Errors returned by Tramuri's public Rust API.
#[derive(Debug, thiserror::Error)]
pub enum TramuriError {
    #[error("project directory does not exist or is not a directory: {0}")]
    InvalidProjectRoot(PathBuf),
    #[error("project is not initialized at {0}; run `tramuri init` first")]
    NotInitialized(PathBuf),
    #[error("Tramuri state path must be a real directory or regular file, not a symlink: {0}")]
    InvalidStatePath(PathBuf),
    #[error(
        "both legacy and current Tramuri databases exist; no data was changed (`{legacy_path}` and `{database_path}`)"
    )]
    ConflictingStateDatabases {
        legacy_path: PathBuf,
        database_path: PathBuf,
    },
    #[error("legacy Tramuri database is busy; close other Tramuri processes and retry: {0}")]
    DatabaseBusyForMigration(PathBuf),
    #[error("unsupported Tramuri database schema version {found}; this build supports {supported}")]
    UnsupportedSchema { found: i64, supported: u32 },
    #[error("path cannot be represented as UTF-8: {0:?}")]
    NonUtf8Path(PathBuf),
    #[error("failed to {operation} `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("SQLite operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("system clock is before the Unix epoch or outside the supported timestamp range")]
    Clock,
    #[error("knowledge statement must not be empty")]
    EmptyKnowledgeStatement,
    #[error("knowledge requires at least one source or explicit human-input provenance")]
    MissingKnowledgeProvenance,
    #[error("invalid knowledge source path: {0}")]
    InvalidKnowledgeSource(String),
    #[error("knowledge record was not found: {0}")]
    KnowledgeNotFound(String),
    #[error("declared capability was not found in the latest workspace scan: {0}")]
    CapabilityNotFound(String),
    #[error("declared capability cannot be linked in its current state: {0}")]
    InvalidCapabilityLink(String),
    #[error("capability source was not hashed and observed in the latest scan: {0}")]
    CapabilityNotFresh(String),
    #[error("knowledge record cannot be superseded because it is not current: {0}")]
    InvalidSupersession(String),
    #[error("knowledge record cannot be withdrawn because it is not current: {0}")]
    InvalidWithdrawal(String),
    #[error("mission objective must not be empty")]
    EmptyMissionObjective,
    #[error("a non-empty success contract must contain at least one required criterion")]
    NoRequiredCriterion,
    #[error("invalid context readiness assessment: {0}")]
    InvalidReadiness(String),
    #[error("success criterion description must not be empty")]
    EmptySuccessCriterion,
    #[error("mission was not found: {0}")]
    MissionNotFound(String),
    #[error("an existing Mission matches this objective; select one explicitly: {0}")]
    MissionSelectionRequired(String),
    #[error("success contract revision {revision} was not found for mission {mission_id}")]
    ContractNotFound { mission_id: String, revision: u32 },
    #[error("success contract revision is outside the supported range")]
    ContractRevisionOverflow,
    #[error("execution actor kind must be human, agent, or tool")]
    InvalidActorKind,
    #[error("actor reference must be 1 to 256 characters without control characters")]
    InvalidActorReference,
    #[error("execution state must be in_progress, completed, blocked, failed, or cancelled")]
    InvalidExecutionState,
    #[error("execution cannot transition from {from} to {to}")]
    InvalidExecutionTransition { from: String, to: String },
    #[error("invalid Tramuri workflow configuration: {0}")]
    InvalidWorkProfileConfig(String),
    #[error(
        "work perspective is ambiguous; choose --perspective product_engineer, fde, mixed, or other"
    )]
    AmbiguousWorkPerspective,
    #[error("execution was not found: {0}")]
    ExecutionNotFound(String),
    #[error("evidence reference was not found: {0}")]
    EvidenceNotFound(String),
    #[error("evidence kind must be project_file, external_reference, or human_attestation")]
    InvalidEvidenceKind,
    #[error("evidence reference is invalid or unsafe: {0}")]
    InvalidEvidenceReference(String),
    #[error("criterion assessment result must be met, not_met, or unverified")]
    InvalidAssessmentResult,
    #[error("criterion assessment verifier kind must be human or mechanism")]
    InvalidVerifierKind,
    #[error("criterion assessment verifier reference must not be empty")]
    EmptyVerifierReference,
    #[error("a met or not_met criterion assessment requires at least one evidence reference")]
    MissingAssessmentEvidence,
    #[error("criterion was not found in the requested Success Contract revision: {0}")]
    CriterionNotFound(String),
    #[error(
        "assessment evidence must belong to the same Mission and Success Contract revision: {0}"
    )]
    AssessmentEvidenceNotFound(String),
    #[error("Product Engineering DEFINE workflow is not started for Mission {0}")]
    DefineWorkflowNotFound(String),
    #[error("Product Engineering DEFINE requires a product_engineer Mission perspective: {0}")]
    DefineRequiresProductEngineer(String),
    #[error("invalid Product Engineering DEFINE operation: {0}")]
    InvalidDefine(String),
    #[error("Product Engineering DEFINE workflow changed concurrently; reload status and retry")]
    DefineRevisionConflict,
}

pub type Result<T> = std::result::Result<T, TramuriError>;
