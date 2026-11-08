# Hook contract

Tramuri uses provider-native hooks only as a deterministic boundary around the
workflow. The handler is the installed command:

```text
tramuri hooks runtime --provider <codex|claude|gemini>
```

It reads one JSON event from standard input, never executes values from that JSON,
and writes provider-compatible JSON only when it needs to add context or deny a
mutating action.

The decision boundary has three parts:

1. The harness normalizes a native payload into HookEvaluation.
2. tramuri.core.policy returns a provider-neutral HookDecision.
3. The harness renders context, allow or block in the provider's native envelope.

This keeps provider output differences out of the workflow policy.

## State policy

The handler finds the nearest project containing `tramuri_output/` and reads SQLite. It
persists only event metadata in `hook_events`:

- provider and lifecycle event;
- session/turn identifier;
- tool name;
- prompt classification (`product_intent` or `general`);
- allowed or blocked decision;
- deterministic state summary.

Prompt text, shell commands, tool arguments and credentials are not persisted.

Lifecycle hooks also maintain a provider-neutral `sessions` record and safe
session events for `SessionStart`, compaction and end boundaries. The runtime
creates a bounded context digest from SQLite state, decisions, evidence,
knowledge and sensor receipts; it does not inject or persist the full prompt.
Checkpoint writes are idempotent when the provider supplies an event identity.
If a checkpoint cannot be written, the hook boundary remains fail-safe and does
not claim durable progress.

The database separates workflow state, session events, checkpoints, reusable
knowledge, sensor receipts and audit events. This prevents a completed command
from being confused with a verified product outcome.

For a product-intent prompt, the handler allows read-only work and Tramuri bootstrap
commands. It blocks edits, writes and non-Tramuri shell commands until a completed
product run is present. After the run completes, the normal provider permission model
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
