# Provider capability reference

Tramuri detects provider executables with a read-only `--version` probe and
also recognizes project-local configuration surfaces. It never installs an
external CLI or MCP server.

| Provider | Managed | Not claimed automatically |
| --- | --- | --- |
| Codex | `.agents/skills`, `.codex/agents` | hooks and MCP unless verified |
| Claude Code | `.claude/skills`, `.claude/agents`, `.claude/hooks`, merged settings | existing `CLAUDE.md` is preserved |
| Gemini CLI | `.gemini/skills`, merged settings and checkpoint hook | subagents are version-dependent |

Every install writes `.tramuri/manifests/<provider>.json` with component hashes.
Existing files are backed up before a managed update. A modified managed file is
preserved and reported as a conflict unless `--force` is explicitly supplied.

The provider adapter is an installation and capability boundary; it is not a
fake programmatic LLM backend. `pm demo` is intentionally local and deterministic.
