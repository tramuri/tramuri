# Provider capability reference

Tramuri detects provider executables with a read-only `--version` probe and
also recognizes project-local configuration surfaces. It never installs an
external CLI or MCP server.

| Provider | Managed | Not claimed automatically |
| --- | --- | --- |
| Codex | `AGENTS.md` block, `.agents/skills`, `.codex/agents`, `.codex/hooks.json` | hook execution until the user trusts it with `/hooks`; actual agent invocation |
| Claude Code | `CLAUDE.md` block, `.claude/skills`, `.claude/agents`, merged lifecycle hooks | existing instructions outside the managed block; actual agent invocation |
| Gemini CLI | `GEMINI.md` block, `.gemini/skills`, `.gemini/agents`, `.gemini/commands/tramuri.toml`, merged lifecycle hooks | runtime subagent availability depends on the installed CLI version; actual agent invocation |

Every install writes `tramuri_output/manifests/<provider>.json` with component
hashes.
Existing files are backed up before a managed update. A modified managed file is
preserved and reported as a conflict unless `--force` is explicitly supplied.

The provider plan installs the `tramuri` entry skill and each focused product
skill in the provider's native skills directory. Focused skills use public IDs
prefixed with `tramuri-`; their directory slugs stay stable so updates replace
their metadata in place. For Codex, use `tramuri update --provider codex` to
refresh `.agents/skills/`; restart Codex if new names are not visible in the
skill selector.

The provider adapter is an installation and capability boundary; it is not a
fake programmatic LLM backend. Each built-in adapter exposes a HarnessManifest
containing version, component paths, capabilities, hook events and notes. The
generic projector validates paths, resolves manifest variables and hands the
result to the conflict-aware installer. Adding a provider must not require
importing or changing tramuri.core or the product workflow module. The deterministic product
demo is intentionally local and does not act as an LLM backend.

Agent files declare bounded product capabilities with explicit trigger
descriptions and output contracts. The Tramuri entry skill selects only those
that match the user's current request. Installed files do not prove that the
provider loaded or called an agent; when its agent tool is unavailable, the
assistant performs the capability inline and must not report delegation.

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
