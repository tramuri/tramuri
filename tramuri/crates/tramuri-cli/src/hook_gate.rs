use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::integrations::Platform;

const STATE_TTL_SECS: u64 = 6 * 60 * 60;
const GATE_REASON: &str = "Tramuri must start or resume a Mission before project tools are used. Preflight queries such as `tramuri status` and `tramuri mission list` are allowed but do not unlock the gate. Run `tramuri start --goal \"...\"` for new work or `tramuri resume --mission <id>` for existing work.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateEvent {
    Prompt,
    BeforeTool,
    AfterTool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GateState {
    updated_at: u64,
    unlocked: bool,
    pending_tool_id: Option<String>,
    #[serde(default)]
    pending_command_hash: Option<u64>,
}

pub fn read_input() -> io::Result<Value> {
    let mut input = String::new();
    io::stdin().take(1024 * 1024).read_to_string(&mut input)?;
    serde_json::from_str(&input).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid hook input: {error}"),
        )
    })
}

pub fn handle(platform: Platform, event: GateEvent, input: &Value) -> io::Result<bool> {
    let workspace = workspace(input)?;
    let session = session_id(input)?;
    let scope = scope(platform, input);
    let path = state_path(&workspace, &session, scope.as_deref());
    let now = now_secs();

    match event {
        GateEvent::Prompt => {
            write_state(
                &path,
                &GateState {
                    updated_at: now,
                    unlocked: false,
                    pending_tool_id: None,
                    pending_command_hash: None,
                },
            )?;
            Ok(true)
        }
        GateEvent::BeforeTool => {
            let mut state = read_state(&path, now);
            if state.as_ref().is_some_and(|state| state.unlocked) {
                return Ok(true);
            }
            if is_safe_preflight(input) {
                return Ok(true);
            }
            if is_start_or_resume(input) {
                let state = state.get_or_insert(GateState {
                    updated_at: now,
                    unlocked: false,
                    pending_tool_id: None,
                    pending_command_hash: None,
                });
                state.updated_at = now;
                state.pending_tool_id = tool_id(input);
                state.pending_command_hash =
                    command(input).map(|command| stable_hash(command.as_bytes()));
                write_state(&path, state)?;
                return Ok(true);
            }
            Ok(false)
        }
        GateEvent::AfterTool => {
            let Some(mut state) = read_state(&path, now) else {
                return Ok(false);
            };
            let matches_id = state.pending_tool_id.as_deref().is_some_and(|pending| {
                tool_id(input)
                    .as_deref()
                    .is_some_and(|received| received == pending)
            });
            let matches_command = state.pending_command_hash.is_some_and(|pending| {
                command(input).is_some_and(|command| stable_hash(command.as_bytes()) == pending)
            });
            let matches_pending =
                matches_id || (state.pending_tool_id.is_none() && matches_command);
            if matches_pending {
                state.pending_tool_id = None;
                state.pending_command_hash = None;
                state.unlocked = tool_succeeded(input);
                state.updated_at = now;
                write_state(&path, &state)?;
            }
            Ok(true)
        }
    }
}

pub fn deny_response(platform: Platform, event: GateEvent) -> Option<Value> {
    if event != GateEvent::BeforeTool {
        return None;
    }
    Some(match platform {
        Platform::Codex | Platform::Claude => serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": GATE_REASON
            }
        }),
        Platform::Cursor => serde_json::json!({
            "permission": "deny",
            "user_message": GATE_REASON,
            "agent_message": GATE_REASON
        }),
        Platform::Gemini => serde_json::json!({
            "decision": "deny",
            "reason": GATE_REASON
        }),
        Platform::Copilot => serde_json::json!({
            "permissionDecision": "deny",
            "permissionDecisionReason": GATE_REASON
        }),
    })
}

fn workspace(input: &Value) -> io::Result<PathBuf> {
    let candidate = input
        .get("cwd")
        .and_then(Value::as_str)
        .or_else(|| {
            input
                .get("workspace_roots")
                .and_then(Value::as_array)
                .and_then(|roots| roots.first())
                .and_then(Value::as_str)
        })
        .or_else(|| input.get("working_directory").and_then(Value::as_str))
        .or_else(|| input.get("workingDirectory").and_then(Value::as_str))
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?);
    Ok(fs::canonicalize(&candidate).unwrap_or(candidate))
}

fn session_id(input: &Value) -> io::Result<String> {
    input
        .get("session_id")
        .or_else(|| input.get("sessionId"))
        .or_else(|| input.get("conversation_id"))
        .or_else(|| input.get("conversationId"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| std::env::var("GEMINI_SESSION_ID").ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "hook input has no session identifier",
            )
        })
}

fn scope(platform: Platform, input: &Value) -> Option<String> {
    match platform {
        Platform::Codex => input
            .get("turn_id")
            .and_then(Value::as_str)
            .map(|id| format!("turn:{id}")),
        Platform::Cursor => input
            .get("generation_id")
            .and_then(Value::as_str)
            .map(|id| format!("generation:{id}")),
        _ => None,
    }
}

fn state_path(workspace: &std::path::Path, session: &str, scope: Option<&str>) -> PathBuf {
    let workspace = workspace.to_string_lossy().to_string();
    let workspace = if cfg!(windows) {
        workspace.to_lowercase()
    } else {
        workspace
    };
    let mut key = workspace;
    key.push('\0');
    key.push_str(session);
    if let Some(scope) = scope {
        key.push('\0');
        key.push_str(scope);
    }
    std::env::temp_dir()
        .join("tramuri")
        .join("hook-gates")
        .join(format!("{:016x}.json", stable_hash(key.as_bytes())))
}

fn stable_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

fn read_state(path: &std::path::Path, now: u64) -> Option<GateState> {
    let state: GateState = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    if now.saturating_sub(state.updated_at) <= STATE_TTL_SECS {
        Some(state)
    } else {
        let _ = fs::remove_file(path);
        None
    }
}

fn write_state(path: &std::path::Path, state: &GateState) -> io::Result<()> {
    let parent = path.parent().expect("gate state has a parent directory");
    fs::create_dir_all(parent)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let bytes = serde_json::to_vec(state).map_err(io::Error::other)?;
    fs::write(&temporary, bytes)?;
    match fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(error) if path.exists() => {
            let _ = fs::remove_file(path);
            fs::rename(temporary, path).map_err(|_| error)
        }
        Err(error) => Err(error),
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn tool_id(input: &Value) -> Option<String> {
    input
        .get("tool_use_id")
        .or_else(|| input.get("tool_call_id"))
        .or_else(|| input.get("toolCallId"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn is_start_or_resume(input: &Value) -> bool {
    let tool_name = input
        .get("tool_name")
        .or_else(|| input.get("toolName"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(
        tool_name.as_str(),
        "bash" | "powershell" | "shell" | "run_shell_command" | "terminal"
    ) {
        return false;
    }
    command(input).is_some_and(is_start_resume_command)
}

fn command(input: &Value) -> Option<&str> {
    let tool_input = input.get("tool_input").or_else(|| input.get("toolArgs"));
    tool_input
        .and_then(|value| {
            value
                .get("command")
                .or_else(|| value.get("cmd"))
                .or_else(|| value.get("script"))
                .and_then(Value::as_str)
        })
        .or_else(|| input.get("command").and_then(Value::as_str))
}

fn is_start_resume_command(command: &str) -> bool {
    let Some(tokens) = standalone_tramuri_tokens(command) else {
        return false;
    };
    if tokens.len() < 2 {
        return false;
    }
    let command_name = tokens[1].to_ascii_lowercase();
    matches!(command_name.as_str(), "start" | "resume")
        && !tokens[2..]
            .iter()
            .any(|argument| matches!(argument.as_str(), "--help" | "-h" | "--version" | "-V"))
}

fn is_safe_preflight(input: &Value) -> bool {
    let tool_name = input
        .get("tool_name")
        .or_else(|| input.get("toolName"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(
        tool_name.as_str(),
        "bash" | "powershell" | "shell" | "run_shell_command" | "terminal"
    ) {
        return false;
    }
    command(input).is_some_and(is_safe_preflight_command)
}

fn is_safe_preflight_command(command: &str) -> bool {
    let Some(tokens) = standalone_tramuri_tokens(command) else {
        return false;
    };
    let args = &tokens[1..];
    if matches!(args, [flag] if matches!(flag.as_str(), "--version" | "-V"))
        || args.first().is_some_and(|argument| argument == "help")
        || args
            .iter()
            .any(|argument| matches!(argument.as_str(), "--help" | "-h"))
    {
        return true;
    }
    if matches!(args, [command] if command == "status")
        || matches!(args, [command, flag] if command == "status" && flag == "--json")
    {
        return true;
    }
    matches!(args, [mission, list] if mission == "mission" && list == "list")
        || matches!(args, [mission, list, flag] if mission == "mission" && list == "list" && flag == "--json")
        || matches!(args, [command, flag] if command == "mission" && flag == "--help")
        || matches!(args, [mission, list, flag] if mission == "mission" && list == "list" && matches!(flag.as_str(), "--help" | "-h"))
}

fn standalone_tramuri_tokens(command: &str) -> Option<Vec<String>> {
    let command = command.trim();
    if command.is_empty() {
        return None;
    }
    let tokens = shell_tokens(command)?;
    let executable = tokens
        .first()?
        .rsplit(['/', '\\'])
        .next()?
        .to_ascii_lowercase();
    matches!(executable.as_str(), "tramuri" | "tramuri.exe").then_some(tokens)
}

fn shell_tokens(command: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut chars = command.chars().peekable();

    while let Some(ch) = chars.next() {
        match quote {
            Some('\'') => match ch {
                '\'' => quote = None,
                '\n' | '\r' => return None,
                _ => current.push(ch),
            },
            Some('"') => match ch {
                '"' => quote = None,
                '\n' | '\r' | '`' => return None,
                '$' if chars
                    .peek()
                    .is_some_and(|next| starts_shell_expansion(*next)) =>
                {
                    return None;
                }
                _ => current.push(ch),
            },
            None => match ch {
                '\'' | '"' => quote = Some(ch),
                ch if ch.is_whitespace() => {
                    if !current.is_empty() {
                        tokens.push(std::mem::take(&mut current));
                    }
                }
                ';' | '&' | '|' | '>' | '<' | '\n' | '\r' | '$' | '`' => return None,
                _ => current.push(ch),
            },
            Some(_) => return None,
        }
    }

    if quote.is_some() {
        return None;
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    Some(tokens)
}

fn starts_shell_expansion(next: char) -> bool {
    next.is_alphanumeric()
        || matches!(
            next,
            '_' | '(' | '{' | '?' | '@' | '#' | '!' | '-' | '*' | '$' | '\''
        )
}

fn tool_succeeded(input: &Value) -> bool {
    if input.get("isError").and_then(Value::as_bool) == Some(true)
        || input.get("is_error").and_then(Value::as_bool) == Some(true)
        || input
            .get("error")
            .is_some_and(|error| !error.is_null() && error != "")
    {
        return false;
    }
    for value in [input.get("tool_result"), input.get("toolResult")]
        .into_iter()
        .flatten()
    {
        if let Some(result_type) = value
            .get("result_type")
            .or_else(|| value.get("resultType"))
            .and_then(Value::as_str)
        {
            return result_type.eq_ignore_ascii_case("success");
        }
    }
    let output = input
        .get("tool_output")
        .or_else(|| input.get("tool_response"))
        .or_else(|| input.get("toolResponse"));
    if let Some(output) = output {
        let value = output
            .as_str()
            .and_then(|text| serde_json::from_str::<Value>(text).ok())
            .unwrap_or_else(|| output.clone());
        if output.as_str().is_some_and(contains_failure_marker) {
            return false;
        }
        if value
            .get("exitCode")
            .and_then(Value::as_i64)
            .is_some_and(|code| code != 0)
            || value
                .get("exit_code")
                .and_then(Value::as_i64)
                .is_some_and(|code| code != 0)
            || value.get("isError").and_then(Value::as_bool) == Some(true)
            || value
                .get("error")
                .is_some_and(|error| !error.is_null() && error != "")
        {
            return false;
        }
        if let Some(status) = value.get("status").and_then(Value::as_str) {
            if matches!(status, "failed" | "error") {
                return false;
            }
        }
    }
    true
}

fn contains_failure_marker(output: &str) -> bool {
    let lower = output.to_ascii_lowercase();
    [
        "exited with status ",
        "exit code: ",
        "exit code ",
        "command failed with status ",
    ]
    .iter()
    .filter_map(|marker| {
        lower
            .find(marker)
            .map(|start| (marker, start + marker.len()))
    })
    .any(|(_, start)| {
        lower[start..]
            .trim_start()
            .chars()
            .next()
            .is_some_and(|digit| digit.is_ascii_digit() && digit != '0')
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn gate_requires_successful_start_or_resume_for_each_prompt() {
        let workspace = tempdir().unwrap();
        let prompt = serde_json::json!({"cwd": workspace.path(), "session_id":"s1"});
        let mut tool = serde_json::json!({"cwd": workspace.path(), "session_id":"s1", "tool_name":"Bash", "tool_input":{"command":"cat README.md"}, "tool_use_id":"read-1"});
        assert!(!handle(Platform::Claude, GateEvent::BeforeTool, &tool).unwrap());
        handle(Platform::Claude, GateEvent::Prompt, &prompt).unwrap();
        assert!(!handle(Platform::Claude, GateEvent::BeforeTool, &tool).unwrap());
        tool["tool_input"]["command"] = Value::String("tramuri status".into());
        assert!(handle(Platform::Claude, GateEvent::BeforeTool, &tool).unwrap());
        tool["tool_input"]["command"] = Value::String("tramuri mission list --json".into());
        assert!(handle(Platform::Claude, GateEvent::BeforeTool, &tool).unwrap());
        tool["tool_input"]["command"] = Value::String("cat README.md".into());
        assert!(!handle(Platform::Claude, GateEvent::BeforeTool, &tool).unwrap());
        tool["tool_input"]["command"] =
            Value::String("tramuri start --goal \"new product\"".into());
        tool["tool_use_id"] = Value::String("start-1".into());
        assert!(handle(Platform::Claude, GateEvent::BeforeTool, &tool).unwrap());
        let mut after = tool.clone();
        after["tool_output"] = Value::String("{\"exitCode\":0}".into());
        assert!(handle(Platform::Claude, GateEvent::AfterTool, &after).unwrap());
        tool["tool_input"]["command"] = Value::String("cat README.md".into());
        tool["tool_use_id"] = Value::String("read-2".into());
        assert!(handle(Platform::Claude, GateEvent::BeforeTool, &tool).unwrap());
        handle(Platform::Claude, GateEvent::Prompt, &prompt).unwrap();
        assert!(!handle(Platform::Claude, GateEvent::BeforeTool, &tool).unwrap());
    }

    #[test]
    fn start_command_must_be_the_only_shell_command() {
        let valid = serde_json::json!({"tool_name":"Bash", "tool_input":{"command":"tramuri start --goal \"new product\""}, "tool_use_id":"1"});
        assert!(is_start_or_resume(&valid));
        for command in [
            "tramuri start; rm -rf .",
            "echo tramuri start",
            "tramuri status",
            "uv run tramuri start --goal x",
            "tramuri resume && cat README.md",
        ] {
            let input = serde_json::json!({"tool_name":"Bash", "tool_input":{"command":command}, "tool_use_id":"1"});
            assert!(!is_start_or_resume(&input), "accepted: {command}");
        }
    }

    #[test]
    fn quoted_punctuation_is_allowed_as_start_data() {
        for tool_name in ["Bash", "PowerShell"] {
            for command in [
                "tramuri start --goal \"Aulae; catálogo R$ 200/mês\" --context \"alunos & instrutores\"",
                "tramuri start --goal 'product; $HOME; $(literal)'",
            ] {
                let input = serde_json::json!({
                    "tool_name": tool_name,
                    "tool_input": {"command": command},
                    "tool_use_id": "quoted-start"
                });
                assert!(
                    is_start_or_resume(&input),
                    "rejected quoted data: {command}"
                );
            }
        }
    }

    #[test]
    fn shell_chaining_and_expansions_remain_rejected_when_quoted() {
        for command in [
            "tramuri start --goal \"new product\"; rm -rf .",
            "tramuri start --goal \"new product\" && cat README.md",
            "tramuri start --goal \"$(Get-Content secret.txt)\"",
            "tramuri start --goal \"$env:PATH\"",
            "tramuri start --goal \"`whoami`\"",
        ] {
            let input = serde_json::json!({
                "tool_name": "PowerShell",
                "tool_input": {"command": command},
                "tool_use_id": "unsafe-start"
            });
            assert!(
                !is_start_or_resume(&input),
                "accepted unsafe command: {command}"
            );
        }
    }

    #[test]
    fn safe_preflight_is_narrow_and_never_unlocks() {
        for command in [
            "tramuri --version",
            "tramuri status",
            "tramuri status --json",
            "tramuri mission list",
            "tramuri mission list --json",
            "tramuri start --help",
            "tramuri resume -h",
            "tramuri status --help",
            "tramuri mission create --help",
            "tramuri help mission create",
        ] {
            assert!(is_safe_preflight_command(command), "not allowed: {command}");
        }
        for command in [
            "tramuri status --open",
            "tramuri status --refresh",
            "tramuri status --path ..",
            "tramuri mission list --path ..",
            "tramuri mission create --objective x",
            "tramuri start --help && cat README.md",
            "uv run tramuri status",
        ] {
            assert!(
                !is_safe_preflight_command(command),
                "unexpectedly allowed: {command}"
            );
        }
    }

    #[test]
    fn successful_help_does_not_count_as_start_or_resume() {
        for command in ["tramuri start --help", "tramuri resume -h"] {
            let input = serde_json::json!({
                "tool_name":"Bash",
                "tool_input":{"command":command},
                "tool_use_id":"help"
            });
            assert!(
                !is_start_or_resume(&input),
                "help unlocked workflow: {command}"
            );
        }
    }

    #[test]
    fn failed_start_does_not_unlock_and_sessions_are_isolated() {
        let workspace = tempdir().unwrap();
        let mut start = serde_json::json!({"cwd":workspace.path(),"session_id":"s1","tool_name":"Bash","tool_use_id":"start","tool_input":{"command":"tramuri start --goal x"}});
        assert!(handle(Platform::Codex, GateEvent::BeforeTool, &start).unwrap());
        let mut after = start.clone();
        after["tool_output"] = Value::String("{\"exitCode\":1}".into());
        assert!(handle(Platform::Codex, GateEvent::AfterTool, &after).unwrap());
        start["tool_name"] = Value::String("Read".into());
        start["tool_input"] = serde_json::json!({});
        assert!(!handle(Platform::Codex, GateEvent::BeforeTool, &start).unwrap());
        start["session_id"] = Value::String("s2".into());
        assert!(!handle(Platform::Codex, GateEvent::BeforeTool, &start).unwrap());
    }

    #[test]
    fn copilot_payload_without_tool_ids_is_correlated_by_command_hash() {
        let workspace = tempdir().unwrap();
        let prompt = serde_json::json!({"cwd":workspace.path(),"sessionId":"copilot-1"});
        handle(Platform::Copilot, GateEvent::Prompt, &prompt).unwrap();
        let start = serde_json::json!({
            "cwd":workspace.path(),
            "sessionId":"copilot-1",
            "toolName":"bash",
            "toolArgs":{"command":"tramuri start --goal \"synthetic task\""}
        });
        assert!(handle(Platform::Copilot, GateEvent::BeforeTool, &start).unwrap());
        let mut after = start.clone();
        after["toolResult"] =
            serde_json::json!({"resultType":"success","textResultForLlm":"Mission created"});
        assert!(handle(Platform::Copilot, GateEvent::AfterTool, &after).unwrap());
        let project_tool = serde_json::json!({"cwd":workspace.path(),"sessionId":"copilot-1","toolName":"view","toolArgs":{}});
        assert!(handle(Platform::Copilot, GateEvent::BeforeTool, &project_tool).unwrap());
    }

    #[test]
    fn failure_text_does_not_unlock_gate() {
        assert!(contains_failure_marker(
            "Exited with status 1 tramuri: failed"
        ));
        assert!(!contains_failure_marker("Exited with status 0"));
    }

    #[test]
    fn malformed_or_incomplete_hook_payload_fails_closed() {
        let input = serde_json::json!({"cwd":".","tool_name":"Bash","tool_input":{"command":"cat README.md"}});
        assert!(session_id(&input).is_err());
        assert_eq!(
            deny_response(Platform::Gemini, GateEvent::BeforeTool).unwrap()["decision"],
            "deny"
        );
    }

    #[test]
    fn every_platform_has_a_native_deny_schema() {
        let responses = [
            (Platform::Codex, "hookSpecificOutput", "permissionDecision"),
            (Platform::Claude, "hookSpecificOutput", "permissionDecision"),
            (Platform::Cursor, "permission", ""),
            (Platform::Gemini, "decision", ""),
            (Platform::Copilot, "permissionDecision", ""),
        ];
        for (platform, key, decision_key) in responses {
            let response = deny_response(platform, GateEvent::BeforeTool).unwrap();
            if decision_key.is_empty() {
                assert_eq!(response[key], "deny");
            } else {
                assert_eq!(response[key][decision_key], "deny");
            }
        }
        assert!(deny_response(Platform::Codex, GateEvent::Prompt).is_none());
    }

    #[test]
    fn tool_ids_and_workspace_keys_are_platform_specific() {
        let cursor = serde_json::json!({"conversation_id":"c1","generation_id":"g1","cwd":"."});
        assert_eq!(session_id(&cursor).unwrap(), "c1");
        assert_eq!(
            scope(Platform::Cursor, &cursor).as_deref(),
            Some("generation:g1")
        );
        assert_ne!(
            stable_hash(b"workspace\0s1\0turn:1"),
            stable_hash(b"workspace\0s1\0turn:2")
        );
    }
}
