use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::storage::database_path;
use crate::{CriterionInput, Mission, MissionInput, Result, TramuriError};

const CONFIG_FILE: &str = "config.toml";
const INITIAL_ACTIVITY: &str = "discover";
const PRODUCT_FRAMING_ACTIVITY: &str = "product_framing";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionPerspective {
    ProductEngineer,
    Fde,
    Mixed,
    Other,
}

impl MissionPerspective {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProductEngineer => "product_engineer",
            Self::Fde => "fde",
            Self::Mixed => "mixed",
            Self::Other => "other",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "product_engineer" => Some(Self::ProductEngineer),
            "fde" => Some(Self::Fde),
            "mixed" => Some(Self::Mixed),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkProfilePreference {
    ProductEngineer,
    Fde,
    Mixed,
    Other,
    Auto,
}

impl WorkProfilePreference {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProductEngineer => "product_engineer",
            Self::Fde => "fde",
            Self::Mixed => "mixed",
            Self::Other => "other",
            Self::Auto => "auto",
        }
    }

    fn mission_perspective(self) -> Option<MissionPerspective> {
        match self {
            Self::ProductEngineer => Some(MissionPerspective::ProductEngineer),
            Self::Fde => Some(MissionPerspective::Fde),
            Self::Mixed => Some(MissionPerspective::Mixed),
            Self::Other => Some(MissionPerspective::Other),
            Self::Auto => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AskWorkPerspective {
    Ambiguous,
    Always,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkProfile {
    pub default_work_perspective: WorkProfilePreference,
    pub ask_work_perspective: AskWorkPerspective,
}

impl Default for WorkProfile {
    fn default() -> Self {
        Self {
            default_work_perspective: WorkProfilePreference::Auto,
            ask_work_perspective: AskWorkPerspective::Ambiguous,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentRouteSource {
    Explicit,
    CurrentIntent,
    Profile,
    Inferred,
    Ambiguous,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntentRoute {
    pub perspective: Option<MissionPerspective>,
    pub source: IntentRouteSource,
    pub next_activity: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionWorkflow {
    pub mission_id: String,
    pub perspective: Option<MissionPerspective>,
    pub perspective_source: Option<IntentRouteSource>,
    pub next_activity: Option<String>,
    pub updated_at_unix_ms: Option<i64>,
    pub perspective_history: Vec<WorkPerspectiveChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkPerspectiveChange {
    pub previous_perspective: Option<MissionPerspective>,
    pub perspective: MissionPerspective,
    pub source: IntentRouteSource,
    pub changed_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionStartResult {
    pub mission: Mission,
    pub workflow: MissionWorkflow,
    pub route: IntentRoute,
}

/// Return local workflow defaults. A missing configuration file keeps existing workspaces usable.
pub fn profile_get(path: impl AsRef<Path>) -> Result<WorkProfile> {
    let root = canonical_root(path.as_ref())?;
    let config_path = checked_config_path(&root, false)?;
    read_profile(&config_path)
}

/// Update only Tramuri's workflow preferences while retaining other TOML values.
pub fn profile_set(path: impl AsRef<Path>, profile: WorkProfile) -> Result<WorkProfile> {
    let root = canonical_root(path.as_ref())?;
    require_initialized(&root)?;
    let config_path = checked_config_path(&root, true)?;
    let mut document = read_document(&config_path)?;
    let root_table = document
        .as_table_mut()
        .ok_or_else(|| invalid_config("top-level TOML value must be a table"))?;
    let workflow = root_table
        .entry("workflow")
        .or_insert_with(|| Value::Table(toml::map::Map::new()));
    let workflow = workflow
        .as_table_mut()
        .ok_or_else(|| invalid_config("[workflow] must be a table"))?;
    workflow.insert(
        "default_work_perspective".to_owned(),
        Value::String(profile.default_work_perspective.as_str().to_owned()),
    );
    workflow.insert(
        "ask_work_perspective".to_owned(),
        Value::String(
            match profile.ask_work_perspective {
                AskWorkPerspective::Ambiguous => "ambiguous",
                AskWorkPerspective::Always => "always",
            }
            .to_owned(),
        ),
    );
    let encoded = toml::to_string_pretty(&document).map_err(invalid_config)?;
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&config_path)
        .map_err(|source| TramuriError::Io {
            operation: "write Tramuri workflow configuration",
            path: config_path.clone(),
            source,
        })?;
    file.write_all(encoded.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|source| TramuriError::Io {
            operation: "write Tramuri workflow configuration",
            path: config_path,
            source,
        })?;
    Ok(profile)
}

/// Route a natural-language objective using explicit input, configured preference, then small
/// deterministic phrase sets. This is a suggestion mechanism, not general natural-language
/// understanding; callers should ask when no safe route is available.
pub fn route_intent(
    objective: &str,
    profile: WorkProfile,
    explicit: Option<MissionPerspective>,
) -> IntentRoute {
    let next_activity = INITIAL_ACTIVITY.to_owned();
    if let Some(perspective) = explicit {
        return IntentRoute {
            perspective: Some(perspective),
            source: IntentRouteSource::Explicit,
            next_activity: initial_activity(perspective).to_owned(),
            rationale: "The caller supplied an explicit work perspective.".to_owned(),
        };
    }

    let (product_signal, fde_signal) = intent_signals(objective);
    if product_signal ^ fde_signal {
        let perspective = if fde_signal {
            MissionPerspective::Fde
        } else {
            MissionPerspective::ProductEngineer
        };
        return IntentRoute {
            perspective: Some(perspective),
            source: IntentRouteSource::CurrentIntent,
            next_activity: initial_activity(perspective).to_owned(),
            rationale: if fde_signal {
                "The objective explicitly describes customer integration or deployment work."
            } else {
                "The objective explicitly describes creating or improving a product."
            }
            .to_owned(),
        };
    }

    if let Some(perspective) = profile.default_work_perspective.mission_perspective() {
        return IntentRoute {
            perspective: Some(perspective),
            source: IntentRouteSource::Profile,
            next_activity: initial_activity(perspective).to_owned(),
            rationale: "The configured Work Profile supplies the default perspective.".to_owned(),
        };
    }

    if product_signal && fde_signal {
        return IntentRoute {
            perspective: None,
            source: IntentRouteSource::Ambiguous,
            next_activity,
            rationale: "The objective contains both product and customer-integration signals."
                .to_owned(),
        };
    }

    IntentRoute {
        perspective: None,
        source: IntentRouteSource::Ambiguous,
        next_activity,
        rationale: "The objective does not identify a work perspective confidently; ask one short question or select one explicitly.".to_owned(),
    }
}

fn initial_activity(perspective: MissionPerspective) -> &'static str {
    match perspective {
        MissionPerspective::ProductEngineer => PRODUCT_FRAMING_ACTIVITY,
        MissionPerspective::Fde | MissionPerspective::Mixed | MissionPerspective::Other => {
            INITIAL_ACTIVITY
        }
    }
}

/// Start an exploratory Mission. This operation is explicit and initializes the workspace if needed.
pub fn mission_start(
    path: impl AsRef<Path>,
    objective: &str,
    context: Option<&str>,
    explicit: Option<MissionPerspective>,
) -> Result<MissionStartResult> {
    if objective.trim().is_empty() {
        return Err(TramuriError::EmptyMissionObjective);
    }
    let root = canonical_root(path.as_ref())?;
    let profile = profile_get(&root)?;
    if profile.ask_work_perspective == AskWorkPerspective::Always && explicit.is_none() {
        return Err(TramuriError::AmbiguousWorkPerspective);
    }
    let route = route_intent(objective, profile, explicit);
    let Some(perspective) = route.perspective else {
        return Err(TramuriError::AmbiguousWorkPerspective);
    };
    if !database_path(&root).exists() {
        crate::init(&root)?;
    }
    let mission = crate::mission::mission_create_with_workflow(
        &root,
        MissionInput {
            objective: objective.to_owned(),
            context: context.map(str::to_owned),
            criteria: Vec::<CriterionInput>::new(),
        },
        perspective,
        route.source,
        Some(route.next_activity.clone()),
    )?;
    let workflow = mission_workflow_get(&root, &mission.id)?
        .ok_or_else(|| TramuriError::MissionNotFound(mission.id.clone()))?;
    Ok(MissionStartResult {
        mission,
        workflow,
        route,
    })
}

pub fn mission_workflow_get(
    path: impl AsRef<Path>,
    mission_id: &str,
) -> Result<Option<MissionWorkflow>> {
    let root = canonical_root(path.as_ref())?;
    let connection = crate::storage::open_database(&root)?;
    let workflow = connection
        .query_row(
            "SELECT perspective, perspective_source, next_activity, updated_at_unix_ms
             FROM mission_workflows WHERE mission_id = ?1",
            [mission_id],
            |row| {
                let perspective: Option<String> = row.get(0)?;
                let source: Option<String> = row.get(1)?;
                Ok((perspective, source, row.get(2)?, row.get(3)?))
            },
        )
        .optional()?;
    let Some((perspective, source, next_activity, updated_at_unix_ms)) = workflow else {
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM missions WHERE id = ?1)",
            [mission_id],
            |row| row.get(0),
        )?;
        return if exists {
            Ok(None)
        } else {
            Err(TramuriError::MissionNotFound(mission_id.to_owned()))
        };
    };
    let perspective = perspective
        .as_deref()
        .map(|value| {
            MissionPerspective::parse(value)
                .ok_or_else(|| invalid_config("unknown stored Mission perspective"))
        })
        .transpose()?;
    let source = source.as_deref().map(parse_route_source).transpose()?;
    let mut statement = connection.prepare(
        "SELECT previous_perspective, perspective, source, changed_at_unix_ms
         FROM mission_work_perspective_history WHERE mission_id = ?1
         ORDER BY changed_at_unix_ms, id",
    )?;
    let history = statement
        .query_map([mission_id], |row| {
            let previous: Option<String> = row.get(0)?;
            let current: String = row.get(1)?;
            let source: String = row.get(2)?;
            Ok((previous, current, source, row.get::<_, i64>(3)?))
        })?
        .map(|row| {
            let (previous, current, source, changed_at_unix_ms) = row?;
            Ok(WorkPerspectiveChange {
                previous_perspective: previous
                    .as_deref()
                    .map(|value| {
                        MissionPerspective::parse(value).ok_or_else(|| {
                            invalid_config("unknown stored previous Mission perspective")
                        })
                    })
                    .transpose()?,
                perspective: MissionPerspective::parse(&current)
                    .ok_or_else(|| invalid_config("unknown stored Mission perspective"))?,
                source: parse_route_source(&source)?,
                changed_at_unix_ms,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(MissionWorkflow {
        mission_id: mission_id.to_owned(),
        perspective,
        perspective_source: source,
        next_activity,
        updated_at_unix_ms,
        perspective_history: history,
    }))
}

pub fn mission_workflow_set(
    path: impl AsRef<Path>,
    mission_id: &str,
    perspective: MissionPerspective,
    source: IntentRouteSource,
    next_activity: Option<&str>,
) -> Result<MissionWorkflow> {
    if source == IntentRouteSource::Ambiguous {
        return Err(TramuriError::AmbiguousWorkPerspective);
    }
    let activity = next_activity
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if activity.is_some_and(|value| value.len() > 80 || value.chars().any(char::is_control)) {
        return Err(TramuriError::InvalidReadiness(
            "next activity must be at most 80 characters without control characters".to_owned(),
        ));
    }
    let root = canonical_root(path.as_ref())?;
    let mut connection = crate::storage::open_database(&root)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let exists: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM missions WHERE id = ?1)",
        [mission_id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(TramuriError::MissionNotFound(mission_id.to_owned()));
    }
    let previous: Option<String> = transaction
        .query_row(
            "SELECT perspective FROM mission_workflows WHERE mission_id = ?1",
            [mission_id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    let now = now_unix_ms()?;
    transaction.execute(
        "INSERT INTO mission_workflows (mission_id, perspective, perspective_source, next_activity, updated_at_unix_ms)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(mission_id) DO UPDATE SET perspective = excluded.perspective,
            perspective_source = excluded.perspective_source, next_activity = excluded.next_activity,
            updated_at_unix_ms = excluded.updated_at_unix_ms",
        params![mission_id, perspective.as_str(), route_source_str(source), activity, now],
    )?;
    if previous.as_deref() != Some(perspective.as_str()) {
        transaction.execute(
            "INSERT INTO mission_work_perspective_history
             (id, mission_id, previous_perspective, perspective, source, changed_at_unix_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                uuid::Uuid::new_v4().to_string(),
                mission_id,
                previous,
                perspective.as_str(),
                route_source_str(source),
                now
            ],
        )?;
    }
    transaction.commit()?;
    mission_workflow_get(&root, mission_id)?
        .ok_or_else(|| TramuriError::MissionNotFound(mission_id.to_owned()))
}

/// Update only a Mission's current focus while preserving its selected perspective and provenance.
pub fn mission_workflow_activity_set(
    path: impl AsRef<Path>,
    mission_id: &str,
    next_activity: Option<&str>,
) -> Result<MissionWorkflow> {
    let activity = next_activity
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if activity.is_some_and(|value| value.len() > 80 || value.chars().any(char::is_control)) {
        return Err(TramuriError::InvalidReadiness(
            "next activity must be at most 80 characters without control characters".to_owned(),
        ));
    }
    let root = canonical_root(path.as_ref())?;
    let mut connection = crate::storage::open_database(&root)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let exists: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM missions WHERE id = ?1)",
        [mission_id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(TramuriError::MissionNotFound(mission_id.to_owned()));
    }
    transaction.execute(
        "INSERT INTO mission_workflows (mission_id, next_activity, updated_at_unix_ms)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(mission_id) DO UPDATE SET next_activity = excluded.next_activity,
            updated_at_unix_ms = excluded.updated_at_unix_ms",
        params![mission_id, activity, now_unix_ms()?],
    )?;
    transaction.commit()?;
    mission_workflow_get(&root, mission_id)?
        .ok_or_else(|| TramuriError::MissionNotFound(mission_id.to_owned()))
}

fn route_source_str(source: IntentRouteSource) -> &'static str {
    match source {
        IntentRouteSource::Explicit => "explicit",
        IntentRouteSource::CurrentIntent => "current_intent",
        IntentRouteSource::Profile => "profile",
        IntentRouteSource::Inferred => "inferred",
        IntentRouteSource::Ambiguous => "ambiguous",
    }
}

fn parse_route_source(value: &str) -> Result<IntentRouteSource> {
    match value {
        "explicit" => Ok(IntentRouteSource::Explicit),
        "current_intent" => Ok(IntentRouteSource::CurrentIntent),
        "profile" => Ok(IntentRouteSource::Profile),
        "inferred" => Ok(IntentRouteSource::Inferred),
        _ => Err(invalid_config("unknown stored Mission perspective source")),
    }
}

fn intent_signals(objective: &str) -> (bool, bool) {
    let value = objective.to_lowercase();
    let product = [
        "criar um produto",
        "criar produto",
        "criar um saas",
        "criar saas",
        "desenvolver um saas",
        "plataforma de cursos",
        "venda de cursos online",
        "vender cursos online",
        "novo produto",
        "desenvolver um produto",
        "build a product",
        "create a product",
        "build a saas",
        "create a saas",
        "online course platform",
        "sell courses online",
        "new product",
        "product idea",
        "aplicativo para usuários",
        "app for users",
        "product engineering",
    ]
    .iter()
    .any(|phrase| value.contains(phrase));
    let fde = [
        "integração com cliente",
        "integracao com cliente",
        "integrar a api do cliente",
        "integração de cliente",
        "integracao de cliente",
        "ambiente do cliente",
        "customer integration",
        "client integration",
        "customer deployment",
        "implantação para cliente",
        "implantacao para cliente",
    ]
    .iter()
    .any(|phrase| value.contains(phrase));
    (product, fde)
}

fn canonical_root(path: &Path) -> Result<PathBuf> {
    std::fs::canonicalize(path).map_err(|source| TramuriError::Io {
        operation: "resolve project directory",
        path: path.to_path_buf(),
        source,
    })
}

fn require_initialized(root: &Path) -> Result<()> {
    let database = database_path(root);
    match fs::symlink_metadata(&database) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(TramuriError::InvalidStatePath(database))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(TramuriError::NotInitialized(database))
        }
        Err(source) => Err(TramuriError::Io {
            operation: "inspect Tramuri database",
            path: database,
            source,
        }),
    }
}

fn checked_config_path(root: &Path, create_directory: bool) -> Result<PathBuf> {
    let state_dir = root.join(".tramuri");
    match fs::symlink_metadata(&state_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(TramuriError::InvalidStatePath(state_dir));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !create_directory => {
            return Ok(state_dir.join(CONFIG_FILE));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&state_dir).map_err(|source| TramuriError::Io {
                operation: "create Tramuri state directory",
                path: state_dir.clone(),
                source,
            })?;
        }
        Err(source) => {
            return Err(TramuriError::Io {
                operation: "inspect Tramuri state directory",
                path: state_dir,
                source,
            });
        }
    }
    let config_path = state_dir.join(CONFIG_FILE);
    match fs::symlink_metadata(&config_path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(TramuriError::InvalidStatePath(config_path))
        }
        Ok(_) => Ok(config_path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(config_path),
        Err(source) => Err(TramuriError::Io {
            operation: "inspect Tramuri workflow configuration",
            path: config_path,
            source,
        }),
    }
}

fn read_document(path: &Path) -> Result<Value> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Value::Table(Default::default()));
        }
        Err(source) => {
            return Err(TramuriError::Io {
                operation: "read Tramuri workflow configuration",
                path: path.to_path_buf(),
                source,
            });
        }
    };
    if bytes.len() > 64 * 1024 {
        return Err(invalid_config(
            "file exceeds the 64 KiB configuration limit",
        ));
    }
    let text = std::str::from_utf8(&bytes).map_err(invalid_config)?;
    toml::from_str(text).map_err(invalid_config)
}

fn read_profile(path: &Path) -> Result<WorkProfile> {
    let document = read_document(path)?;
    let Some(root) = document.as_table() else {
        return Err(invalid_config("top-level TOML value must be a table"));
    };
    let Some(workflow) = root.get("workflow") else {
        return Ok(WorkProfile::default());
    };
    let workflow = workflow
        .as_table()
        .ok_or_else(|| invalid_config("[workflow] must be a table"))?;
    let preference = workflow
        .get("default_work_perspective")
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| invalid_config("default_work_perspective must be a string"))
        })
        .transpose()?
        .map(parse_preference)
        .transpose()?
        .unwrap_or(WorkProfilePreference::Auto);
    let ask = workflow
        .get("ask_work_perspective")
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| invalid_config("ask_work_perspective must be a string"))
        })
        .transpose()?
        .map(parse_ask)
        .transpose()?
        .unwrap_or(AskWorkPerspective::Ambiguous);
    Ok(WorkProfile {
        default_work_perspective: preference,
        ask_work_perspective: ask,
    })
}

fn parse_preference(value: &str) -> Result<WorkProfilePreference> {
    match value {
        "product_engineer" => Ok(WorkProfilePreference::ProductEngineer),
        "fde" => Ok(WorkProfilePreference::Fde),
        "mixed" => Ok(WorkProfilePreference::Mixed),
        "other" => Ok(WorkProfilePreference::Other),
        "auto" => Ok(WorkProfilePreference::Auto),
        _ => Err(invalid_config(
            "default_work_perspective must be product_engineer, fde, mixed, other, or auto",
        )),
    }
}

fn parse_ask(value: &str) -> Result<AskWorkPerspective> {
    match value {
        "ambiguous" => Ok(AskWorkPerspective::Ambiguous),
        "always" => Ok(AskWorkPerspective::Always),
        _ => Err(invalid_config(
            "ask_work_perspective must be ambiguous or always",
        )),
    }
}

fn invalid_config(error: impl std::fmt::Display) -> TramuriError {
    TramuriError::InvalidWorkProfileConfig(error.to_string())
}

fn now_unix_ms() -> Result<i64> {
    let value = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TramuriError::Clock)?
        .as_millis();
    i64::try_from(value).map_err(|_| TramuriError::Clock)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace() -> tempfile::TempDir {
        tempfile::tempdir().expect("temporary workspace")
    }

    #[test]
    fn profile_persists_preferences_and_preserves_other_settings() {
        let project = workspace();
        crate::init(project.path()).unwrap();
        let config = project.path().join(".tramuri/config.toml");
        fs::write(&config, "[other]\nvalue = 7\n").unwrap();
        let profile = WorkProfile {
            default_work_perspective: WorkProfilePreference::ProductEngineer,
            ask_work_perspective: AskWorkPerspective::Always,
        };
        assert_eq!(profile_set(project.path(), profile).unwrap(), profile);
        assert_eq!(profile_get(project.path()).unwrap(), profile);
        let text = fs::read_to_string(config).unwrap();
        assert!(text.contains("[other]"));
        assert!(text.contains("value = 7"));
    }

    #[test]
    fn route_uses_explicit_current_intent_then_profile_and_reports_ambiguity() {
        let automatic = WorkProfile::default();
        let course_saas = route_intent(
            "Quero criar um SaaS de venda de cursos online para alunos",
            automatic,
            None,
        );
        assert_eq!(
            course_saas.perspective,
            Some(MissionPerspective::ProductEngineer)
        );
        assert_eq!(course_saas.source, IntentRouteSource::CurrentIntent);
        assert_eq!(course_saas.next_activity, "product_framing");
        assert_eq!(
            route_intent("Quero criar um produto novo", automatic, None).perspective,
            Some(MissionPerspective::ProductEngineer)
        );
        assert_eq!(
            route_intent("Integrar a API no ambiente do cliente", automatic, None).perspective,
            Some(MissionPerspective::Fde)
        );
        assert_eq!(
            route_intent("Quero criar uma solução", automatic, None).perspective,
            None
        );
        let profile = WorkProfile {
            default_work_perspective: WorkProfilePreference::ProductEngineer,
            ..automatic
        };
        let routed = route_intent("Preciso integrar a API do cliente", profile, None);
        assert_eq!(routed.perspective, Some(MissionPerspective::Fde));
        let routed = route_intent(
            "Quero criar uma solução",
            profile,
            Some(MissionPerspective::Fde),
        );
        assert_eq!(routed.perspective, Some(MissionPerspective::Fde));
        assert_eq!(routed.source, IntentRouteSource::Explicit);
    }

    #[test]
    fn start_creates_exploratory_mission_with_planned_activity_not_readiness() {
        let project = workspace();
        let started = mission_start(
            project.path(),
            "Quero criar um SaaS de venda de cursos online para alunos",
            None,
            None,
        )
        .unwrap();
        assert!(started.mission.success_contract.criteria.is_empty());
        assert_eq!(
            started.workflow.perspective,
            Some(MissionPerspective::ProductEngineer)
        );
        assert_eq!(
            started.workflow.next_activity.as_deref(),
            Some("product_framing")
        );
        let readiness = crate::readiness_get(project.path(), &started.mission.id).unwrap();
        assert_eq!(readiness.state, crate::ReadinessState::NotAssessed);
        assert_eq!(
            mission_workflow_get(project.path(), &started.mission.id).unwrap(),
            Some(started.workflow)
        );
    }

    #[test]
    fn activity_update_preserves_work_perspective_and_source() {
        let project = workspace();
        let started = mission_start(
            project.path(),
            "Quero criar um SaaS para vender cursos online",
            None,
            None,
        )
        .unwrap();
        let updated =
            mission_workflow_activity_set(project.path(), &started.mission.id, Some("discover"))
                .unwrap();
        assert_eq!(updated.next_activity.as_deref(), Some("discover"));
        assert_eq!(
            updated.perspective,
            Some(MissionPerspective::ProductEngineer)
        );
        assert_eq!(
            updated.perspective_source,
            Some(IntentRouteSource::CurrentIntent)
        );
        assert_eq!(updated.perspective_history.len(), 1);
    }

    #[test]
    fn activity_update_can_add_focus_to_legacy_mission_without_inventing_perspective() {
        let project = workspace();
        crate::init(project.path()).unwrap();
        let mission = crate::mission_create(
            project.path(),
            MissionInput {
                objective: "Resume an older product Mission".to_owned(),
                context: None,
                criteria: Vec::new(),
            },
        )
        .unwrap();
        let updated =
            mission_workflow_activity_set(project.path(), &mission.id, Some("product_framing"))
                .unwrap();
        assert_eq!(updated.next_activity.as_deref(), Some("product_framing"));
        assert_eq!(updated.perspective, None);
        assert_eq!(updated.perspective_source, None);
        assert!(updated.perspective_history.is_empty());
    }

    #[test]
    fn changing_planned_activity_preserves_but_stales_a_different_readiness_assessment() {
        let project = workspace();
        let started = mission_start(project.path(), "Criar um produto", None, None).unwrap();
        crate::readiness_assess(
            project.path(),
            crate::ReadinessAssessmentInput {
                mission_id: started.mission.id.clone(),
                activity: "discover".to_owned(),
                decision: crate::ReadinessDecision::Ready,
                criteria_applied: vec!["Exploration goal is clear".to_owned()],
                gaps: Vec::new(),
                blockers: Vec::new(),
                accepted_uncertainties: Vec::new(),
                actor_reference: None,
            },
        )
        .unwrap();
        let changed = mission_workflow_set(
            project.path(),
            &started.mission.id,
            MissionPerspective::Mixed,
            IntentRouteSource::Explicit,
            Some("prototype"),
        )
        .unwrap();
        assert_eq!(changed.perspective_history.len(), 2);
        assert_eq!(
            changed.perspective_history[1].previous_perspective,
            Some(MissionPerspective::ProductEngineer)
        );
        let snapshot = crate::status_snapshot(project.path()).unwrap();
        let mission = &snapshot.missions[0];
        assert_eq!(
            mission.workflow.as_ref().unwrap().next_activity.as_deref(),
            Some("prototype")
        );
        assert_eq!(mission.readiness.state, crate::ReadinessState::Stale);
        assert_eq!(
            mission.readiness.assessment.as_ref().unwrap().activity,
            "discover"
        );
    }
}
