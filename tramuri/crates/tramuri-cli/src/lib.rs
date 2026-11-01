use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde::Deserialize;
use tramuri_core::{
    ActorKind, CriterionAssessmentInput, CriterionAssessmentResult, CriterionInput, EvidenceInput,
    EvidenceKind, ExecutionInput, ExecutionState, KnowledgeFilter, KnowledgeInput, KnowledgeType,
    MissionInput, VerifierKind, evidence_add, evidence_get, evidence_list, execution_get,
    execution_list, execution_record, init, knowledge_add, knowledge_list, knowledge_withdraw,
    mission_close, mission_contract_revise, mission_create, mission_get, outcome_assess,
    outcome_get, scan,
};

mod integrations;

#[derive(Debug, Parser)]
#[command(
    name = "tramuri",
    version,
    about = "Local-first infrastructure for verified work outcomes"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize Tramuri's local project database.
    Init {
        /// Project directory (defaults to the current directory).
        path: Option<PathBuf>,
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Inventory project files and persist the latest complete snapshot.
    Scan {
        /// Project directory (defaults to the current directory).
        path: Option<PathBuf>,
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Store and retrieve source-backed project knowledge.
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommand,
    },
    /// Define and retrieve work objectives and success criteria.
    Mission {
        #[command(subcommand)]
        command: MissionCommand,
    },
    /// Record manual work against a mission.
    Execution {
        #[command(subcommand)]
        command: ExecutionCommand,
    },
    /// Attach a provenance-bearing reference to an execution.
    Evidence {
        #[command(subcommand)]
        command: EvidenceCommand,
    },
    /// Record criterion reviews and retrieve the aggregate Mission outcome.
    Outcome {
        #[command(subcommand)]
        command: OutcomeCommand,
    },
    /// Install Tramuri's project instructions and skill for an AI platform.
    Install {
        /// Install into the current or selected project directory.
        #[arg(long, required = true)]
        project: bool,
        /// Target project directory (defaults to the current directory).
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Platform integration to install.
        #[arg(long, value_enum)]
        platform: integrations::Platform,
    },
    /// Remove Tramuri-managed integration files from a project.
    Uninstall {
        /// Remove from a project directory (defaults to the current directory).
        #[arg(long, required = true)]
        project: bool,
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum)]
        platform: integrations::Platform,
    },
    /// Show installed Tramuri project integrations.
    Integrations {
        #[command(subcommand)]
        command: IntegrationCommand,
    },
}

#[derive(Debug, Subcommand)]
enum IntegrationCommand {
    Status {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum OutcomeCommand {
    Assess {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        mission: String,
        #[arg(long)]
        contract_revision: u32,
        #[arg(long)]
        criterion: String,
        #[arg(long)]
        result: String,
        #[arg(long, value_delimiter = ',')]
        evidence: Vec<String>,
        #[arg(long)]
        verifier_kind: String,
        #[arg(long)]
        verifier: String,
        #[arg(long)]
        explanation: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Show {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        mission: String,
        #[arg(long)]
        contract_revision: Option<u32>,
        #[arg(long)]
        json: bool,
    },
}

fn parse_assessment_result(
    value: &str,
) -> Result<CriterionAssessmentResult, Box<dyn std::error::Error>> {
    match value {
        "met" => Ok(CriterionAssessmentResult::Met),
        "not_met" => Ok(CriterionAssessmentResult::NotMet),
        "unverified" => Ok(CriterionAssessmentResult::Unverified),
        _ => Err("result must be met, not_met, or unverified".into()),
    }
}

fn parse_verifier_kind(value: &str) -> Result<VerifierKind, Box<dyn std::error::Error>> {
    match value {
        "human" => Ok(VerifierKind::Human),
        "mechanism" => Ok(VerifierKind::Mechanism),
        _ => Err("verifier kind must be human or mechanism".into()),
    }
}

#[derive(Debug, Subcommand)]
enum ExecutionCommand {
    Record {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        mission: String,
        #[arg(long)]
        contract_revision: u32,
        #[arg(long)]
        actor_kind: String,
        #[arg(long)]
        status: String,
        #[arg(long)]
        actor: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// List a Mission's execution history, newest first.
    List {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        mission: String,
        #[arg(long)]
        json: bool,
    },
    /// Retrieve one execution by ID.
    Show {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum EvidenceCommand {
    Add {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        mission: String,
        #[arg(long)]
        execution: String,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        locator: String,
        #[arg(long)]
        json: bool,
    },
    /// List a Mission's evidence references, newest first.
    List {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        mission: String,
        #[arg(long)]
        json: bool,
    },
    /// Retrieve one stored evidence reference by ID without following its locator.
    Show {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
}

fn parse_actor_kind(value: &str) -> Result<ActorKind, Box<dyn std::error::Error>> {
    match value {
        "human" => Ok(ActorKind::Human),
        "agent" => Ok(ActorKind::Agent),
        "tool" => Ok(ActorKind::Tool),
        _ => Err("actor kind must be human, agent, or tool".into()),
    }
}

fn parse_execution_state(value: &str) -> Result<ExecutionState, Box<dyn std::error::Error>> {
    match value {
        "in_progress" => Ok(ExecutionState::InProgress),
        "completed" => Ok(ExecutionState::Completed),
        "blocked" => Ok(ExecutionState::Blocked),
        "failed" => Ok(ExecutionState::Failed),
        "cancelled" => Ok(ExecutionState::Cancelled),
        _ => Err("status must be in_progress, completed, blocked, failed, or cancelled".into()),
    }
}

fn parse_evidence_kind(value: &str) -> Result<EvidenceKind, Box<dyn std::error::Error>> {
    match value {
        "project_file" => Ok(EvidenceKind::ProjectFile),
        "external_reference" => Ok(EvidenceKind::ExternalReference),
        "human_attestation" => Ok(EvidenceKind::HumanAttestation),
        _ => Err("kind must be project_file, external_reference, or human_attestation".into()),
    }
}

#[derive(Debug, Subcommand)]
enum MissionCommand {
    /// Create a Mission and its first Success Contract.
    Create {
        /// Initialized local Tramuri workspace (defaults to the current directory).
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Mission objective.
        #[arg(long)]
        objective: String,
        /// Optional context for the objective.
        #[arg(long)]
        context: Option<String>,
        /// JSON file containing a `criteria` array.
        #[arg(long)]
        criteria_file: PathBuf,
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Retrieve a Mission and its current Success Contract.
    Show {
        /// Initialized local Tramuri workspace (defaults to the current directory).
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Mission identifier.
        #[arg(long)]
        mission: String,
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Close a Mission without declaring its outcome verified.
    Close {
        /// Initialized local Tramuri workspace (defaults to the current directory).
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Mission identifier.
        #[arg(long)]
        mission: String,
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Manage Success Contract revisions.
    Contract {
        #[command(subcommand)]
        command: MissionContractCommand,
    },
}

#[derive(Debug, Subcommand)]
enum MissionContractCommand {
    /// Create a new Success Contract revision without rewriting prior revisions.
    Revise {
        /// Initialized local Tramuri workspace (defaults to the current directory).
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Mission identifier.
        #[arg(long)]
        mission: String,
        /// JSON file containing a `criteria` array.
        #[arg(long)]
        criteria_file: PathBuf,
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CriteriaFile {
    criteria: Vec<FileCriterion>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileCriterion {
    description: String,
    #[serde(default = "required_by_default")]
    required: bool,
    #[serde(default)]
    evidence_expectation: Option<String>,
}

fn required_by_default() -> bool {
    true
}

fn read_criteria(
    path: &std::path::Path,
) -> Result<Vec<CriterionInput>, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    let file: CriteriaFile = serde_json::from_slice(&bytes)?;
    Ok(file
        .criteria
        .into_iter()
        .map(|criterion| CriterionInput {
            description: criterion.description,
            required: criterion.required,
            evidence_expectation: criterion.evidence_expectation,
        })
        .collect())
}

#[derive(Debug, Subcommand)]
enum KnowledgeCommand {
    /// Add a fact, decision, inference, or open question.
    Add {
        /// Project directory (defaults to the current directory).
        path: Option<PathBuf>,
        /// Knowledge type.
        #[arg(long, value_enum)]
        r#type: KnowledgeKindArg,
        /// Knowledge statement.
        #[arg(long)]
        statement: String,
        /// Project-relative source path; may be provided more than once.
        #[arg(long = "source")]
        sources: Vec<String>,
        /// Mark the human input as provenance.
        #[arg(long)]
        human_input: bool,
        /// Current record explicitly superseded by this record.
        #[arg(long)]
        supersedes: Option<String>,
        /// Print the record as JSON.
        #[arg(long)]
        json: bool,
    },
    /// List current knowledge or its history.
    List {
        /// Project directory (defaults to the current directory).
        path: Option<PathBuf>,
        /// Filter by knowledge type.
        #[arg(long, value_enum)]
        r#type: Option<KnowledgeKindArg>,
        /// Include superseded and withdrawn records.
        #[arg(long)]
        include_history: bool,
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Withdraw a current record without deleting its history.
    Withdraw {
        /// Project directory (defaults to the current directory).
        path: Option<PathBuf>,
        /// Knowledge record ID.
        #[arg(long)]
        id: String,
        /// Print the updated record as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum KnowledgeKindArg {
    Fact,
    Decision,
    Inference,
    Question,
}

impl From<KnowledgeKindArg> for KnowledgeType {
    fn from(value: KnowledgeKindArg) -> Self {
        match value {
            KnowledgeKindArg::Fact => Self::Fact,
            KnowledgeKindArg::Decision => Self::Decision,
            KnowledgeKindArg::Inference => Self::Inference,
            KnowledgeKindArg::Question => Self::Question,
        }
    }
}

pub fn run_from<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match Cli::try_parse_from(args) {
        Ok(cli) => match run(cli) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("tramuri: {error}");
                1
            }
        },
        Err(error) => {
            let exit_code = error.exit_code();
            let _ = error.print();
            exit_code
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Init { path, json } => {
            let report = init(path.unwrap_or_else(|| PathBuf::from(".")))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else if report.already_initialized {
                println!("Tramuri is already initialized in {}", report.project_root);
            } else {
                println!("Initialized Tramuri in {}", report.project_root);
            }
        }
        Command::Scan { path, json } => {
            let report = scan(path.unwrap_or_else(|| PathBuf::from(".")))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "Scanned {} files in {} ({})",
                    report.files.len(),
                    report.project_root,
                    if report.complete {
                        "complete"
                    } else {
                        "partial"
                    }
                );
                println!(
                    "Added: {}, changed: {}, unchanged: {}, unverified: {}, deleted: {}, sensitive excluded: {}",
                    report.added.len(),
                    report.changed.len(),
                    report.unchanged.len(),
                    report.unverified.len(),
                    report.deleted.len(),
                    report.sensitive_entries_excluded
                );
                for warning in &report.warnings {
                    let path = warning.path.as_deref().unwrap_or("<project>");
                    eprintln!("warning [{}] {path}: {}", warning.code, warning.message);
                }
            }
        }
        Command::Knowledge { command } => match command {
            KnowledgeCommand::Add {
                path,
                r#type,
                statement,
                sources,
                human_input,
                supersedes,
                json,
            } => {
                let record = knowledge_add(
                    path.unwrap_or_else(|| PathBuf::from(".")),
                    KnowledgeInput {
                        kind: r#type.into(),
                        statement,
                        sources,
                        human_input,
                        supersedes,
                    },
                )?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&record)?);
                } else {
                    println!("Saved knowledge record {}", record.id);
                }
            }
            KnowledgeCommand::List {
                path,
                r#type,
                include_history,
                json,
            } => {
                let records = knowledge_list(
                    path.unwrap_or_else(|| PathBuf::from(".")),
                    KnowledgeFilter {
                        kind: r#type.map(Into::into),
                        include_history,
                    },
                )?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&records)?);
                } else if records.is_empty() {
                    println!("No knowledge records found.");
                } else {
                    for record in records {
                        println!(
                            "{}\t{:?}\t{:?}\t{:?}\t{}",
                            record.id,
                            record.kind,
                            record.status,
                            record.freshness,
                            record.statement
                        );
                    }
                }
            }
            KnowledgeCommand::Withdraw { path, id, json } => {
                let record = knowledge_withdraw(path.unwrap_or_else(|| PathBuf::from(".")), &id)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&record)?);
                } else {
                    println!("Withdrawn knowledge record {}", record.id);
                }
            }
        },
        Command::Mission { command } => match command {
            MissionCommand::Create {
                path,
                objective,
                context,
                criteria_file,
                json,
            } => {
                let mission = mission_create(
                    path,
                    MissionInput {
                        objective,
                        context,
                        criteria: read_criteria(&criteria_file)?,
                    },
                )?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&mission)?);
                } else {
                    println!("Created Mission {}", mission.id);
                }
            }
            MissionCommand::Show {
                path,
                mission,
                json,
            } => {
                let result = mission_get(path, &mission)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!("{}\t{:?}\t{}", result.id, result.state, result.objective);
                }
            }
            MissionCommand::Close {
                path,
                mission,
                json,
            } => {
                let result = mission_close(path, &mission)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!("Closed Mission {}", result.id);
                }
            }
            MissionCommand::Contract { command } => match command {
                MissionContractCommand::Revise {
                    path,
                    mission,
                    criteria_file,
                    json,
                } => {
                    let result =
                        mission_contract_revise(path, &mission, read_criteria(&criteria_file)?)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!(
                            "Created Success Contract revision {} for Mission {}",
                            result.success_contract.revision, result.id
                        );
                    }
                }
            },
        },
        Command::Execution {
            command:
                ExecutionCommand::Record {
                    path,
                    mission,
                    contract_revision,
                    actor_kind,
                    status,
                    actor,
                    notes,
                    json,
                },
        } => {
            let result = execution_record(
                path,
                ExecutionInput {
                    mission_id: mission,
                    contract_revision,
                    actor_kind: parse_actor_kind(&actor_kind)?,
                    actor_reference: actor,
                    state: parse_execution_state(&status)?,
                    notes,
                },
            )?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Recorded {} execution {}", status, result.id);
            }
        }
        Command::Execution {
            command:
                ExecutionCommand::List {
                    path,
                    mission,
                    json,
                },
        } => {
            let results = execution_list(path, &mission)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                for result in results {
                    println!(
                        "{}\t{:?}\t{}\trevision {}",
                        result.id,
                        result.state,
                        result.started_at_unix_ms,
                        result.contract_revision
                    );
                }
            }
        }
        Command::Execution {
            command: ExecutionCommand::Show { path, id, json },
        } => {
            let result = execution_get(path, &id)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Execution: {}", result.id);
                println!("Mission: {}", result.mission_id);
                println!(
                    "Contract: {} revision {}",
                    result.contract_id, result.contract_revision
                );
                println!(
                    "Actor: {:?} ({})",
                    result.actor_kind,
                    result.actor_reference.as_deref().unwrap_or("unspecified")
                );
                println!("State: {:?}", result.state);
                println!("Started: {}", result.started_at_unix_ms);
                println!("Ended: {:?}", result.ended_at_unix_ms);
                if let Some(notes) = result.notes {
                    println!("Notes: {notes}");
                }
            }
        }
        Command::Evidence {
            command:
                EvidenceCommand::Add {
                    path,
                    mission,
                    execution,
                    kind,
                    locator,
                    json,
                },
        } => {
            let result = evidence_add(
                path,
                EvidenceInput {
                    mission_id: mission,
                    execution_id: execution,
                    kind: parse_evidence_kind(&kind)?,
                    locator,
                },
            )?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Recorded evidence reference {}", result.id);
            }
        }
        Command::Evidence {
            command:
                EvidenceCommand::List {
                    path,
                    mission,
                    json,
                },
        } => {
            let results = evidence_list(path, &mission)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                for result in results {
                    println!(
                        "{}\t{}\t{:?}\t{}\t{}",
                        result.id,
                        result.execution_id,
                        result.kind,
                        result.captured_at_unix_ms,
                        result.availability
                    );
                }
            }
        }
        Command::Evidence {
            command: EvidenceCommand::Show { path, id, json },
        } => {
            let result = evidence_get(path, &id)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Evidence: {}", result.id);
                println!("Mission: {}", result.mission_id);
                println!("Execution: {}", result.execution_id);
                println!("Kind: {:?}", result.kind);
                println!("Locator: {}", result.locator);
                println!("SHA-256: {:?}", result.sha256);
                println!("Hash status: {:?}", result.hash_status);
                println!("Availability at capture: {}", result.availability);
                println!("Scan ID: {:?}", result.scan_id);
                println!("Captured: {}", result.captured_at_unix_ms);
            }
        }
        Command::Outcome { command } => match command {
            OutcomeCommand::Assess {
                path,
                mission,
                contract_revision,
                criterion,
                result,
                evidence,
                verifier_kind,
                verifier,
                explanation,
                json,
            } => {
                let assessment = outcome_assess(
                    path,
                    CriterionAssessmentInput {
                        mission_id: mission,
                        contract_revision,
                        criterion_id: criterion,
                        result: parse_assessment_result(&result)?,
                        evidence_ids: evidence,
                        verifier_kind: parse_verifier_kind(&verifier_kind)?,
                        verifier_reference: verifier,
                        explanation,
                    },
                )?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&assessment)?);
                } else {
                    println!("Recorded {} assessment {}", result, assessment.id);
                }
            }
            OutcomeCommand::Show {
                path,
                mission,
                contract_revision,
                json,
            } => {
                let outcome = outcome_get(path, &mission, contract_revision)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&outcome)?);
                } else {
                    println!("Mission outcome: {}", outcome.state);
                }
            }
        },
        Command::Install {
            project: _,
            path,
            platform,
        } => {
            for file in integrations::install(&path, platform)? {
                println!("Installed {file}");
            }
        }
        Command::Uninstall {
            project: _,
            path,
            platform,
        } => {
            for file in integrations::uninstall(&path, platform)? {
                println!("Removed {file}");
            }
        }
        Command::Integrations {
            command: IntegrationCommand::Status { path },
        } => {
            for (platform, installed) in integrations::status(&path)? {
                println!(
                    "{platform}: {}",
                    if installed {
                        "installed"
                    } else {
                        "not installed"
                    }
                );
            }
        }
    }
    Ok(())
}
