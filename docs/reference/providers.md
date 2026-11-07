# Provider capability reference

Tramuri detects provider executables with a read-only `--version` probe and
also recognizes project-local configuration surfaces. It never installs an
external CLI or MCP server.

| Provider | Managed | Not claimed automatically |
| --- | --- | --- |
| Codex | `AGENTS.md` block, `.agents/skills`, `.codex/agents`, `.codex/hooks.json` | hook execution until the user trusts it with `/hooks`; MCP |
| Claude Code | `CLAUDE.md` block, `.claude/skills`, `.claude/agents`, merged lifecycle hooks | existing instructions outside the managed block |
| Gemini CLI | `GEMINI.md` block, `.gemini/skills`, merged lifecycle hooks | subagents are version-dependent |

Every install writes `.tramuri/manifests/<provider>.json` with component hashes.
Existing files are backed up before a managed update. A modified managed file is
preserved and reported as a conflict unless `--force` is explicitly supplied.

The provider adapter is an installation and capability boundary; it is not a
fake programmatic LLM backend. `pm demo` is intentionally local and deterministic.

## Deterministic hook gate

When a provider supports hooks, Tramuri registers a small command hook that reads
the event JSON and consults `.tramuri/database/tramuri.sqlite3`. It records hook
metadata without storing prompt text, commands or tool arguments. A product-intent
turn receives the current Tramuri state; mutating tool calls are denied until a
completed PM run exists, except for the commands and `.tramuri/input/` writes needed
to bootstrap that run.

The gate is a workflow guardrail, not authentication. Codex project hooks are
subject to Codex's trust review; use `/hooks` to review and trust them. If a
provider disables hooks, runs in an untrusted project, or does not expose the
relevant event, `AGENTS.md`/`CLAUDE.md`/`GEMINI.md` remain the fallback guidance and
Tramuri reports that coverage limitation rather than claiming enforcement.
