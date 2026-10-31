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
    #[error("knowledge record cannot be superseded because it is not current: {0}")]
    InvalidSupersession(String),
    #[error("knowledge record cannot be withdrawn because it is not current: {0}")]
    InvalidWithdrawal(String),
    #[error("mission objective must not be empty")]
    EmptyMissionObjective,
    #[error("a success contract must contain at least one required criterion")]
    NoRequiredCriterion,
    #[error("success criterion description must not be empty")]
    EmptySuccessCriterion,
    #[error("mission was not found: {0}")]
    MissionNotFound(String),
    #[error("success contract revision {revision} was not found for mission {mission_id}")]
    ContractNotFound { mission_id: String, revision: u32 },
    #[error("success contract revision is outside the supported range")]
    ContractRevisionOverflow,
    #[error("execution actor kind must be human, agent, or tool")]
    InvalidActorKind,
    #[error("execution state must be in_progress, completed, blocked, failed, or cancelled")]
    InvalidExecutionState,
    #[error("execution was not found: {0}")]
    ExecutionNotFound(String),
    #[error("evidence kind must be project_file, external_reference, or human_attestation")]
    InvalidEvidenceKind,
    #[error("evidence reference is invalid or unsafe: {0}")]
    InvalidEvidenceReference(String),
}

pub type Result<T> = std::result::Result<T, TramuriError>;
