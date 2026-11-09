# Tramuri

<p align="center">
  <img src="https://raw.githubusercontent.com/tramuri/tramuri/main/assets/logo.png" alt="Tramuri logo" width="220">
</p>

<p align="center"><a href="https://github.com/tramuri/tramuri/blob/main/README.md">English</a> · <a href="https://github.com/tramuri/tramuri/blob/main/README.pt-BR.md">Português (Brasil)</a></p>

<p align="center"><strong>From intent to a verified product definition.</strong></p>

<p align="center">
  <a href="https://github.com/tramuri/tramuri/actions/workflows/ci.yml"><img src="https://github.com/tramuri/tramuri/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://pypi.org/project/tramuri/"><img src="https://img.shields.io/pypi/v/tramuri" alt="PyPI version"></a>
</p>

Tramuri is a local-first Python tool for turning an incomplete product idea into
a structured, evidence-aware product result. It keeps workflow state in SQLite,
installs provider-native guidance for Codex, Claude Code and Gemini CLI, and
generates a bilingual report plus a complete offline product experience.

The current product-definition workflow preserves the supplied domain, supports
provider-backed market research, and generates connected marketing, customer
and admin surfaces when the product calls for them. Product Engineering remains
a future stage; the generated experience is not a production application.

The runtime is split into a provider-neutral core, the product capability module and
thin provider harnesses. The core owns workflow state and policy; harnesses own
native Codex, Claude Code and Gemini CLI projections. The public API exposes the
new core, product module and harness boundaries directly; deprecated facade
packages are not part of this release.

## Prerequisites

| Requirement | Minimum | Check | Install |
| --- | --- | --- | --- |
| Python | 3.10+ | `python --version` | [python.org](https://www.python.org/downloads/) |
| uv (recommended) | any | `uv --version` | `curl -LsSf https://astral.sh/uv/install.sh \| sh` |
| pipx (alternative) | current | `pipx --version` | `python -m pip install --user pipx` |

Platform shortcuts:

```bash
# macOS (Homebrew)
brew install python@3.12 uv

# Windows (PowerShell)
winget install --id=astral-sh.uv -e

# Ubuntu/Debian
sudo apt install python3.12 python3-pip pipx
# or install uv:
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## Quickstart (30 seconds)

```bash
uv tool install tramuri
tramuri init --provider auto
```

Open your assistant in the same project and run the installed skill:

```text
/tramuri .        # Claude Code or Gemini CLI
$tramuri .        # Codex
```

That is enough to begin. Use `--provider all` instead of `auto` when you want
to install the project components for all three providers explicitly.

## Install

### Important

The official PyPI package is `tramuri`. The command installed on your `PATH` is
also `tramuri`. This project does not use the Graphify package name or command.

Choose one isolated CLI installation method:

```bash
uv tool install tramuri
# If `tramuri` is not found afterwards, restart the shell or run:
# uv tool update-shell
tramuri --version
```

Alternatively:

```bash
pipx install tramuri
# or
python -m pip install tramuri
```

No Rust, Node.js, Docker, hosted backend or duplicated provider API key is
required.

### Project-scoped provider installation

From the project you want Tramuri to accompany, install the native provider
components once. Installation is project-local by default:

```bash
tramuri init --provider auto       # detect providers available in this project
tramuri init --provider all        # explicitly install all built-in adapters
```

Then start Tramuri inside your assistant:

```text
Claude Code: /tramuri .
Gemini CLI:  /tramuri .
Codex:       $tramuri .
```

`/tramuri .` means “use this project as context”. Claude Code and Gemini expose
that name as a native slash command. Codex exposes project skills with its
official `$skill-name` notation, so `$tramuri .` is the guaranteed Codex form;
the installed instructions also recognize `/tramuri .` when it is sent as a
normal prompt.

Tramuri starts from the user's product intent. If the intent is missing, it asks
one concise grouped question about what to create or improve, who it serves and
the desired outcome. It does not invent a product. Once the intent is clear, it
defaults to MVP, records low-impact assumptions and asks only critical safety,
authorization or irreversible-decision gates.

The generated state and product artifacts stay together in one directory:

```text
tramuri_output/
├── portal/index.html                 # central product map
├── reports/<run-id>/report.html      # product report
├── reports/<run-id>/prototype.html  # offline navigation prototype
├── demo/                             # marketing, customer and admin experience
├── database/tramuri.sqlite3          # durable workflow memory
├── memorys/                          # readable memory and per-run summaries
├── manifests/                        # provider installation records
└── backups/                          # safe installer backups
```

Provider-native files remain in `.agents/`, `.claude/` or `.gemini/` because
those locations are how the assistants discover skills, commands and hooks.

The installed entry skill is written as:

```text
.agents/skills/tramuri/SKILL.md        # Codex
.claude/skills/tramuri/SKILL.md        # Claude Code
.gemini/skills/tramuri/SKILL.md        # Gemini CLI
.gemini/commands/tramuri.toml          # Gemini native /tramuri command
```

Focused product skills are installed alongside the entry skill, for example
`.agents/skills/product-framing/SKILL.md` and
`.agents/skills/product-memory/SKILL.md` in Codex projects. Their public names
are Tramuri-prefixed, so you can invoke `$tramuri-framing` or `$tramuri-memory`.
Run `tramuri update --provider codex` to install or refresh them. If newly installed
skills do not appear in Codex, restart Codex.

Use the terminal to inspect the same real workflow state:

```bash
tramuri status
tramuri runs list
tramuri resume <run-id>
tramuri memory show
tramuri memory search "pricing decision"
tramuri memory save --title "Launch region" --content "Start in Brazil" --classification decision
```

Tramuri saves concise project memory and workflow state in SQLite, then writes
readable, regenerable files to `tramuri_output/memorys/`. It does not save full
conversation transcripts. When asked what it remembers or what work was in
progress, the `tramuri-memory` skill retrieves these records and checks current
status before answering.

## Common commands

Run the installed skill inside an assistant:

```text
/tramuri .                              # Claude Code or Gemini CLI
$tramuri .                              # Codex
/tramuri explain "value proposition"   # explain persisted product facts
$tramuri explain "MVP scope"            # Codex equivalent
```

Run deterministic or machine-readable operations from a shell:

```bash
tramuri --help
tramuri status
tramuri --json explain "value proposition"
tramuri product run --idea-file idea.md --mode mvp
tramuri product research --run-id <run-id> --research-file research.json
tramuri product demo --mode mvp
tramuri experience check <run-id>
tramuri report open <run-id>
tramuri report open <run-id> --no-open
tramuri runs list
tramuri runs show <run-id>
tramuri resume <run-id>
tramuri providers list
tramuri doctor
tramuri init --provider all --dry-run
tramuri update --provider all
tramuri uninstall --provider codex
```

`explain` reads only persisted artifacts, decisions, dependencies and evidence
from `tramuri_output`. When a subject is not recorded, it reports that gap
instead of inventing an explanation.

If `tramuri` is not found after an isolated install, restart the shell or run
`uv tool update-shell`. If a provider is not detected, run `tramuri doctor` and
then install the provider's CLI using its official instructions. Tramuri never
installs third-party assistants or MCP servers automatically.

## Run the deterministic demo

```bash
tramuri product demo --mode mvp
tramuri runs list
tramuri report open <run-id> --no-open
```

The demo requires no network or provider authentication. Generated hypotheses
and example data are explicitly marked as synthetic. The local demo records
market research as `not_run`; it never presents invented web research as
evidence. A provider can produce a `tramuri.research.v1` JSON pack with source
URLs, competitors, market signals, buying motivations, positioning, audience
sizing and build/buy/partner trade-offs; import it with `tramuri product
research` to resume the research-dependent stages.

Each demo also produces `tramuri_output/portal/index.html`, the Product
Experience map, and `tramuri_output/demo/` with marketing, customer/learner and
admin surfaces, real local links and simulated interactions. Open the portal to
move from Product Definition to Product Demo; Product UX is represented by the
generated experience, while Product Engineering remains a future stage.
Validate the generated artifacts with `tramuri --json experience check <run-id>`.

The workflow survives provider-session boundaries. Use `tramuri status` to see
the real stage position, checkpoint, sensor gates and next action, then
`tramuri resume <run-id>` to continue. Tramuri records local project knowledge
with provenance and treats repository documents as untrusted data. Low-impact
ambiguity becomes a recorded hypothesis; only critical authorization or safety
decisions become human gates.

## Install provider components

```bash
tramuri providers list
tramuri init --provider codex
tramuri init --provider claude
tramuri init --provider gemini
tramuri init --provider all --dry-run
```

Installation is project-local by default, idempotent, conflict-aware and
manifested. Existing unmanaged files are preserved.

## Development

```bash
uv sync --group dev
uv run pytest
uv run ruff check .
uv run mypy src/tramuri
uv build
```

See [docs/index.md](docs/index.md) for the architecture, provider capability
matrix, product workflow and release procedure.

## License

Apache-2.0. See [LICENSE](LICENSE).
