# Hook contract

Tramuri uses provider-native hooks only as a deterministic boundary around the
workflow. The handler is the installed command:

```text
tramuri hooks runtime --provider <codex|claude|gemini>
```

It reads one JSON event from standard input, never executes values from that JSON,
and writes provider-compatible JSON only when it needs to add context or deny a
mutating action.

## State policy

The handler finds the nearest project containing `.tramuri/` and reads SQLite. It
persists only event metadata in `hook_events`:

- provider and lifecycle event;
- session/turn identifier;
- tool name;
- prompt classification (`product_intent` or `general`);
- allowed or blocked decision;
- deterministic state summary.

Prompt text, shell commands, tool arguments and credentials are not persisted.

For a product-intent prompt, the handler allows read-only work and Tramuri bootstrap
commands. It blocks edits, writes and non-Tramuri shell commands until a completed
PM run is present. After the run completes, the normal provider permission model
continues to apply.

## Coverage

| Provider | Project configuration | Gate events |
| --- | --- | --- |
| Codex | `.codex/hooks.json` | `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse` |
| Claude Code | `.claude/settings.json` | `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse` |
| Gemini CLI | `.gemini/settings.json` | `SessionStart`, `BeforeAgent`, `BeforeTool`, `AfterTool` |

Hook support is provider- and version-dependent. Codex requires reviewing/trusting
project-local hooks with `/hooks`; disabled or untrusted hooks do not enforce the
gate. Hooks are additive and merged with existing user configuration.
