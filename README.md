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
- A native Rust CLI and a Python SDK backed by the same Rust core.

The published Python SDK includes a prebuilt Rust extension on supported platforms; users do not
need a Rust toolchain.

Tramuri stores its local project state under `.tramuri/`, with SQLite at
`.tramuri/database/tramuri.sqlite3` and the latest scan report under `.tramuri/discovery/`.
Evidence is recorded as references; Tramuri does not fetch external resources or store scanned file
contents as evidence.

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

Tramuri 0.3.0 includes the CLI distribution and project integration installer. Install the CLI in
an isolated tool environment:

```bash
uv tool install tramuri
tramuri --version
tramuri init .
tramuri scan .
```

`pipx install tramuri` is also available as an isolated-install alternative.
If the `tramuri` command is not found after installation, run `uv tool update-shell` and reopen
your terminal.

To add Tramuri guidance to a project, choose a supported assistant platform:

```bash
tramuri install --project --platform codex
tramuri install --project --platform copilot
tramuri install --project --platform claude
tramuri install --project --platform gemini
tramuri integrations status
```

The installer adds platform instructions, a Tramuri skill, and its on-demand references. It
preserves existing user content and does not install hooks. These integrations are optional; Tramuri
does not require an assistant to use its CLI or SDK.

### How an assistant can use Tramuri

The project installer is intended to give a supported assistant persistent, platform-native
guidance for when Tramuri is useful. The skill explains the workflow; the CLI performs Tramuri
operations against local project state. Both are optional, and the assistant remains free to use
other tools.

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

Tramuri is in active early development. Project Discovery, persistent knowledge, Missions, manual
Execution and Evidence records, and criterion assessments are part of the current product. Versioned
Context Pack and Evidence Pack export is planned; it will use explicit selection and authorization
and will be safe by default.

## Community

- [Contributing](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security policy](SECURITY.md)
- [Support](SUPPORT.md)
- [License](LICENSE) (Apache-2.0)
