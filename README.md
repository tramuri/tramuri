# Tramuri

<p align="center">
  <img src="https://raw.githubusercontent.com/tramuri/tramuri/main/assets/logo.png" alt="Tramuri logo" width="220">
</p>

<p align="center"><a href="https://github.com/tramuri/tramuri/blob/main/README.md">English</a> · <a href="https://github.com/tramuri/tramuri/blob/main/README.pt-BR.md">Português (Brasil)</a></p>

<p align="center"><strong>From intent to verified outcomes.</strong></p>

<p align="center">
  <a href="https://github.com/tramuri/tramuri/actions/workflows/ci.yml"><img src="https://github.com/tramuri/tramuri/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://pypi.org/project/tramuri/"><img src="https://img.shields.io/pypi/v/tramuri" alt="PyPI version"></a>
</p>

Tramuri is open-source, local-first infrastructure for turning product and business objectives
into coordinated work and verifiable outcomes. It helps people, agents, and tools carry context
between sessions and distinguish completed work from results that have actually been reviewed.

Tramuri is designed for Product Engineers, Forward Deployed Engineers (FDEs), and product teams.
It works without requiring a source-code repository, an LLM, or a particular agent framework.

## What Tramuri provides

- **Missions and Success Contracts** to describe an objective and its acceptance criteria.
- **Persistent knowledge** with source references and history.
- **Execution and evidence records** that keep work, supporting references, and assessment distinct.
- **Verified outcomes** based on explicit criterion reviews; a completed execution alone does not
  mark a Mission as achieved.
- **Project Discovery** as an optional way to inventory a code project. Scanning does not execute
  repository files and excludes sensitive paths by default.
- **Capability Discovery** to catalog supported project-local skills, instructions, agents, hooks,
  and MCP declarations without executing or connecting to them.
- A native Rust CLI and a Python SDK backed by the same Rust core.

The published Python SDK includes a prebuilt Rust extension on supported platforms; users do not
need a Rust toolchain.

Tramuri stores its local project state under `.tramuri/`, with SQLite at
`.tramuri/database/tramuri.sqlite3` and the latest scan report under `.tramuri/discovery/`.
Evidence is recorded as references; Tramuri does not fetch external resources or store scanned file
contents as evidence. The capability catalog is a derived snapshot; declared MCP servers and hooks
are not considered operationally verified.

## Prerequisites

| Requirement | Version | Check | Install |
| --- | --- | --- | --- |
| Python | 3.11 or newer | `python --version` | [python.org](https://www.python.org/downloads/) |
| `uv` (recommended) | Any current version | `uv --version` | [uv installation guide](https://docs.astral.sh/uv/getting-started/installation/) |

PyPI installations use prebuilt wheels on supported platforms. You do not need Rust installed.

## Install

### Python SDK

Install Tramuri into an existing Python project:

```bash
uv add tramuri
```

Or use pip:

```bash
python -m pip install tramuri
```

### CLI and assistant integrations

The Tramuri package includes the CLI distribution and project integration installer. Install the CLI in
an isolated tool environment:

```bash
uv tool install tramuri
tramuri --version
```

`pipx install tramuri` is also available as an isolated-install alternative.
If the `tramuri` command is not found after installation, run `uv tool update-shell` and reopen
your terminal.

After a new Tramuri release is published, update it with the package manager used to install it:

```bash
uv tool upgrade tramuri
# If installed with pipx instead:
pipx upgrade tramuri
tramuri --version
```

Install Tramuri into the project for the assistant you use, preserving existing instructions and hooks:

```bash
tramuri install --platform codex --project
```

Then describe a substantive goal in the assistant chat, for example: “I want to improve customer
onboarding” or “I am taking over this customer integration.” The installed instructions guide a
Mission start or resume even if you do not mention Tramuri. A clear goal initializes the local
database automatically; tell the assistant your work perspective only if it is not clear from the
request. You do not need to run `tramuri init` first or learn CLI syntax. Mission and Success Contract
setup stays exploratory, and a project scan is optional. At the end of a coherent work session that
changes Mission records, the assistant refreshes the local reports. It opens the HTML only if it has
not already opened it during that session; otherwise it refreshes the files without opening another
browser window. You can also ask “open the Tramuri report,” run `tramuri status --open`, or run
`tramuri status --refresh` to refresh without opening the browser.

The local report opens in English or Brazilian Portuguese and shows work perspective (Product
Engineer/FDE) separately from the recorded actor. Actor labels are provided by the caller and are
not authenticated; Tramuri does not infer a current Mission owner.

`tramuri capabilities --type skill --json` filters the latest declarative catalog. Scanning honors
project ignore rules and sensitive-path exclusions; it does not read global agent settings, execute
hooks, start MCP servers, or make network connections.

Codex, Claude Code, Cursor, and Gemini CLI receive native project instructions, a Tramuri overview
skill plus focused setup, Product Engineering, discovery, Mission, and integrity skills, and a
project-local session-start hook. Hooks check local database-file presence and give startup guidance;
they do not read Mission records or attach prompt data. The assistant must explicitly run Tramuri commands
when it needs workspace context. Hooks follow each host's project trust settings, and installation
preserves unrelated instructions and hook entries. Cursor uses `.cursor/rules/tramuri.mdc`,
`.cursor/skills/tramuri/`, and `.cursor/hooks.json`; the other hosts use their native instruction,
skill, and hook locations.

Copilot project instructions and skills remain supported; Copilot hooks are not installed by Tramuri.

```bash
tramuri install --project --platform copilot
tramuri install --project --platform claude
tramuri install --project --platform cursor
tramuri install --project --platform gemini
```

`tramuri status` summarizes the workspace, `tramuri status --open` generates and opens the local
report, `tramuri status --refresh` refreshes the report files without opening a browser, and
`tramuri resume --mission <id>` reconstructs recorded work and readiness for the selected next
activity. A Mission can start without criteria; its Outcome stays pending until required criteria
are added and reviewed. Readiness is advisory and never blocks execution:

```bash
tramuri start --goal "Criar um produto para pequenos negócios"
tramuri status
tramuri resume --mission <id>
```

`tramuri start` creates an exploratory Mission and routes clear phrases to a work perspective.
Product Engineer Missions start with `product_framing`; other perspectives start with Discovery.
The Context Readiness stays `not_assessed` until
someone records an assessment. When intent is unclear, provide `--perspective product_engineer|fde|mixed|other`
or ask the assistant to clarify. Routing in the Rust Core uses a small phrase set; it does not claim
general natural-language understanding. A connected assistant can suggest a perspective, which the
user can correct.

Choose a default perspective for this workspace with:

```bash
tramuri profile set product_engineer
tramuri profile set fde --ask always
tramuri profile set auto
```

The current Mission focus can be changed without changing perspective provenance or approving
Context Readiness. Product Engineer phases can use `product_framing`, `discover`, `define`, `scope`,
`prototype`, `implement`, `validate`, `launch`, and `learn`. Record a meaningful Execution once work
starts, then close that same record explicitly:

```bash
tramuri mission workflow set --mission <id> --perspective product_engineer --activity prototype
tramuri mission workflow set --mission <legacy-id> --perspective product_engineer --source inferred --activity product_framing
tramuri mission workflow activity --mission <id> --activity discover
tramuri execution update --id <execution-id> --status completed
```

The available terminal statuses are `completed`, `blocked`, `failed`, and `cancelled`. Execution
completion does not verify the Mission outcome. Add or revise criteria during Discovery when they
become clear:

```bash
tramuri readiness assess --mission <id> --activity discover --state needs_review \
  --gap "Interview data is not available yet"
tramuri status --open
```

Use `tramuri readiness show --mission <id>` to review the current assessment and its history.
Changing the objective, Success Contract, or explicitly linked Knowledge/Capabilities marks the
latest readiness as stale while retaining the original assessment. `tramuri mission list`
discovers Missions. The report files are `.tramuri/reports/tramauri.html`, `tramauri.md`, and
`tramauri.json`. Opening an older supported database can apply its existing schema migrations;
status reads do not change Mission, execution, evidence, or outcome records. These commands work
without an assistant; integrations are optional.

Knowledge and discovered Capabilities can be explicitly associated with a Mission using
`tramuri mission link-knowledge` and `tramuri mission link-capability`. `tramuri resume --mission <id>`
includes only those recorded associations. Tramuri does not infer Mission relevance; capability
links retain the observed source hash and become changed, missing, or unverified after later scans.
Actor attribution is caller supplied and is not authenticated.

Follow complete synthetic projects from setup to handoff in the [Product Engineer and FDE
examples](docs/examples/README.md).

### How an assistant can use Tramuri

The project installer gives a supported assistant persistent, platform-native workflow guidance.
For substantive project work, the instructions direct the assistant to start or resume a Mission
before exploring files or invoking other creation tools, then continue the requested task. The skill
explains the workflow; the CLI performs Tramuri operations against local project state. Hooks do not
inspect prompts or force an assistant to follow instructions, so this remains host guidance rather
than a technical enforcement mechanism.

| Interface | Use |
| --- | --- |
| CLI | Run local discovery, knowledge, and mission workflows. |
| Assistant skill and instructions | Help an installed assistant choose and use Tramuri's CLI. |
| Python SDK | Call the same Rust-backed capabilities from Python applications. |
| MCP | No Tramuri MCP server is available currently; use the CLI or SDK. |

Mission creation does not require a Git repository or project scan. See the
[product vision](VISION.md) for Tramuri's scope and principles, and the
[changelog](CHANGELOG.md) for release history.

## Project status

Tramuri is in active early development. Project and Capability Discovery, persistent knowledge,
Missions, explicit Knowledge/Capability links, manual Execution and Evidence records, and criterion
assessments are part of the current product. Capability formats are platform-specific; only
documented project paths are recognized.
Versioned
Context Pack and Evidence Pack export is planned; it will use explicit selection and authorization
and will be safe by default.

## Community

- [Contributing](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security policy](SECURITY.md)
- [Support](SUPPORT.md)
- [License](LICENSE) (Apache-2.0)
