use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::discovery::{ScanReport, canonical_project_root};
use crate::{HashStatus, Result, TramuriError};

pub const CAPABILITY_SCHEMA: &str = "tramuri.capabilities.v1";
const ADAPTER_VERSION: &str = "1";
const MAX_CAPABILITY_FILE_BYTES: u64 = 1024 * 1024;
const MAX_DESCRIPTION_CHARS: usize = 512;
const ARTIFACT_NAME: &str = "latest-capabilities.json";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityKind {
    Agent,
    Skill,
    Rule,
    Hook,
    McpServer,
}

impl CapabilityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Skill => "skill",
            Self::Rule => "rule",
            Self::Hook => "hook",
            Self::McpServer => "mcp_server",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    DeclaredAndRecognized,
    Invalid,
    Disabled,
    NotVerified,
    NotRecognized,
    UnavailableReadError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityRelationship {
    pub relation: String,
    pub target_id: String,
    pub provenance_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: CapabilityKind,
    pub ecosystem: String,
    pub path: String,
    pub source_reference: String,
    pub sha256: Option<String>,
    pub adapter_version: String,
    pub state: CapabilityState,
    /// `not_verified` means a declaration was read but the resource was not executed or contacted.
    pub availability: String,
    pub observed_at_unix_ms: i64,
    pub dependencies: Vec<String>,
    pub relationships: Vec<CapabilityRelationship>,
    pub limitation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilitySnapshot {
    pub schema: String,
    pub observed_at_unix_ms: i64,
    pub scan_complete: bool,
    pub capabilities: Vec<Capability>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CapabilitySummary {
    pub scan_complete: bool,
    pub total: usize,
    pub agents: usize,
    pub skills: usize,
    pub rules: usize,
    pub hooks: usize,
    pub mcp_servers: usize,
}

pub(crate) fn capability_summary(root: &Path) -> Result<CapabilitySummary> {
    let artifact = artifact_path(root);
    let snapshot = read_artifact(&artifact).map_err(|source| TramuriError::Io {
        operation: "read Tramuri capability registry",
        path: artifact,
        source,
    })?;
    let Some(snapshot) = snapshot else {
        return Ok(CapabilitySummary::default());
    };
    let mut summary = CapabilitySummary {
        scan_complete: snapshot.scan_complete,
        total: snapshot.capabilities.len(),
        ..CapabilitySummary::default()
    };
    for capability in snapshot.capabilities {
        match capability.category {
            CapabilityKind::Agent => summary.agents += 1,
            CapabilityKind::Skill => summary.skills += 1,
            CapabilityKind::Rule => summary.rules += 1,
            CapabilityKind::Hook => summary.hooks += 1,
            CapabilityKind::McpServer => summary.mcp_servers += 1,
        }
    }
    Ok(summary)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Adapter {
    Markdown(CapabilityKind, &'static str),
    Hooks(&'static str),
    Mcp(&'static str),
}

#[derive(Debug)]
struct ReadSource {
    bytes: Vec<u8>,
    hash: String,
}

/// Returns the latest declarative capability snapshot produced by `scan`.
/// Opening the workspace database may perform a supported migration, as other Core reads do.
pub fn capability_snapshot(path: impl AsRef<Path>) -> Result<CapabilitySnapshot> {
    let root = canonical_project_root(path.as_ref())?;
    crate::storage::open_database(&root)?;
    let artifact = artifact_path(&root);
    match read_artifact(&artifact) {
        Ok(Some(snapshot)) => Ok(snapshot),
        Ok(None) => Ok(CapabilitySnapshot {
            schema: CAPABILITY_SCHEMA.to_owned(),
            observed_at_unix_ms: now_unix_ms()?,
            scan_complete: false,
            capabilities: Vec::new(),
            warnings: vec!["No capability scan is recorded; run `tramuri scan`.".to_owned()],
        }),
        Err(source) => Err(TramuriError::Io {
            operation: "read Tramuri capability registry",
            path: artifact,
            source,
        }),
    }
}

/// Return the latest capability snapshot, optionally filtered by its declared category.
pub fn capability_list(
    path: impl AsRef<Path>,
    kind: Option<CapabilityKind>,
) -> Result<CapabilitySnapshot> {
    let mut snapshot = capability_snapshot(path)?;
    if let Some(kind) = kind {
        snapshot.capabilities.retain(|item| item.category == kind);
    }
    Ok(snapshot)
}

pub(crate) fn update_capability_artifact(root: &Path, scan: &ScanReport) -> Result<()> {
    let observed_at_unix_ms = now_unix_ms()?;
    let artifact = artifact_path(root);
    let previous = read_artifact(&artifact).ok().flatten();
    let mut cached_by_source: BTreeMap<String, Vec<Capability>> = BTreeMap::new();
    if let Some(previous) = &previous {
        for capability in &previous.capabilities {
            cached_by_source
                .entry(capability.path.clone())
                .or_default()
                .push(capability.clone());
        }
    }

    let mut current_paths = BTreeSet::new();
    let mut capabilities = Vec::new();
    let mut warnings = Vec::new();
    for file in &scan.files {
        let adapters = adapters_for_path(&file.path);
        if adapters.is_empty() {
            continue;
        }
        current_paths.insert(file.path.clone());
        if file.hash_status != HashStatus::Hashed {
            capabilities.extend(adapters.iter().map(|adapter| {
                unavailable_capability(
                    adapter,
                    &file.path,
                    file.sha256.as_deref(),
                    observed_at_unix_ms,
                    if file.hash_status == HashStatus::TooLarge {
                        "Configuration exceeds the 1 MiB parser limit; content was not read."
                    } else {
                        "The scanner could not verify the source file contents."
                    },
                )
            }));
            continue;
        }

        if let Some(cached) = cached_by_source.get(&file.path)
            && cached.iter().all(|item| item.sha256 == file.sha256)
            && cached
                .iter()
                .all(|item| item.adapter_version == ADAPTER_VERSION)
        {
            capabilities.extend(cached.iter().cloned());
            continue;
        }

        let source = match read_source(root, &file.path) {
            Ok(source) => source,
            Err(message) => {
                warnings.push(format!("{}: {message}", file.path));
                capabilities.extend(adapters.iter().map(|adapter| {
                    unavailable_capability(
                        adapter,
                        &file.path,
                        file.sha256.as_deref(),
                        observed_at_unix_ms,
                        &message,
                    )
                }));
                continue;
            }
        };
        // The scanner fingerprint and the bytes parsed must match; do not persist a raced read.
        if file.sha256.as_deref() != Some(source.hash.as_str()) {
            warnings.push(format!(
                "{} changed after scan; capability result is unverified.",
                file.path
            ));
            capabilities.extend(adapters.iter().map(|adapter| {
                unavailable_capability(
                    adapter,
                    &file.path,
                    Some(&source.hash),
                    observed_at_unix_ms,
                    "The source changed after the inventory scan; run `tramuri scan` again.",
                )
            }));
            continue;
        }
        for adapter in adapters {
            capabilities.extend(parse_adapter(
                adapter,
                &file.path,
                &source,
                observed_at_unix_ms,
                &mut warnings,
            ));
        }
    }

    if !scan.complete
        && let Some(previous) = &previous
    {
        for capability in &previous.capabilities {
            if !current_paths.contains(&capability.path)
                && !capabilities
                    .iter()
                    .any(|current| current.id == capability.id)
            {
                // An incomplete walk cannot prove that an unseen capability was removed.
                capabilities.push(capability.clone());
            }
        }
        warnings.push(
            "The scan was incomplete; unseen previous capabilities were retained.".to_owned(),
        );
    }
    resolve_declared_relationships(&mut capabilities);
    capabilities.sort_by(|a, b| {
        (&a.category, &a.ecosystem, &a.path, &a.name, &a.id).cmp(&(
            &b.category,
            &b.ecosystem,
            &b.path,
            &b.name,
            &b.id,
        ))
    });
    warnings.sort();
    warnings.dedup();
    let snapshot = CapabilitySnapshot {
        schema: CAPABILITY_SCHEMA.to_owned(),
        observed_at_unix_ms,
        scan_complete: scan.complete,
        capabilities,
        warnings,
    };
    if previous.as_ref().is_some_and(|old| {
        old.scan_complete == snapshot.scan_complete
            && old.capabilities == snapshot.capabilities
            && old.warnings == snapshot.warnings
    }) {
        return Ok(());
    }
    write_artifact(root, &snapshot)
}

fn adapters_for_path(path: &str) -> Vec<Adapter> {
    let normalized = path.replace('\\', "/");
    let components: Vec<_> = normalized.split('/').collect();
    let basename = components.last().copied().unwrap_or_default();
    let mut adapters = Vec::new();

    if basename == "AGENTS.md" {
        adapters.push(Adapter::Markdown(CapabilityKind::Rule, "agent_skills"));
    }
    if basename == "CLAUDE.md" {
        adapters.push(Adapter::Markdown(CapabilityKind::Rule, "claude_code"));
    }
    if basename == "GEMINI.md" {
        adapters.push(Adapter::Markdown(CapabilityKind::Rule, "gemini_cli"));
    }
    if normalized == ".github/copilot-instructions.md"
        || (normalized.starts_with(".github/instructions/")
            && basename.ends_with(".instructions.md"))
    {
        adapters.push(Adapter::Markdown(CapabilityKind::Rule, "github_copilot"));
    }
    if is_under_directory(&components, &[".cursor", "rules"]) && basename.ends_with(".mdc") {
        adapters.push(Adapter::Markdown(CapabilityKind::Rule, "cursor"));
    }

    for (directory, ecosystem) in [
        (&[".agents", "skills"][..], "agent_skills"),
        (&[".claude", "skills"][..], "claude_code"),
        (&[".cursor", "skills"][..], "cursor"),
        (&[".gemini", "skills"][..], "gemini_cli"),
        (&[".github", "skills"][..], "github_copilot"),
    ] {
        if is_under_directory(&components, directory) && basename == "SKILL.md" {
            adapters.push(Adapter::Markdown(CapabilityKind::Skill, ecosystem));
        }
    }
    if is_under_directory(&components, &[".claude", "agents"]) && basename.ends_with(".md") {
        adapters.push(Adapter::Markdown(CapabilityKind::Agent, "claude_code"));
    }
    if is_under_directory(&components, &[".gemini", "agents"]) && basename.ends_with(".md") {
        adapters.push(Adapter::Markdown(CapabilityKind::Agent, "gemini_cli"));
    }
    if is_under_directory(&components, &[".github", "agents"]) && basename.ends_with(".md") {
        adapters.push(Adapter::Markdown(CapabilityKind::Agent, "github_copilot"));
    }

    match normalized.as_str() {
        ".codex/hooks.json" => adapters.push(Adapter::Hooks("openai_codex")),
        ".claude/settings.json" => {
            adapters.push(Adapter::Hooks("claude_code"));
            adapters.push(Adapter::Mcp("claude_code"));
        }
        ".mcp.json" => adapters.push(Adapter::Mcp("claude_code")),
        ".cursor/hooks.json" => adapters.push(Adapter::Hooks("cursor")),
        ".cursor/mcp.json" => adapters.push(Adapter::Mcp("cursor")),
        ".gemini/settings.json" => {
            adapters.push(Adapter::Hooks("gemini_cli"));
            adapters.push(Adapter::Mcp("gemini_cli"));
        }
        ".vscode/mcp.json" => adapters.push(Adapter::Mcp("github_copilot")),
        _ => {}
    }
    if is_under_directory(&components, &[".github", "hooks"]) && basename.ends_with(".json") {
        adapters.push(Adapter::Hooks("github_copilot"));
    }
    adapters
}

fn is_under_directory(components: &[&str], directory: &[&str]) -> bool {
    components.len() > directory.len() && components.starts_with(directory)
}

fn parse_adapter(
    adapter: Adapter,
    path: &str,
    source: &ReadSource,
    observed_at_unix_ms: i64,
    warnings: &mut Vec<String>,
) -> Vec<Capability> {
    match adapter {
        Adapter::Markdown(category, ecosystem) => parse_markdown(
            category,
            ecosystem,
            path,
            source,
            observed_at_unix_ms,
            warnings,
        ),
        Adapter::Hooks(ecosystem) => {
            parse_hooks(ecosystem, path, source, observed_at_unix_ms, warnings)
        }
        Adapter::Mcp(ecosystem) => {
            parse_mcp(ecosystem, path, source, observed_at_unix_ms, warnings)
        }
    }
}

fn parse_markdown(
    category: CapabilityKind,
    ecosystem: &str,
    path: &str,
    source: &ReadSource,
    observed_at_unix_ms: i64,
    warnings: &mut Vec<String>,
) -> Vec<Capability> {
    let text = match std::str::from_utf8(&source.bytes) {
        Ok(text) => text,
        Err(_) => {
            return vec![invalid_capability(
                category,
                ecosystem,
                path,
                &source.hash,
                observed_at_unix_ms,
                "File is not valid UTF-8.",
            )];
        }
    };
    let metadata =
        if category == CapabilityKind::Rule && ecosystem != "cursor" && !text.starts_with("---") {
            BTreeMap::new()
        } else {
            match parse_frontmatter(text) {
                Ok(metadata) => metadata,
                Err(message) => {
                    return vec![invalid_capability(
                        category,
                        ecosystem,
                        path,
                        &source.hash,
                        observed_at_unix_ms,
                        message,
                    )];
                }
            }
        };
    let fallback = Path::new(path)
        .parent()
        .and_then(Path::file_name)
        .or_else(|| Path::new(path).file_stem())
        .and_then(|value| value.to_str())
        .unwrap_or("unnamed")
        .to_owned();
    let name = metadata.get("name").cloned().unwrap_or(fallback);
    let description = metadata.get("description").and_then(|value| {
        let value = truncate(value, MAX_DESCRIPTION_CHARS);
        (!looks_sensitive_metadata(&value)).then_some(value)
    });
    let state = if metadata
        .get("disabled")
        .is_some_and(|value| value == "true")
    {
        CapabilityState::Disabled
    } else {
        CapabilityState::DeclaredAndRecognized
    };
    let dependencies = if category == CapabilityKind::Agent && ecosystem == "claude_code" {
        metadata
            .get("mcpServers")
            .or_else(|| metadata.get("mcp_servers"))
            .map(|value| parse_declared_list(value))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    if description
        .as_ref()
        .is_some_and(|value| value.len() >= MAX_DESCRIPTION_CHARS)
    {
        warnings.push(format!(
            "{path}: description was truncated to {MAX_DESCRIPTION_CHARS} characters."
        ));
    }
    vec![make_capability(
        category,
        ecosystem,
        path,
        &name,
        description,
        &source.hash,
        observed_at_unix_ms,
        state,
        "not_verified",
        dependencies,
        None,
        &name,
    )]
}

fn parse_frontmatter(text: &str) -> std::result::Result<BTreeMap<String, String>, &'static str> {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Err("Expected YAML front matter beginning with `---`.");
    }
    let mut values: BTreeMap<String, String> = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut block = false;
    let mut closed = false;
    for line in lines {
        if line.trim() == "---" {
            if let Some(key) = current.take()
                && let Some(value) = values.get_mut(&key)
            {
                *value = value.trim().to_owned();
            }
            closed = true;
            break;
        }
        if (line.starts_with(' ') || line.starts_with('\t')) && block {
            if let Some(key) = &current {
                let value = values.entry(key.clone()).or_default();
                if !value.is_empty() {
                    value.push(
                        if matches!(key.as_str(), "mcpServers" | "mcp_servers")
                            && line.trim_start().starts_with("- ")
                        {
                            ','
                        } else {
                            ' '
                        },
                    );
                }
                value.push_str(if matches!(key.as_str(), "mcpServers" | "mcp_servers") {
                    line.trim().trim_start_matches("- ")
                } else {
                    line.trim()
                });
            }
            continue;
        }
        block = false;
        let Some((key, raw)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        if !matches!(
            key,
            "name" | "description" | "disabled" | "dependencies" | "mcpServers" | "mcp_servers"
        ) {
            current = None;
            continue;
        }
        let raw = raw.trim();
        if raw.is_empty() && matches!(key, "mcpServers" | "mcp_servers") {
            values.insert(key.to_owned(), String::new());
            current = Some(key.to_owned());
            block = true;
            continue;
        }
        if matches!(raw, ">" | "|" | ">-" | "|-" | ">+" | "|+") {
            values.insert(key.to_owned(), String::new());
            current = Some(key.to_owned());
            block = true;
        } else {
            values.insert(key.to_owned(), unquote_scalar(raw));
            current = None;
        }
    }
    if !closed {
        return Err("YAML front matter has no closing `---`.");
    }
    Ok(values)
}

fn parse_declared_list(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(unquote_scalar)
        .filter(|name| !name.is_empty())
        .collect()
}

fn resolve_declared_relationships(capabilities: &mut [Capability]) {
    let mut server_ids: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for item in capabilities
        .iter()
        .filter(|item| item.category == CapabilityKind::McpServer)
    {
        server_ids
            .entry(item.name.clone())
            .or_default()
            .push(item.id.clone());
    }
    for item in capabilities {
        item.relationships
            .retain(|relationship| relationship.relation != "uses_mcp_server");
        if item.category != CapabilityKind::Agent || item.ecosystem != "claude_code" {
            continue;
        }
        for dependency in &item.dependencies {
            if let Some(ids) = server_ids.get(dependency)
                && ids.len() == 1
            {
                item.relationships.push(CapabilityRelationship {
                    relation: "uses_mcp_server".to_owned(),
                    target_id: ids[0].clone(),
                    provenance_path: item.path.clone(),
                });
            }
        }
        item.relationships.sort_by(|left, right| {
            (&left.relation, &left.target_id).cmp(&(&right.relation, &right.target_id))
        });
    }
}

fn unquote_scalar(value: &str) -> String {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return serde_json::from_str(value)
            .unwrap_or_else(|_| value[1..value.len() - 1].to_owned());
    }
    if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
        return value[1..value.len() - 1].replace("''", "'");
    }
    value.trim().trim_matches(['"', '\'']).to_owned()
}

fn parse_hooks(
    ecosystem: &str,
    path: &str,
    source: &ReadSource,
    observed_at_unix_ms: i64,
    warnings: &mut Vec<String>,
) -> Vec<Capability> {
    let value: Value = match serde_json::from_slice::<Value>(&source.bytes) {
        Ok(value) if value.is_object() => value,
        Ok(_) => {
            return vec![invalid_capability(
                CapabilityKind::Hook,
                ecosystem,
                path,
                &source.hash,
                observed_at_unix_ms,
                "Hook configuration must be a JSON object.",
            )];
        }
        Err(error) => {
            return vec![invalid_capability(
                CapabilityKind::Hook,
                ecosystem,
                path,
                &source.hash,
                observed_at_unix_ms,
                &format!("Invalid JSON: {}", safe_json_error(&error.to_string())),
            )];
        }
    };
    let Some(hooks) = value.get("hooks").and_then(Value::as_object) else {
        return vec![unrecognized_capability(
            CapabilityKind::Hook,
            ecosystem,
            path,
            &source.hash,
            observed_at_unix_ms,
            "No recognized `hooks` object was declared.",
        )];
    };
    let disabled = value
        .pointer("/hooksConfig/enabled")
        .and_then(Value::as_bool)
        == Some(false);
    let mut output = Vec::new();
    for (event, entries) in hooks {
        let is_valid = entries.is_array();
        output.push(make_capability(
            CapabilityKind::Hook,
            ecosystem,
            path,
            event,
            None,
            &source.hash,
            observed_at_unix_ms,
            if !is_valid {
                CapabilityState::Invalid
            } else if disabled {
                CapabilityState::Disabled
            } else {
                CapabilityState::DeclaredAndRecognized
            },
            "not_verified",
            Vec::new(),
            if !is_valid {
                Some("Hook event value must be an array.".to_owned())
            } else {
                None
            },
            event,
        ));
    }
    if output.is_empty() {
        warnings.push(format!(
            "{path}: hook configuration contains no declared events."
        ));
        output.push(unrecognized_capability(
            CapabilityKind::Hook,
            ecosystem,
            path,
            &source.hash,
            observed_at_unix_ms,
            "No hook events were declared.",
        ));
    }
    output
}

fn parse_mcp(
    ecosystem: &str,
    path: &str,
    source: &ReadSource,
    observed_at_unix_ms: i64,
    _warnings: &mut Vec<String>,
) -> Vec<Capability> {
    let value: Value = match serde_json::from_slice::<Value>(&source.bytes) {
        Ok(value) if value.is_object() => value,
        Ok(_) => {
            return vec![invalid_capability(
                CapabilityKind::McpServer,
                ecosystem,
                path,
                &source.hash,
                observed_at_unix_ms,
                "MCP configuration must be a JSON object.",
            )];
        }
        Err(error) => {
            return vec![invalid_capability(
                CapabilityKind::McpServer,
                ecosystem,
                path,
                &source.hash,
                observed_at_unix_ms,
                &format!("Invalid JSON: {}", safe_json_error(&error.to_string())),
            )];
        }
    };
    let servers = value
        .get("mcpServers")
        .or_else(|| value.get("servers"))
        .and_then(Value::as_object);
    let Some(servers) = servers else {
        return vec![unrecognized_capability(
            CapabilityKind::McpServer,
            ecosystem,
            path,
            &source.hash,
            observed_at_unix_ms,
            "No recognized `mcpServers` or `servers` object was declared.",
        )];
    };
    if servers.is_empty() {
        return vec![unrecognized_capability(
            CapabilityKind::McpServer,
            ecosystem,
            path,
            &source.hash,
            observed_at_unix_ms,
            "The MCP server declaration is empty.",
        )];
    }
    servers
        .iter()
        .map(|(name, config)| {
            let valid_transport = config.get("command").and_then(Value::as_str).is_some()
                || config.get("url").and_then(Value::as_str).is_some();
            let state = if config.get("disabled").and_then(Value::as_bool) == Some(true) {
                CapabilityState::Disabled
            } else if config.is_object() && valid_transport {
                CapabilityState::DeclaredAndRecognized
            } else {
                CapabilityState::Invalid
            };
            make_capability(
                CapabilityKind::McpServer,
                ecosystem,
                path,
                name,
                None,
                &source.hash,
                observed_at_unix_ms,
                state,
                "not_verified",
                Vec::new(),
                if !config.is_object() {
                    Some("Server entry must be a JSON object.".to_owned())
                } else if !valid_transport {
                    Some(
                        "Server entry must declare a command or URL; availability is not tested."
                            .to_owned(),
                    )
                } else {
                    None
                },
                name,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)] // Keeps construction centralized for stable IDs and sanitization.
fn make_capability(
    category: CapabilityKind,
    ecosystem: &str,
    path: &str,
    name: &str,
    description: Option<String>,
    hash: &str,
    observed_at_unix_ms: i64,
    state: CapabilityState,
    availability: &str,
    dependencies: Vec<String>,
    limitation: Option<String>,
    resource_key: &str,
) -> Capability {
    let id_input = format!(
        "{}\0{}\0{}\0{}",
        category.as_str(),
        ecosystem,
        path,
        resource_key
    );
    let digest = Sha256::digest(id_input.as_bytes());
    Capability {
        id: format!("cap_{}", &hex::encode(digest)[..24]),
        name: sanitize_text(name, 120),
        description: description.map(|value| sanitize_text(&value, MAX_DESCRIPTION_CHARS)),
        category,
        ecosystem: ecosystem.to_owned(),
        path: path.to_owned(),
        source_reference: path.to_owned(),
        sha256: Some(hash.to_owned()),
        adapter_version: ADAPTER_VERSION.to_owned(),
        state,
        availability: availability.to_owned(),
        observed_at_unix_ms,
        dependencies,
        relationships: Vec::new(),
        limitation,
    }
}

fn invalid_capability(
    category: CapabilityKind,
    ecosystem: &str,
    path: &str,
    hash: &str,
    observed: i64,
    message: &str,
) -> Capability {
    make_capability(
        category,
        ecosystem,
        path,
        &fallback_name(path),
        None,
        hash,
        observed,
        CapabilityState::Invalid,
        "not_verified",
        Vec::new(),
        Some(message.to_owned()),
        "file",
    )
}

fn unrecognized_capability(
    category: CapabilityKind,
    ecosystem: &str,
    path: &str,
    hash: &str,
    observed: i64,
    message: &str,
) -> Capability {
    make_capability(
        category,
        ecosystem,
        path,
        &fallback_name(path),
        None,
        hash,
        observed,
        CapabilityState::NotRecognized,
        "not_verified",
        Vec::new(),
        Some(message.to_owned()),
        "file",
    )
}

fn unavailable_capability(
    adapter: &Adapter,
    path: &str,
    hash: Option<&str>,
    observed: i64,
    message: &str,
) -> Capability {
    let (category, ecosystem) = match adapter {
        Adapter::Markdown(category, ecosystem) => (*category, *ecosystem),
        Adapter::Hooks(ecosystem) => (CapabilityKind::Hook, *ecosystem),
        Adapter::Mcp(ecosystem) => (CapabilityKind::McpServer, *ecosystem),
    };
    let mut item = make_capability(
        category,
        ecosystem,
        path,
        &fallback_name(path),
        None,
        hash.unwrap_or(""),
        observed,
        CapabilityState::UnavailableReadError,
        "not_verified",
        Vec::new(),
        Some(message.to_owned()),
        "file",
    );
    if hash.is_none() {
        item.sha256 = None;
    }
    item
}

fn fallback_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("unnamed")
        .to_owned()
}

fn sanitize_text(value: &str, max_chars: usize) -> String {
    value
        .chars()
        .filter(|character| !character.is_control() || *character == '\n' || *character == '\t')
        .take(max_chars)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn looks_sensitive_metadata(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "password",
        "passwd",
        "secret",
        "token",
        "credential",
        "api_key",
        "api-key",
        "apikey",
        "access_key",
        "access-key",
        "authorization",
        "bearer ",
        "-----begin ",
        "ghp_",
        "gho_",
        "github_pat_",
        "sk-",
        "akia",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn safe_json_error(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(160)
        .collect()
}

fn read_source(root: &Path, relative: &str) -> std::result::Result<ReadSource, String> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("Invalid relative source path.".to_owned());
    }
    let root =
        fs::canonicalize(root).map_err(|_| "Workspace root could not be resolved.".to_owned())?;
    let mut current = root.clone();
    for component in relative_path.components() {
        let Component::Normal(component) = component else {
            return Err("Invalid relative source path.".to_owned());
        };
        current.push(component);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|_| "Source path could not be read.".to_owned())?;
        if metadata.file_type().is_symlink() {
            return Err("Symlinked capability sources are not followed.".to_owned());
        }
    }
    let canonical =
        fs::canonicalize(&current).map_err(|_| "Source path could not be resolved.".to_owned())?;
    if !canonical.starts_with(&root) {
        return Err("Source escaped the authorized workspace.".to_owned());
    }
    let metadata =
        fs::metadata(&canonical).map_err(|_| "Source metadata could not be read.".to_owned())?;
    if !metadata.is_file() {
        return Err("Capability source is not a regular file.".to_owned());
    }
    if metadata.len() > MAX_CAPABILITY_FILE_BYTES {
        return Err("Configuration exceeds the 1 MiB parser limit.".to_owned());
    }
    let file =
        fs::File::open(&canonical).map_err(|_| "Source file could not be opened.".to_owned())?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_CAPABILITY_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Source file could not be read.".to_owned())?;
    if bytes.len() as u64 > MAX_CAPABILITY_FILE_BYTES {
        return Err("Configuration exceeds the 1 MiB parser limit.".to_owned());
    }
    let after =
        fs::canonicalize(&current).map_err(|_| "Source path changed during read.".to_owned())?;
    let after_path_metadata = fs::symlink_metadata(&current)
        .map_err(|_| "Source path changed during read.".to_owned())?;
    if after != canonical
        || !after.starts_with(&root)
        || after_path_metadata.file_type().is_symlink()
    {
        return Err("Source path changed during read; no descriptor was stored.".to_owned());
    }
    let hash = hex::encode(Sha256::digest(&bytes));
    Ok(ReadSource { bytes, hash })
}

fn artifact_path(root: &Path) -> PathBuf {
    root.join(".tramuri").join("discovery").join(ARTIFACT_NAME)
}

fn read_artifact(path: &Path) -> std::io::Result<Option<CapabilitySnapshot>> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "capability artifact has no parent",
        )
    })?;
    let parent_metadata = match fs::symlink_metadata(parent) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "capability artifact directory is not a real directory",
        ));
    }
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "capability artifact is not a regular file",
        ));
    }
    let file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take(16 * 1024 * 1024).read_to_end(&mut bytes)?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(std::io::Error::other)
}

fn write_artifact(root: &Path, snapshot: &CapabilitySnapshot) -> Result<()> {
    let directory = root.join(".tramuri").join("discovery");
    let metadata = fs::symlink_metadata(&directory).map_err(|source| TramuriError::Io {
        operation: "inspect Tramuri discovery directory",
        path: directory.clone(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(TramuriError::InvalidStatePath(directory));
    }
    let path = artifact_path(root);
    if let Ok(metadata) = fs::symlink_metadata(&path)
        && (metadata.file_type().is_symlink() || !metadata.is_file())
    {
        return Err(TramuriError::InvalidStatePath(path));
    }
    let bytes = serde_json::to_vec_pretty(snapshot).map_err(|error| TramuriError::Io {
        operation: "serialize Tramuri capability registry",
        path: path.clone(),
        source: std::io::Error::other(error),
    })?;
    let temporary = directory.join(format!(
        ".capabilities-{}-{}.tmp",
        std::process::id(),
        snapshot.observed_at_unix_ms
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|source| TramuriError::Io {
            operation: "create temporary capability registry",
            path: temporary.clone(),
            source,
        })?;
    if let Err(source) = file.write_all(&bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(&temporary);
        return Err(TramuriError::Io {
            operation: "write capability registry",
            path: temporary,
            source,
        });
    }
    drop(file);
    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(&path).map_err(|source| TramuriError::Io {
            operation: "replace capability registry",
            path: path.clone(),
            source,
        })?;
    }
    fs::rename(&temporary, &path).map_err(|source| {
        let _ = fs::remove_file(&temporary);
        TramuriError::Io {
            operation: "replace capability registry",
            path,
            source,
        }
    })
}

fn now_unix_ms() -> Result<i64> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TramuriError::Clock)?;
    i64::try_from(elapsed.as_millis()).map_err(|_| TramuriError::Clock)
}
