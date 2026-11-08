# Provider capability reference

Tramuri detects provider executables with a read-only `--version` probe and
also recognizes project-local configuration surfaces. It never installs an
external CLI or MCP server.

| Provider | Managed | Not claimed automatically |
| --- | --- | --- |
| Codex | `AGENTS.md` block, `.agents/skills/tramuri`, `.codex/agents`, `.codex/hooks.json` | hook execution until the user trusts it with `/hooks`; MCP |
| Claude Code | `CLAUDE.md` block, `.claude/skills/tramuri`, `.claude/agents`, merged lifecycle hooks | existing instructions outside the managed block |
| Gemini CLI | `GEMINI.md` block, `.gemini/skills/tramuri`, `.gemini/commands/tramuri.toml`, merged lifecycle hooks | subagents are version-dependent |

Every install writes `tramuri_output/manifests/<provider>.json` with component
hashes.
Existing files are backed up before a managed update. A modified managed file is
preserved and reported as a conflict unless `--force` is explicitly supplied.

The provider adapter is an installation and capability boundary; it is not a
fake programmatic LLM backend. Each built-in adapter exposes a HarnessManifest
containing version, component paths, capabilities, hook events and notes. The
generic projector validates paths, resolves manifest variables and hands the
result to the conflict-aware installer. Adding a provider must not require
importing or changing tramuri.core or the product workflow module. The deterministic product
demo is intentionally local and does not act as an LLM backend.

## Deterministic hook gate

When a provider supports hooks, Tramuri registers a small command hook that reads
the event JSON and consults `tramuri_output/database/tramuri.sqlite3`. It records hook
metadata without storing prompt text, commands or tool arguments. A product-intent
turn receives the current Tramuri state; mutating tool calls are denied until a
completed product run exists, except for the commands and `tramuri_output/input/` writes needed
to bootstrap that run.

The gate is a workflow guardrail, not authentication. Codex project hooks are
subject to Codex's trust review; use `/hooks` to review and trust them. If a
provider disables hooks, runs in an untrusted project, or does not expose the
relevant event, `AGENTS.md`/`CLAUDE.md`/`GEMINI.md` remain the fallback guidance and
Tramuri reports that coverage limitation rather than claiming enforcement.

Provider lifecycle support is capability-based. Codex, Claude Code and Gemini
can persist the same Tramuri session/checkpoint model, while their native event
names and trust requirements remain visible in the manifest. A missing native
event produces an observed fallback, not simulated coverage.
