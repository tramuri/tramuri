use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use clap::ValueEnum;

const START: &str = "<!-- tramuri:managed:start -->";
const END: &str = "<!-- tramuri:managed:end -->";
const SKILL_MARKER: &str = "<!-- tramuri:managed-skill:v1 -->";
const REFERENCE_MARKER: &str = "<!-- tramuri:managed-reference:v1 -->";
const CODEX_HOOK_FILE: &str = ".codex/hooks.json";
const CODEX_HOOK_COMMAND: &str = "tramuri status --hint";
const CODEX_HOOK_STATUS: &str = "Checking Tramuri availability";
const CLAUDE_HOOK_FILE: &str = ".claude/settings.json";
const CLAUDE_HOOK_COMMAND: &str = "tramuri status --hint --hook-format claude";
const CURSOR_HOOK_FILE: &str = ".cursor/hooks.json";
const CURSOR_HOOK_COMMAND: &str = "tramuri status --hint --hook-format cursor";
const GEMINI_HOOK_FILE: &str = ".gemini/settings.json";
const GEMINI_HOOK_COMMAND: &str = "tramuri status --hint --hook-format gemini";
const CURSOR_RULE_FRONTMATTER: &str =
    "---\ndescription: Tramuri project guidance\nalwaysApply: true\n---\n\n";
const SKILLS: [(&str, &str); 5] = [
    (
        "tramuri",
        include_str!("../../../python/tramuri/skills/tramuri/SKILL.md"),
    ),
    (
        "tramuri-start",
        include_str!("../../../python/tramuri/skills/tramuri-start/SKILL.md"),
    ),
    (
        "tramuri-discovery",
        include_str!("../../../python/tramuri/skills/tramuri-discovery/SKILL.md"),
    ),
    (
        "tramuri-missions",
        include_str!("../../../python/tramuri/skills/tramuri-missions/SKILL.md"),
    ),
    (
        "tramuri-integrity",
        include_str!("../../../python/tramuri/skills/tramuri-integrity/SKILL.md"),
    ),
];
const CODEX_INSTRUCTIONS: &str = include_str!("../../../always_on/agents-md.md");
const COPILOT_INSTRUCTIONS: &str = include_str!("../../../always_on/copilot-instructions.md");
const CLAUDE_INSTRUCTIONS: &str = include_str!("../../../always_on/claude-md.md");
const GEMINI_INSTRUCTIONS: &str = include_str!("../../../always_on/gemini-md.md");
const CURSOR_INSTRUCTIONS: &str = include_str!("../../../always_on/cursor-rules.md");
const REFERENCES: [(&str, &str); 5] = [
    (
        "project-discovery.md",
        include_str!("../../../python/tramuri/skills/tramuri/references/project-discovery.md"),
    ),
    (
        "knowledge.md",
        include_str!("../../../python/tramuri/skills/tramuri/references/knowledge.md"),
    ),
    (
        "missions-and-outcomes.md",
        include_str!("../../../python/tramuri/skills/tramuri/references/missions-and-outcomes.md"),
    ),
    (
        "context-and-evidence.md",
        include_str!("../../../python/tramuri/skills/tramuri/references/context-and-evidence.md"),
    ),
    (
        "platform-integrations.md",
        include_str!("../../../python/tramuri/skills/tramuri/references/platform-integrations.md"),
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Platform {
    Codex,
    Copilot,
    Claude,
    Cursor,
    Gemini,
}

impl Platform {
    pub fn has_hook(self) -> bool {
        hook_file(self).is_some()
    }

    fn name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Copilot => "copilot",
            Self::Claude => "claude",
            Self::Cursor => "cursor",
            Self::Gemini => "gemini",
        }
    }

    fn instruction_file(self) -> &'static str {
        match self {
            Self::Codex => "AGENTS.md",
            Self::Copilot => ".github/copilot-instructions.md",
            Self::Claude => "CLAUDE.md",
            Self::Cursor => ".cursor/rules/tramuri.mdc",
            Self::Gemini => "GEMINI.md",
        }
    }

    fn skill_root(self) -> &'static str {
        match self {
            Self::Codex => ".agents/skills",
            Self::Copilot => ".github/skills",
            Self::Claude => ".claude/skills",
            Self::Cursor => ".cursor/skills",
            Self::Gemini => ".gemini/skills",
        }
    }

    fn instructions(self) -> &'static str {
        match self {
            Self::Codex => CODEX_INSTRUCTIONS,
            Self::Copilot => COPILOT_INSTRUCTIONS,
            Self::Claude => CLAUDE_INSTRUCTIONS,
            Self::Cursor => CURSOR_INSTRUCTIONS,
            Self::Gemini => GEMINI_INSTRUCTIONS,
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str((*self).name())
    }
}

pub fn install(root: &Path, platform: Platform) -> io::Result<Vec<String>> {
    let root = canonical_root(root)?;
    let instruction_path = Path::new(platform.instruction_file());
    let skills = skill_paths(platform);
    let reference_paths = reference_paths(platform);
    let block = format!("{START}\n## Tramuri\n{}\n{END}", platform.instructions());

    validate_existing_parent(&root, instruction_path)?;
    if matches!(platform, Platform::Cursor) {
        validate_cursor_rule(&root.join(instruction_path))?;
    }
    if let Some(hook_file) = hook_file(platform) {
        validate_existing_parent(&root, Path::new(hook_file))?;
        validate_native_hook(&root.join(hook_file), platform)?;
    }

    let mut skill_targets = Vec::with_capacity(skills.len());
    for (relative, _) in &skills {
        let target = ensure_parent(&root, relative)?;
        validate_managed_target(&target, |text| {
            has_front_matter(text) && text.contains(SKILL_MARKER)
        })?;
        skill_targets.push(target);
    }
    let mut reference_targets = Vec::with_capacity(reference_paths.len());
    for (relative, _) in &reference_paths {
        let target = ensure_parent(&root, relative)?;
        validate_managed_target(&target, |text| text.starts_with(REFERENCE_MARKER))?;
        reference_targets.push(target);
    }
    if matches!(platform, Platform::Cursor) {
        write_cursor_rule(&root, instruction_path, &block)?;
    } else {
        write_managed_instruction(&root, instruction_path, &block)?;
    }
    for ((_, content), target) in skills.iter().zip(skill_targets) {
        write_managed_skill(&target, content)?;
    }
    for ((_, content), target) in REFERENCES.iter().zip(reference_targets) {
        fs::write(target, content)?;
    }
    if hook_file(platform).is_some() {
        install_native_hook(&root, platform)?;
    }
    let mut installed = vec![platform.instruction_file().to_owned()];
    installed.extend(
        skills
            .iter()
            .map(|(path, _)| path.to_string_lossy().into_owned()),
    );
    installed.extend(
        reference_paths
            .iter()
            .map(|(path, _)| path.to_string_lossy().into_owned()),
    );
    if let Some(hook_file) = hook_file(platform) {
        installed.push(hook_file.to_owned());
    }
    Ok(installed)
}

pub fn uninstall(root: &Path, platform: Platform) -> io::Result<Vec<String>> {
    let root = canonical_root(root)?;
    let instruction = Path::new(platform.instruction_file());
    let skills = skill_paths(platform);
    validate_existing_parent(&root, instruction)?;
    for (skill, _) in &skills {
        validate_existing_parent(&root, skill)?;
    }
    let reference_paths = reference_paths(platform);
    for (path, _) in &reference_paths {
        validate_existing_parent(&root, path)?;
    }
    if let Some(hook_file) = hook_file(platform) {
        validate_existing_parent(&root, Path::new(hook_file))?;
        validate_native_hook(&root.join(hook_file), platform)?;
    }
    let mut removed = Vec::new();
    let removed_instruction = if matches!(platform, Platform::Cursor) {
        remove_cursor_rule(&root, instruction)?
    } else {
        remove_managed_instruction(&root, instruction)?
    };
    if removed_instruction {
        removed.push(platform.instruction_file().to_owned());
    }
    for (path, _) in &reference_paths {
        if remove_managed_reference(&root, path)? {
            removed.push(path.to_string_lossy().into_owned());
        }
    }
    if let Some(references_dir) = reference_paths.first().and_then(|(path, _)| path.parent()) {
        let _ = fs::remove_dir(root.join(references_dir));
    }
    for (skill, _) in &skills {
        if remove_managed_skill(&root, skill)? {
            removed.push(skill.to_string_lossy().into_owned());
        }
    }
    if let Some(hook_file) = hook_file(platform) {
        if remove_native_hook(&root, platform)? {
            removed.push(hook_file.to_owned());
        }
    }
    Ok(removed)
}

pub fn status(root: &Path) -> io::Result<Vec<(Platform, bool)>> {
    let root = canonical_root(root)?;
    [
        Platform::Codex,
        Platform::Copilot,
        Platform::Claude,
        Platform::Cursor,
        Platform::Gemini,
    ]
    .into_iter()
    .map(|platform| -> io::Result<(Platform, bool)> {
        let instructions = root.join(platform.instruction_file());
        let skills = skill_paths(platform);
        let references_installed = reference_paths(platform).iter().all(|(path, _)| {
            read_if_regular(&root.join(path)).is_ok_and(|text| text.starts_with(REFERENCE_MARKER))
        });
        let installed = read_if_regular(&instructions).is_ok_and(|text| has_managed_block(&text))
            && skills.iter().all(|(skill, _)| {
                read_if_regular(&root.join(skill))
                    .is_ok_and(|text| has_front_matter(&text) && text.contains(SKILL_MARKER))
            })
            && references_installed;
        let installed = if hook_file(platform).is_some() {
            installed && native_hook_installed(&root, platform)?
        } else {
            installed
        };
        Ok((platform, installed))
    })
    .collect()
}

fn hook_file(platform: Platform) -> Option<&'static str> {
    match platform {
        Platform::Codex => Some(CODEX_HOOK_FILE),
        Platform::Claude => Some(CLAUDE_HOOK_FILE),
        Platform::Cursor => Some(CURSOR_HOOK_FILE),
        Platform::Gemini => Some(GEMINI_HOOK_FILE),
        Platform::Copilot => None,
    }
}

fn hook_command(platform: Platform) -> Option<&'static str> {
    match platform {
        Platform::Codex => Some(CODEX_HOOK_COMMAND),
        Platform::Claude => Some(CLAUDE_HOOK_COMMAND),
        Platform::Cursor => Some(CURSOR_HOOK_COMMAND),
        Platform::Gemini => Some(GEMINI_HOOK_COMMAND),
        Platform::Copilot => None,
    }
}

fn hook_event(platform: Platform) -> Option<&'static str> {
    match platform {
        Platform::Codex | Platform::Claude | Platform::Gemini => Some("SessionStart"),
        Platform::Cursor => Some("sessionStart"),
        Platform::Copilot => None,
    }
}

fn validate_native_hook(path: &Path, platform: Platform) -> io::Result<()> {
    let Some(text) = read_optional_regular(path)? else {
        return Ok(());
    };
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "cannot merge {} hooks because {} is invalid JSON: {error}",
                platform,
                path.display()
            ),
        )
    })?;
    if !value.is_object() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "hook settings file must contain a JSON object",
        ));
    }
    if matches!(platform, Platform::Cursor)
        && value
            .get("version")
            .is_some_and(|version| version.as_u64() != Some(1))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Cursor hooks version must be 1",
        ));
    }
    let Some(hooks) = value.get("hooks") else {
        return Ok(());
    };
    if !hooks.is_object() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "hooks must be an object",
        ));
    }
    let event = hook_event(platform).expect("platform with hook has event");
    if hooks.get(event).is_some_and(|entries| !entries.is_array()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{event} hooks must be an array"),
        ));
    }
    Ok(())
}

fn read_optional_regular(path: &Path) -> io::Result<Option<String>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "refusing to access a symlink or non-file: {}",
                    path.display()
                ),
            ))
        }
        Ok(_) => fs::read_to_string(path).map(Some),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn hook_entry(platform: Platform) -> serde_json::Value {
    let command = hook_command(platform).expect("platform with hook has command");
    match platform {
        Platform::Codex => serde_json::json!({
            "type": "command",
            "command": command,
            "commandWindows": "tramuri.exe status --hint",
            "statusMessage": CODEX_HOOK_STATUS,
            "timeout": 10,
            "additionalContextLimit": 2500
        }),
        Platform::Claude => serde_json::json!({
            "type": "command",
            "command": command,
            "timeout": 10
        }),
        Platform::Cursor => serde_json::json!({"command": command}),
        Platform::Gemini => serde_json::json!({
            "name": "tramuri-session-start",
            "type": "command",
            "command": command,
            "timeout": 10000,
            "description": "Print a static Tramuri reminder without reading workspace records"
        }),
        Platform::Copilot => unreachable!("Copilot does not install hooks"),
    }
}

fn hook_group(platform: Platform) -> serde_json::Value {
    let entry = hook_entry(platform);
    match platform {
        Platform::Codex => serde_json::json!({
            "matcher": "startup|resume",
            "hooks": [entry]
        }),
        Platform::Claude => serde_json::json!({
            "matcher": "startup|resume",
            "hooks": [entry]
        }),
        Platform::Gemini => serde_json::json!({"hooks": [entry]}),
        Platform::Cursor | Platform::Copilot => unreachable!("hook is not grouped"),
    }
}

fn is_managed_hook(value: &serde_json::Value, platform: Platform) -> bool {
    value.get("command").and_then(serde_json::Value::as_str) == hook_command(platform)
}

fn contains_managed_hook(group: &serde_json::Value, platform: Platform) -> bool {
    if matches!(platform, Platform::Cursor) {
        return is_managed_hook(group, platform);
    }
    group
        .get("hooks")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|entries| entries.iter().any(|entry| is_managed_hook(entry, platform)))
}

fn read_hook_config(path: &Path) -> io::Result<Option<serde_json::Value>> {
    let Some(text) = read_optional_regular(path)? else {
        return Ok(None);
    };
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid hook JSON: {error}"),
        )
    })?;
    Ok(Some(value))
}

fn hook_entries_mut(
    value: &mut serde_json::Value,
    platform: Platform,
) -> io::Result<&mut Vec<serde_json::Value>> {
    let event = hook_event(platform).expect("platform with hook has event");
    value
        .as_object_mut()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "hook settings must be an object",
            )
        })?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "hooks must be an object"))?
        .entry(event)
        .or_insert_with(|| serde_json::Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{event} hooks must be an array"),
            )
        })
}

fn native_hook_installed(root: &Path, platform: Platform) -> io::Result<bool> {
    let path = root.join(hook_file(platform).expect("platform with hook has file"));
    let Some(value) = read_hook_config(&path)? else {
        return Ok(false);
    };
    let event = hook_event(platform).expect("platform with hook has event");
    Ok(value
        .get("hooks")
        .and_then(|hooks| hooks.get(event))
        .and_then(serde_json::Value::as_array)
        .is_some_and(|groups| {
            groups
                .iter()
                .any(|group| contains_managed_hook(group, platform))
        }))
}

fn install_native_hook(root: &Path, platform: Platform) -> io::Result<()> {
    let relative = Path::new(hook_file(platform).expect("platform with hook has file"));
    let path = ensure_parent(root, relative)?;
    let mut value = match read_hook_config(&path)? {
        Some(value) => value,
        None if matches!(platform, Platform::Cursor) => {
            serde_json::json!({"version": 1, "hooks": {}})
        }
        None => serde_json::json!({"hooks": {}}),
    };
    if matches!(platform, Platform::Cursor) {
        value
            .as_object_mut()
            .expect("validated Cursor settings object")
            .entry("version")
            .or_insert(serde_json::json!(1));
    }
    let entries = hook_entries_mut(&mut value, platform)?;
    if !entries
        .iter()
        .any(|item| contains_managed_hook(item, platform))
    {
        if matches!(platform, Platform::Cursor) {
            entries.push(hook_entry(platform));
        } else {
            entries.push(hook_group(platform));
        }
    }
    let encoded = serde_json::to_vec_pretty(&value)
        .map_err(|error| io::Error::other(format!("failed to serialize hook settings: {error}")))?;
    fs::write(path, encoded)
}

fn remove_native_hook(root: &Path, platform: Platform) -> io::Result<bool> {
    let path = root.join(hook_file(platform).expect("platform with hook has file"));
    let Some(mut value) = read_hook_config(&path)? else {
        return Ok(false);
    };
    let event = hook_event(platform).expect("platform with hook has event");
    let Some(groups) = value
        .get_mut("hooks")
        .and_then(|hooks| hooks.get_mut(event))
        .and_then(serde_json::Value::as_array_mut)
    else {
        return Ok(false);
    };
    let mut removed = false;
    if matches!(platform, Platform::Cursor) {
        let previous = groups.len();
        groups.retain(|entry| !is_managed_hook(entry, platform));
        removed = previous != groups.len();
    } else {
        let mut retained = Vec::with_capacity(groups.len());
        for group in groups.iter_mut() {
            if let Some(handlers) = group
                .get_mut("hooks")
                .and_then(serde_json::Value::as_array_mut)
            {
                let old_len = handlers.len();
                handlers.retain(|entry| !is_managed_hook(entry, platform));
                let removed_from_group = old_len != handlers.len();
                removed |= removed_from_group;
                if removed_from_group && handlers.is_empty() {
                    continue;
                }
            }
            retained.push(group.clone());
        }
        *groups = retained;
    }
    if removed {
        fs::write(
            path,
            serde_json::to_vec_pretty(&value).map_err(|error| {
                io::Error::other(format!("failed to serialize hook settings: {error}"))
            })?,
        )?;
    }
    Ok(removed)
}

fn reference_paths(platform: Platform) -> Vec<(PathBuf, &'static str)> {
    let skill_dir = Path::new(platform.skill_root()).join("tramuri");
    REFERENCES
        .iter()
        .map(|(name, content)| (skill_dir.join("references").join(name), *content))
        .collect()
}

fn skill_paths(platform: Platform) -> Vec<(PathBuf, &'static str)> {
    let root = Path::new(platform.skill_root());
    SKILLS
        .iter()
        .map(|(name, content)| (root.join(name).join("SKILL.md"), *content))
        .collect()
}

fn validate_cursor_rule(path: &Path) -> io::Result<()> {
    match read_optional_regular(path)? {
        Some(text) if has_front_matter(&text) && has_managed_block(&text) => Ok(()),
        Some(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "{} already exists and is not managed by Tramuri",
                path.display()
            ),
        )),
        None => Ok(()),
    }
}

fn write_cursor_rule(root: &Path, relative: &Path, block: &str) -> io::Result<()> {
    let path = ensure_parent(root, relative)?;
    let content = match read_optional_regular(&path)? {
        Some(original) => replace_managed_block(&original, block)?,
        None => format!("{CURSOR_RULE_FRONTMATTER}{block}\n"),
    };
    fs::write(path, content)
}

fn remove_cursor_rule(root: &Path, relative: &Path) -> io::Result<bool> {
    let path = root.join(relative);
    let original = match read_optional_regular(&path)? {
        Some(text) => text,
        None => return Ok(false),
    };
    if !has_front_matter(&original) || !has_managed_block(&original) {
        return Ok(false);
    }
    let updated = replace_managed_block(&original, "")?;
    if updated.trim() == CURSOR_RULE_FRONTMATTER.trim() {
        fs::remove_file(path)?;
    } else {
        fs::write(path, updated)?;
    }
    Ok(true)
}

fn validate_managed_target(path: &Path, is_managed: impl FnOnce(&str) -> bool) -> io::Result<()> {
    match read_if_regular(path) {
        Ok(text) if is_managed(&text) => Ok(()),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "{} already exists and is not managed by Tramuri",
                path.display()
            ),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn canonical_root(root: &Path) -> io::Result<PathBuf> {
    let root = fs::canonicalize(root)?;
    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "project path is not a directory",
        ));
    }
    Ok(root)
}

fn ensure_parent(root: &Path, relative: &Path) -> io::Result<PathBuf> {
    let mut parent = root.to_path_buf();
    let components = relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .components();
    for component in components {
        let Component::Normal(name) = component else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "integration path contains an unsafe component",
            ));
        };
        parent.push(name);
        match fs::symlink_metadata(&parent) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "refusing to write through non-directory or symlink {}",
                        parent.display()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&parent)?,
            Err(error) => return Err(error),
        }
    }
    Ok(root.join(relative))
}

fn validate_existing_parent(root: &Path, relative: &Path) -> io::Result<bool> {
    let mut parent = root.to_path_buf();
    for component in relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .components()
    {
        let Component::Normal(name) = component else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "integration path contains an unsafe component",
            ));
        };
        parent.push(name);
        match fs::symlink_metadata(&parent) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "refusing to access through non-directory or symlink {}",
                        parent.display()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error),
        }
    }
    Ok(true)
}

fn read_if_regular(path: &Path) -> io::Result<String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "managed file is not a regular file",
            ))
        }
        Ok(_) => fs::read_to_string(path),
        Err(error) => Err(error),
    }
}

fn write_managed_instruction(root: &Path, relative: &Path, block: &str) -> io::Result<()> {
    let path = ensure_parent(root, relative)?;
    let original = match read_if_regular(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };
    let updated = replace_managed_block(&original, block)?;
    if updated != original {
        fs::write(path, updated)?;
    }
    Ok(())
}

fn replace_managed_block(original: &str, block: &str) -> io::Result<String> {
    let starts = original.matches(START).count();
    let ends = original.matches(END).count();
    if starts != ends || starts > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Tramuri integration markers are malformed or duplicated",
        ));
    }
    if starts == 1 {
        let start = original.find(START).expect("count checked");
        let end = original.find(END).expect("count checked") + END.len();
        if end < start {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Tramuri markers are reversed",
            ));
        }
        let mut updated = String::with_capacity(original.len() + block.len());
        updated.push_str(&original[..start]);
        updated.push_str(block);
        updated.push_str(&original[end..]);
        Ok(updated)
    } else if original.is_empty() {
        Ok(format!("{block}\n"))
    } else {
        let separator = if original.ends_with('\n') {
            "\n"
        } else {
            "\n\n"
        };
        Ok(format!("{original}{separator}{block}\n"))
    }
}

fn remove_managed_instruction(root: &Path, relative: &Path) -> io::Result<bool> {
    let path = root.join(relative);
    let original = match read_if_regular(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !has_managed_block(&original) {
        return Ok(false);
    }
    let updated = replace_managed_block(&original, "")?;
    let updated = updated.trim_matches('\n').to_owned();
    if updated.is_empty() {
        fs::write(path, "")?;
    } else {
        fs::write(path, format!("{updated}\n"))?;
    }
    Ok(true)
}

fn has_managed_block(text: &str) -> bool {
    text.matches(START).count() == 1 && text.matches(END).count() == 1
}

fn has_front_matter(text: &str) -> bool {
    text.lines().next() == Some("---")
}

fn write_managed_skill(path: &Path, content: &str) -> io::Result<()> {
    match read_if_regular(path) {
        Ok(text) if has_front_matter(&text) && text.contains(SKILL_MARKER) => {
            fs::write(path, content)
        }
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "{} already exists and is not managed by Tramuri",
                path.display()
            ),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => fs::write(path, content),
        Err(error) => Err(error),
    }
}

fn remove_managed_reference(root: &Path, relative: &Path) -> io::Result<bool> {
    let path = root.join(relative);
    let text = match read_if_regular(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !text.starts_with(REFERENCE_MARKER) {
        return Ok(false);
    }
    fs::remove_file(path)?;
    Ok(true)
}

fn remove_managed_skill(root: &Path, relative: &Path) -> io::Result<bool> {
    let path = root.join(relative);
    let text = match read_if_regular(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !has_front_matter(&text) || !text.contains(SKILL_MARKER) {
        return Ok(false);
    }
    fs::remove_file(&path)?;
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir(parent);
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn install_is_idempotent_and_preserves_existing_instructions() {
        let project = tempdir().unwrap();
        fs::write(
            project.path().join("AGENTS.md"),
            "# My project\nKeep this.\n",
        )
        .unwrap();

        install(project.path(), Platform::Codex).unwrap();
        install(project.path(), Platform::Codex).unwrap();

        let instructions = fs::read_to_string(project.path().join("AGENTS.md")).unwrap();
        assert!(instructions.starts_with("# My project\nKeep this."));
        assert_eq!(instructions.matches(START).count(), 1);
        assert_eq!(instructions.matches(END).count(), 1);
        assert!(
            status(project.path())
                .unwrap()
                .contains(&(Platform::Codex, true))
        );
    }

    #[test]
    fn codex_hook_merges_with_user_hooks_and_uninstall_preserves_them() {
        let project = tempdir().unwrap();
        let hooks = project.path().join(CODEX_HOOK_FILE);
        fs::create_dir_all(hooks.parent().unwrap()).unwrap();
        fs::write(
            &hooks,
            r#"{"description":"user config","hooks":{"SessionStart":[{"matcher":"startup","hooks":[{"type":"command","command":"user-command"}]}],"SessionEnd":[{"hooks":[{"type":"command","command":"user-end"}]}]}}"#,
        )
        .unwrap();

        install(project.path(), Platform::Codex).unwrap();
        install(project.path(), Platform::Codex).unwrap();

        let installed: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&hooks).unwrap()).unwrap();
        let starts = installed["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(starts.len(), 2);
        assert!(installed["hooks"]["SessionEnd"][0]["hooks"][0]["command"] == "user-end");
        assert!(installed["description"] == "user config");
        assert!(native_hook_installed(project.path(), Platform::Codex).unwrap());

        uninstall(project.path(), Platform::Codex).unwrap();

        let after: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&hooks).unwrap()).unwrap();
        assert_eq!(after["hooks"]["SessionStart"].as_array().unwrap().len(), 1);
        assert!(after["hooks"]["SessionStart"][0]["hooks"][0]["command"] == "user-command");
        assert!(after["hooks"]["SessionEnd"][0]["hooks"][0]["command"] == "user-end");
    }

    #[test]
    fn native_session_hooks_merge_idempotently_and_uninstall_only_tramuri_entries() {
        for platform in [
            Platform::Codex,
            Platform::Claude,
            Platform::Cursor,
            Platform::Gemini,
        ] {
            let project = tempdir().unwrap();
            let hook_file = project.path().join(hook_file(platform).unwrap());
            fs::create_dir_all(hook_file.parent().unwrap()).unwrap();
            let seed = match platform {
                Platform::Codex => {
                    r#"{"owner":"user","hooks":{"SessionStart":[{"matcher":"startup","hooks":[{"type":"command","command":"user-hook"}]}]}}"#
                }
                Platform::Claude => {
                    r#"{"model":"user-model","hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"user-hook"}]}],"Stop":[{"hooks":[{"type":"command","command":"user-stop"}]}]}}"#
                }
                Platform::Cursor => {
                    r#"{"version":1,"userSetting":true,"hooks":{"sessionStart":[{"command":"user-hook"}],"afterFileEdit":[{"command":"user-edit"}]}}"#
                }
                Platform::Gemini => {
                    r#"{"theme":"user-theme","hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"user-hook"}]}],"BeforeAgent":[{"hooks":[{"type":"command","command":"user-agent"}]}]}}"#
                }
                Platform::Copilot => unreachable!(),
            };
            fs::write(&hook_file, seed).unwrap();

            install(project.path(), platform).unwrap();
            install(project.path(), platform).unwrap();

            assert!(status(project.path()).unwrap().contains(&(platform, true)));
            let installed: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(&hook_file).unwrap()).unwrap();
            assert!(installed.to_string().contains("user-hook"));
            assert!(
                installed
                    .to_string()
                    .contains(hook_command(platform).unwrap())
            );
            match platform {
                Platform::Codex | Platform::Claude => {
                    let groups = installed["hooks"]["SessionStart"].as_array().unwrap();
                    assert_eq!(groups.len(), 2);
                    assert!(
                        groups
                            .iter()
                            .any(|group| contains_managed_hook(group, platform))
                    );
                }
                Platform::Cursor => {
                    let hooks = installed["hooks"]["sessionStart"].as_array().unwrap();
                    assert_eq!(hooks.len(), 2);
                    assert_eq!(installed["version"], 1);
                }
                Platform::Gemini => {
                    let groups = installed["hooks"]["SessionStart"].as_array().unwrap();
                    assert_eq!(groups.len(), 2);
                    assert!(
                        groups
                            .iter()
                            .any(|group| contains_managed_hook(group, platform))
                    );
                }
                Platform::Copilot => unreachable!(),
            }

            uninstall(project.path(), platform).unwrap();

            let remaining: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(&hook_file).unwrap()).unwrap();
            assert!(remaining.to_string().contains("user-hook"));
            assert!(
                !remaining
                    .to_string()
                    .contains(hook_command(platform).unwrap())
            );
            assert!(!status(project.path()).unwrap().contains(&(platform, true)));
        }
    }

    #[test]
    fn invalid_native_hook_settings_are_rejected_before_any_project_files_change() {
        for platform in [Platform::Claude, Platform::Cursor, Platform::Gemini] {
            let project = tempdir().unwrap();
            let hook_file = project.path().join(hook_file(platform).unwrap());
            fs::create_dir_all(hook_file.parent().unwrap()).unwrap();
            let event = hook_event(platform).unwrap();
            let malformed = format!(r#"{{"hooks":{{"{event}":"invalid"}}}}"#);
            fs::write(&hook_file, &malformed).unwrap();

            let error = install(project.path(), platform).unwrap_err();

            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert_eq!(fs::read_to_string(&hook_file).unwrap(), malformed);
            assert!(!project.path().join(platform.instruction_file()).exists());
            for (skill, _) in skill_paths(platform) {
                assert!(!project.path().join(skill).exists());
            }
        }
    }

    #[test]
    fn invalid_codex_hook_file_refuses_install_without_modifying_instructions() {
        let project = tempdir().unwrap();
        fs::write(project.path().join("AGENTS.md"), "# User instructions\n").unwrap();
        let hook_file = project.path().join(CODEX_HOOK_FILE);
        fs::create_dir_all(hook_file.parent().unwrap()).unwrap();
        fs::write(&hook_file, "not-json").unwrap();

        let error = install(project.path(), Platform::Codex).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(
            fs::read_to_string(project.path().join("AGENTS.md")).unwrap(),
            "# User instructions\n"
        );
        assert_eq!(fs::read_to_string(hook_file).unwrap(), "not-json");
    }

    #[test]
    fn all_supported_platforms_get_native_instructions_and_discoverable_skills() {
        let project = tempdir().unwrap();
        for platform in [
            Platform::Codex,
            Platform::Copilot,
            Platform::Claude,
            Platform::Cursor,
            Platform::Gemini,
        ] {
            install(project.path(), platform).unwrap();
            let instructions = project.path().join(platform.instruction_file());
            let instructions = fs::read_to_string(instructions).unwrap();
            assert!(instructions.contains(START));
            assert!(instructions.contains(&format!("{}/tramuri/SKILL.md", platform.skill_root())));
            assert!(instructions.contains("verified outcome"));
            assert!(instructions.contains("explicit item"));
            let main_skill = fs::read_to_string(
                project
                    .path()
                    .join(platform.skill_root())
                    .join("tramuri/SKILL.md"),
            )
            .unwrap();
            assert_eq!(skill_paths(platform).len(), 5);
            for (relative, _) in skill_paths(platform) {
                let skill = fs::read_to_string(project.path().join(&relative)).unwrap();
                let skill_name = relative
                    .parent()
                    .and_then(Path::file_name)
                    .expect("skill directory has a name")
                    .to_string_lossy();
                assert_eq!(skill.lines().next(), Some("---"));
                assert!(
                    skill
                        .lines()
                        .any(|line| line == format!("name: {skill_name}"))
                );
                assert!(skill.contains(SKILL_MARKER));
            }
            for (relative, _) in reference_paths(platform) {
                let reference = fs::read_to_string(project.path().join(&relative)).unwrap();
                assert!(reference.starts_with(REFERENCE_MARKER));
                assert!(
                    main_skill
                        .contains(&relative.file_name().unwrap().to_string_lossy().to_string())
                );
            }
        }
    }

    #[test]
    fn managed_skill_front_matter_accepts_lf_and_crlf() {
        assert!(has_front_matter("---\nname: tramuri\n"));
        assert!(has_front_matter("---\r\nname: tramuri\r\n"));
        assert!(!has_front_matter("----\r\nname: tramuri\r\n"));
    }

    #[test]
    fn install_refuses_to_overwrite_an_existing_unmanaged_skill() {
        let project = tempdir().unwrap();
        let skill = project
            .path()
            .join(".claude/skills/tramuri-discovery/SKILL.md");
        fs::create_dir_all(skill.parent().unwrap()).unwrap();
        fs::write(&skill, "# User skill\n").unwrap();

        let error = install(project.path(), Platform::Claude).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(skill).unwrap(), "# User skill\n");
        assert!(!project.path().join("CLAUDE.md").exists());
    }

    #[test]
    fn uninstall_removes_only_the_managed_section() {
        let project = tempdir().unwrap();
        fs::write(project.path().join("GEMINI.md"), "# User instructions\n").unwrap();
        install(project.path(), Platform::Gemini).unwrap();

        uninstall(project.path(), Platform::Gemini).unwrap();

        let instructions = fs::read_to_string(project.path().join("GEMINI.md")).unwrap();
        assert!(instructions.contains("# User instructions"));
        assert!(!instructions.contains(START));
        for (skill, _) in skill_paths(Platform::Gemini) {
            assert!(!project.path().join(skill).exists());
        }
        for (relative, _) in reference_paths(Platform::Gemini) {
            assert!(!project.path().join(relative).exists());
        }
    }

    #[test]
    fn install_refuses_an_unmanaged_reference_without_modifying_instructions() {
        let project = tempdir().unwrap();
        fs::write(project.path().join("CLAUDE.md"), "# Existing guidance\n").unwrap();
        let reference = project
            .path()
            .join(".claude/skills/tramuri/references/knowledge.md");
        fs::create_dir_all(reference.parent().unwrap()).unwrap();
        fs::write(&reference, "# User reference\n").unwrap();

        let error = install(project.path(), Platform::Claude).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read_to_string(project.path().join("CLAUDE.md")).unwrap(),
            "# Existing guidance\n"
        );
        assert_eq!(fs::read_to_string(reference).unwrap(), "# User reference\n");
    }

    #[test]
    fn uninstall_preserves_unmanaged_files_in_references_directory() {
        let project = tempdir().unwrap();
        install(project.path(), Platform::Codex).unwrap();
        let custom = project
            .path()
            .join(".agents/skills/tramuri/references/custom.md");
        fs::write(&custom, "# Custom user reference\n").unwrap();

        uninstall(project.path(), Platform::Codex).unwrap();

        assert_eq!(
            fs::read_to_string(custom).unwrap(),
            "# Custom user reference\n"
        );
        assert!(
            project
                .path()
                .join(".agents/skills/tramuri/references")
                .is_dir()
        );
    }

    #[test]
    fn install_rejects_symlinked_integration_directories() {
        let project = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let link = project.path().join(".claude");
        #[cfg(unix)]
        std::os::unix::fs::symlink(outside.path(), &link).unwrap();
        #[cfg(windows)]
        if std::os::windows::fs::symlink_dir(outside.path(), &link).is_err() {
            return;
        }

        let error = install(project.path(), Platform::Claude).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(!outside.path().join("skills/tramuri/SKILL.md").exists());
    }

    #[test]
    fn uninstall_refuses_symlinked_integration_directories() {
        let project = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let skill_dir = outside.path().join("tramuri");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!("---\nname: tramuri\n---\n{SKILL_MARKER}\n"),
        )
        .unwrap();
        let link = project.path().join(".claude");
        #[cfg(unix)]
        std::os::unix::fs::symlink(outside.path(), &link).unwrap();
        #[cfg(windows)]
        if std::os::windows::fs::symlink_dir(outside.path(), &link).is_err() {
            return;
        }

        let error = uninstall(project.path(), Platform::Claude).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(skill_dir.join("SKILL.md").is_file());
    }
}
