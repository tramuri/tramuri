# Tramuri

<p align="center">
  <img src="https://raw.githubusercontent.com/tramuri/tramuri/main/assets/logo.png" alt="Tramuri logo" width="220">
</p>

<p align="center"><a href="https://github.com/tramuri/tramuri/blob/main/packages/typescript/README.md">English</a> · <a href="https://github.com/tramuri/tramuri/blob/main/packages/typescript/README.pt-BR.md">Português (Brasil)</a></p>

<p align="center"><strong>From intent to verified outcomes.</strong></p>

<p align="center">
  <a href="https://www.npmjs.com/package/@tramuri/typescript"><img src="https://img.shields.io/npm/v/@tramuri/typescript" alt="npm version"></a>
  <a href="https://pypi.org/project/tramuri/"><img src="https://img.shields.io/pypi/v/tramuri" alt="PyPI version"></a>
</p>

Tramuri is open-source, local-first infrastructure for turning product and business objectives into coordinated work and verifiable outcomes. It helps people, agents, and tools carry context between sessions and distinguish completed work from results that have actually been reviewed.

Tramuri is designed for Product Engineers, Forward Deployed Engineers (FDEs), and product teams. The full product does not require a source-code repository, an LLM, or a particular agent framework.

> **Package scope:** `@tramuri/typescript` is an early TypeScript CLI preview. Today it lets you select a work perspective (Product Engineer, FDE, or mixed) from a terminal or as JSON. It does **not** yet provide the full Mission, SQLite, Knowledge, Execution, Evidence, Outcome, integration, or DEFINE workflow available in the Python distribution. This npm package does not require Python or Rust; the full CLI currently distributed on PyPI requires Python 3.11+ and ships prebuilt Rust-backed wheels on supported platforms.

New to Tramuri? Follow the [Product Engineer tutorial](https://github.com/tramuri/tramuri/tree/main/docs/examples/product-engineer) or the [FDE tutorial](https://github.com/tramuri/tramuri/tree/main/docs/examples/fde). Both include Brazilian Portuguese instructions.

## What the full Tramuri product provides

- **Missions and Success Contracts** to describe objectives and acceptance criteria.
- **Persistent knowledge** with source references and history.
- **Execution and evidence records** that keep work, supporting references, and assessment distinct.
- **Verified outcomes** based on explicit criterion reviews; a completed execution alone does not mark a Mission as achieved.
- **Optional Project Discovery** to inventory a code project without executing repository files.
- **Capability Discovery** to catalog supported project-local skills, instructions, agents, hooks, and MCP declarations without executing or connecting to them.
- A native Rust CLI and Python SDK backed by the same Rust core.

The Python package published on PyPI includes a prebuilt Rust extension on supported platforms, so users do not need a Rust toolchain. Local state is stored under `.tramuri/`; evidence is recorded as references rather than fetched content. Declared hooks and MCP servers are not treated as operationally verified.

## What this npm preview does

The current TypeScript package provides one small, explicit operation:

```text
tramuri-cli perspective choose [--perspective <value>] [--json]
```

Available values:

| Value | Meaning |
| --- | --- |
| `product_engineer` | Focus on the product problem and outcomes for users. |
| `fde` | Focus on a customer need, systems, constraints, and acceptance. |
| `mixed` | Connect product evolution to a customer integration. |

Without `--perspective`, the CLI presents a keyboard-accessible terminal selector. With `--json`, it returns the versioned `tramuri.work-perspective-selection.v1` result. The selection is returned to the caller; it is not persisted and does not create or select a Mission.

This preview does not install agent instructions, skills, or hooks, and does not implement the full product workflows described above. Use the Python distribution below for those capabilities.

## Install the TypeScript CLI

### Node.js

Requires Node.js 22.14 or newer. Bun is not required.

```bash
npm install --global @tramuri/typescript
tramuri-cli --help
tramuri-cli perspective choose --perspective product_engineer --json
```

For a one-off invocation without a global install:

```bash
npx --yes --package @tramuri/typescript tramuri-cli --help
```

The previous executable name `tramuri-ts` remains available as a compatibility alias.

### Bun

Bun is optional. You can add the package to a JavaScript/TypeScript project and run its command without installing Python or Rust:

```bash
bun add @tramuri/typescript
bun run tramuri-cli --help
bun run tramuri-cli perspective choose --perspective fde --json
```

The package is also tested with Bun 1.3.14. Node.js users can use npm/npx and `tramuri-cli` without Bun.

## Install the full Tramuri CLI and Python SDK

The current full product is distributed as `tramuri` on PyPI. This is a separate package from `@tramuri/typescript`.

For the CLI and local agent integrations, install it in an isolated tool environment:

```bash
uv tool install tramuri
tramuri --version
```

The CLI installation uses Python 3.11 or newer and a compatible prebuilt wheel; Rust is not required on supported platforms. To use the SDK in a Python project:

```bash
uv add tramuri
```

Or with pip:

```bash
python -m pip install tramuri
```

Update with the same manager used to install:

```bash
uv tool upgrade tramuri
# or, if installed with pipx:
pipx upgrade tramuri
```

Install project-local instructions, skills, and supported hooks for an agent host, preserving unrelated user configuration:

```bash
tramuri install --project --platform codex
```

Supported integrations in the Python distribution include Codex, Claude Code, Cursor, Gemini CLI, and GitHub Copilot, subject to each host's native hook support and trust settings. Hooks do not run on every surface and are not a guarantee that an agent followed instructions. The full Tramuri workflow can start or resume a Mission before project tools are used when a compatible, enabled hook is active; instructions and skills provide guidance on hosts without an applicable hook.

For detailed setup and work-through tutorials, see [Product Engineer](https://github.com/tramuri/tramuri/tree/main/docs/examples/product-engineer), [FDE](https://github.com/tramuri/tramuri/tree/main/docs/examples/fde), and the [Python package page](https://pypi.org/project/tramuri/).

## Development

From `packages/typescript`:

```bash
bun install --frozen-lockfile
bun run ci
npm pack --dry-run
```

The package is Apache-2.0 licensed. See the [repository](https://github.com/tramuri/tramuri), [vision](https://github.com/tramuri/tramuri/blob/main/VISION.md), and [changelog](./CHANGELOG.md).
