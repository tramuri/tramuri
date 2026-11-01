use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use clap::ValueEnum;

const START: &str = "<!-- tramuri:managed:start -->";
const END: &str = "<!-- tramuri:managed:end -->";
const SKILL_MARKER: &str = "<!-- tramuri:managed-skill:v1 -->";
const REFERENCE_MARKER: &str = "<!-- tramuri:managed-reference:v1 -->";
const SKILL: &str = include_str!("../../../python/tramuri/skills/tramuri/SKILL.md");
const CODEX_INSTRUCTIONS: &str = include_str!("../../../always_on/agents-md.md");
const COPILOT_INSTRUCTIONS: &str = include_str!("../../../always_on/copilot-instructions.md");
const CLAUDE_INSTRUCTIONS: &str = include_str!("../../../always_on/claude-md.md");
const GEMINI_INSTRUCTIONS: &str = include_str!("../../../always_on/gemini-md.md");
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
    Gemini,
}

impl Platform {
    fn name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Copilot => "copilot",
            Self::Claude => "claude",
            Self::Gemini => "gemini",
        }
    }

    fn instruction_file(self) -> &'static str {
        match self {
            Self::Codex => "AGENTS.md",
            Self::Copilot => ".github/copilot-instructions.md",
            Self::Claude => "CLAUDE.md",
            Self::Gemini => "GEMINI.md",
        }
    }

    fn skill_file(self) -> &'static str {
        match self {
            Self::Codex => ".agents/skills/tramuri/SKILL.md",
            Self::Copilot => ".github/skills/tramuri/SKILL.md",
            Self::Claude => ".claude/skills/tramuri/SKILL.md",
            Self::Gemini => ".gemini/skills/tramuri/SKILL.md",
        }
    }

    fn instructions(self) -> &'static str {
        match self {
            Self::Codex => CODEX_INSTRUCTIONS,
            Self::Copilot => COPILOT_INSTRUCTIONS,
            Self::Claude => CLAUDE_INSTRUCTIONS,
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
    let skill_path = Path::new(platform.skill_file());
    let reference_paths = reference_paths(platform);
    let block = format!("{START}\n## Tramuri\n{}\n{END}", platform.instructions());

    let _ = ensure_parent(&root, instruction_path)?;
    let skill_target = ensure_parent(&root, skill_path)?;
    validate_managed_target(&skill_target, |text| {
        has_front_matter(text) && text.contains(SKILL_MARKER)
    })?;
    let mut reference_targets = Vec::with_capacity(reference_paths.len());
    for (relative, _) in &reference_paths {
        let target = ensure_parent(&root, relative)?;
        validate_managed_target(&target, |text| text.starts_with(REFERENCE_MARKER))?;
        reference_targets.push(target);
    }

    write_managed_instruction(&root, instruction_path, &block)?;
    write_managed_skill(&root, skill_path)?;
    for ((_, content), target) in REFERENCES.iter().zip(reference_targets) {
        fs::write(target, content)?;
    }
    let mut installed = vec![
        platform.instruction_file().to_owned(),
        platform.skill_file().to_owned(),
    ];
    installed.extend(
        reference_paths
            .iter()
            .map(|(path, _)| path.to_string_lossy().into_owned()),
    );
    Ok(installed)
}

pub fn uninstall(root: &Path, platform: Platform) -> io::Result<Vec<String>> {
    let root = canonical_root(root)?;
    let instruction = Path::new(platform.instruction_file());
    let skill = Path::new(platform.skill_file());
    validate_existing_parent(&root, instruction)?;
    validate_existing_parent(&root, skill)?;
    let reference_paths = reference_paths(platform);
    for (path, _) in &reference_paths {
        validate_existing_parent(&root, path)?;
    }
    let mut removed = Vec::new();
    if remove_managed_instruction(&root, instruction)? {
        removed.push(platform.instruction_file().to_owned());
    }
    for (path, _) in &reference_paths {
        if remove_managed_reference(&root, path)? {
            removed.push(path.to_string_lossy().into_owned());
        }
    }
    let skill_dir = root.join(skill.parent().expect("skill path has a parent"));
    if let Some(references_dir) = reference_paths.first().and_then(|(path, _)| path.parent()) {
        let _ = fs::remove_dir(root.join(references_dir));
    }
    if remove_managed_skill(&root, skill)? {
        removed.push(platform.skill_file().to_owned());
    }
    let _ = fs::remove_dir(skill_dir);
    Ok(removed)
}

pub fn status(root: &Path) -> io::Result<Vec<(Platform, bool)>> {
    let root = canonical_root(root)?;
    Ok([
        Platform::Codex,
        Platform::Copilot,
        Platform::Claude,
        Platform::Gemini,
    ]
    .into_iter()
    .map(|platform| {
        let instructions = root.join(platform.instruction_file());
        let skill = root.join(platform.skill_file());
        let references_installed = reference_paths(platform).iter().all(|(path, _)| {
            read_if_regular(&root.join(path)).is_ok_and(|text| text.starts_with(REFERENCE_MARKER))
        });
        let installed = read_if_regular(&instructions).is_ok_and(|text| has_managed_block(&text))
            && read_if_regular(&skill)
                .is_ok_and(|text| has_front_matter(&text) && text.contains(SKILL_MARKER))
            && references_installed;
        (platform, installed)
    })
    .collect())
}

fn reference_paths(platform: Platform) -> Vec<(PathBuf, &'static str)> {
    let skill_dir = Path::new(platform.skill_file())
        .parent()
        .expect("skill path has a parent");
    REFERENCES
        .iter()
        .map(|(name, content)| (skill_dir.join("references").join(name), *content))
        .collect()
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

fn write_managed_skill(root: &Path, relative: &Path) -> io::Result<()> {
    let path = ensure_parent(root, relative)?;
    match read_if_regular(&path) {
        Ok(text) if has_front_matter(&text) && text.contains(SKILL_MARKER) => {
            fs::write(path, SKILL)
        }
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "{} already exists and is not managed by Tramuri",
                path.display()
            ),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => fs::write(path, SKILL),
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
    fn all_supported_platforms_get_native_instructions_and_discoverable_skills() {
        let project = tempdir().unwrap();
        for platform in [
            Platform::Codex,
            Platform::Copilot,
            Platform::Claude,
            Platform::Gemini,
        ] {
            install(project.path(), platform).unwrap();
            let instructions = project.path().join(platform.instruction_file());
            let skill = project.path().join(platform.skill_file());
            let instructions = fs::read_to_string(instructions).unwrap();
            assert!(instructions.contains(START));
            assert!(instructions.contains(platform.skill_file()));
            assert!(instructions.contains("verified outcome"));
            assert!(instructions.contains("explicit item"));
            let skill = fs::read_to_string(skill).unwrap();
            assert_eq!(
                skill.lines().take(2).collect::<Vec<_>>(),
                ["---", "name: tramuri"]
            );
            assert!(skill.contains(SKILL_MARKER));
            for (relative, _) in reference_paths(platform) {
                let reference = fs::read_to_string(project.path().join(&relative)).unwrap();
                assert!(reference.starts_with(REFERENCE_MARKER));
                assert!(
                    skill.contains(&relative.file_name().unwrap().to_string_lossy().to_string())
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
        let skill = project.path().join(Platform::Claude.skill_file());
        fs::create_dir_all(skill.parent().unwrap()).unwrap();
        fs::write(&skill, "# User skill\n").unwrap();

        let error = install(project.path(), Platform::Claude).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(skill).unwrap(), "# User skill\n");
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
        assert!(!project.path().join(Platform::Gemini.skill_file()).exists());
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
