<!-- tramuri:managed-reference:v1 -->

# Assistant platform integrations

Tramuri can install a discoverable skill and a managed instruction section for supported platforms.
Use the installed CLI's help to confirm syntax. The current commands include:

```text
tramuri install --project --platform <codex|copilot|claude|cursor|gemini> --path <project>
tramuri integrations status --path <project>
tramuri uninstall --project --platform <codex|copilot|claude|cursor|gemini> --path <project>
```

The installer owns only its marked instruction section or rule, managed skill/reference files, and
its native project hook entries. It preserves surrounding instructions and settings for Codex
(`.codex/hooks.json`), Claude Code (`.claude/settings.json`), Cursor (`.cursor/hooks.json`), Gemini
CLI (`.gemini/settings.json`), and Copilot (`.github/hooks/tramuri.json`). It refuses to overwrite
unmarked skill, reference, or Cursor rule files and refuses malformed hook JSON without changing it.
Uninstall removes only Tramuri's managed instruction section, rule, skill files, and hook entries;
inspect status and resolve conflicts explicitly.

The installer registers native prompt/turn and tool hooks for Codex, Claude Code, Cursor, Gemini CLI,
and Copilot CLI. While locked, the gate permits only direct non-Mission-writing preflight queries:
`tramuri --version`, `tramuri status [--json]`, `tramuri mission list [--json]`, and direct help
queries. Those commands do not unlock the gate. Other local project tools remain blocked until a
direct `tramuri start` or `tramuri resume` command completes successfully for that prompt/turn.
Status/list may run the CLI's existing additive SQLite migrations when opening an older database;
the hook itself never accesses SQLite. The gate rejects command chaining, wrappers, report-opening/refreshing options, and alternate paths
while locked. Codex scopes the gate to its `turn_id`; the
other integrations reset it on their native prompt event. The gate stores only a short-lived hashed
session/workspace key and a pending tool identifier under the OS temporary directory. It does not
read or persist prompt text, Mission records, or project files. Session-start hints remain a
separate convenience and check only regular database-file presence; they do not open SQLite, run
migrations/scans, or attach Mission records. Hooks are effective only when the host supports, trusts,
and enables them; hosted Codex tools and unsupported Copilot surfaces may bypass project hooks, so
always-on instructions remain the fallback. The installer does not change global configuration.
Capability discovery records hook declarations without executing them.

`tramuri status --open` generates `.tramuri/reports/tramauri.html`, `tramauri.md`, and
`tramauri.json`, then opens the HTML. `tramuri status --refresh` updates those three files without
opening a browser. Use `--open` once when the report should be viewed; after later Mission changes
in the same session, use `--refresh` to avoid opening another window. The HTML is self-contained
and local. Reports omit Mission context, execution
notes, evidence locators, verifier references, and assessment explanations. Mission objectives
and Success Contract criteria are included as necessary local workspace fields. Knowledge is not
attached to a Mission unless an explicit stored relationship exists.

Integration instructions are convenience entry points, not runtime dependencies. Tramuri's core and
CLI remain independent of any assistant platform. Installing an integration does not grant an agent
authority to export data.

## Declarative Capability Discovery

`tramuri scan` builds `.tramuri/discovery/latest-capabilities.json`; `tramuri capabilities` reads
that snapshot and `tramuri capabilities --type <agent|skill|rule|hook|mcp-server> --json` filters it.
The Python SDK exposes the same Core result with `tramuri.capabilities(path, type=...)`.

The current adapters recognize only these project-local paths:

| Ecosystem | Agents | Skills | Rules and instructions | Hooks | MCP declarations |
| --- | --- | --- | --- | --- | --- |
| Codex | Not supported (`.toml` agent definitions are not parsed) | `.agents/skills/**/SKILL.md` | `AGENTS.md` | `.codex/hooks.json` event declarations | Not supported (`config.toml` is not parsed) |
| Claude Code | `.claude/agents/**/*.md` | `.claude/skills/**/SKILL.md`, shared `.agents/skills/**/SKILL.md` | `CLAUDE.md` | `.claude/settings.json` | `.mcp.json`, `mcpServers` in `.claude/settings.json` |
| Cursor | Not supported | `.cursor/skills/**/SKILL.md`, shared `.agents/skills/**/SKILL.md` | `.cursor/rules/**/*.mdc`, `AGENTS.md` | `.cursor/hooks.json` | `.cursor/mcp.json` |
| Gemini CLI | `.gemini/agents/**/*.md` | `.gemini/skills/**/SKILL.md`, shared `.agents/skills/**/SKILL.md` | `GEMINI.md`, `AGENTS.md` | `hooks` in `.gemini/settings.json` | `mcpServers` in `.gemini/settings.json` |
| GitHub Copilot | `.github/agents/**/*.md` | `.github/skills/**/SKILL.md`, shared `.agents/skills/**/SKILL.md` | `.github/copilot-instructions.md`, `.github/instructions/**/*.instructions.md`, `AGENTS.md` | `.github/hooks/**/*.json` | `servers` in `.vscode/mcp.json` |

This is a bounded adapter list, not a claim of complete host compatibility. Personal/global settings,
installed extensions, Codex TOML configuration, and unlisted file paths are not scanned. Hidden
project paths are included by Project Discovery when they are not ignored. Git-ignored files are
excluded; discovery does not override `.gitignore` or the sensitive-file deny list.

Agent and Skill adapters read bounded YAML front matter fields only; they do not store instruction
bodies. Descriptions containing common credential markers are omitted from exported capability
records. Hook adapters record event names, not handler commands. MCP adapters record server names and
whether a `command` or `url` is declared; they do not copy command values, arguments, URLs, headers,
or environment values. No hook or server is executed, and no connection is attempted. Operational
availability is always `not_verified`.

The catalog does not infer relationships to Missions, Knowledge, or source modules. Configurations
are data, not instructions for Tramuri. New file formats require fixtures for valid, invalid,
ignored, oversized, and conflicting inputs before they are called supported.

Format references: [Codex Skills](https://developers.openai.com/plugins/concepts/skills),
[Claude Code subagents](https://code.claude.com/docs/en/sub-agents),
[Claude Code skills](https://code.claude.com/docs/en/skills),
[Claude Code hooks](https://code.claude.com/docs/en/hooks),
[Cursor rules](https://docs.cursor.com/context/rules-for-ai),
[Gemini CLI skills](https://geminicli.com/docs/cli/using-agent-skills/),
[Gemini CLI MCP](https://geminicli.com/docs/tools/mcp-server/), and
[GitHub Copilot customization](https://docs.github.com/en/copilot/reference/customization-cheat-sheet).
