# Changelog

Mudanças notáveis do Tramuri seguem [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
e [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Nothing yet.

### Changed

- Nothing yet.

## [0.14.0] - 2026-11-08

### Added

- A context-aware Product Demo Engine that classifies product experiences and
  generates versioned, offline demo specifications, templates, interactions,
  and traceability from persisted product context.
- Evidence-backed product research packs for market signals, competitors,
  buying motivation, positioning, sizing, and build/buy/partner trade-offs.
- Curated project-memory `save`, `search`, and `show` commands with SQLite
  persistence and regenerable Markdown projections under
  `tramuri_output/memorys/`.
- Focused product capabilities with Tramuri-prefixed skills, including
  `$tramuri-status`, `$tramuri-memory`, and `$tramuri-experience`.

### Changed

- Provider integrations install focused Tramuri-prefixed product skills
  individually and direct work to the relevant capability.
- Product Experience reports now expose persisted delivery, browser validation,
  review results, and product-to-demo traceability.

### Fixed

- Browser validation now distinguishes unavailable browser tooling from failed
  product interactions, and `experience check` persists the resulting delivery
  status accurately.
- Experience lifecycle status now follows the latest recorded transition,
  including when a workflow returns to a previously seen state.

## [0.13.3] - 2026-11-08

### Fixed

- Provider detection now treats an executable as installed even when its
  `--version` probe fails or returns no version output.
- The automatic installation guidance now explains how to choose a provider
  explicitly when no supported CLI is detected.

## [0.13.2] - 2026-11-08

### Added

- Deterministic product workflows with resumable local context, status, knowledge
  and explain capabilities.
- Product Experience portal, navigable product demos and browser-oriented quality
  sensors.
- Manual TestPyPI publishing workflow with OIDC and Dependabot updates for uv and
  GitHub Actions.

### Changed

- Provider integrations now install the `/tramuri` or `$tramuri` product skill and
  keep the product workflow gate aligned across Codex, Claude Code and Gemini CLI.
- Output, reports and persistent state use the `tramuri_output/` project directory.

## [0.13.1] - 2026-11-07

### Fixed

- Provider installation now adds a managed Tramuri identity/workflow block to
  `AGENTS.md`, `CLAUDE.md` or `GEMINI.md` without replacing existing instructions.
- The main `tramuri-pm` skill now explains how to identify Tramuri, start a
  workflow and report the next action.
- Updates and uninstall remove only Tramuri's managed instruction block.
- Provider-native hooks now record deterministic checkpoints in SQLite and gate
  mutating product work until a PM run is completed, subject to each provider's
  trust and hook coverage rules.

## [0.13.0] - 2026-11-07

### Added

- Primeira implementação Python-first do CLI, core local e persistência SQLite.
- Workflow determinístico de Product Management para POC, MVP e Enterprise.
- Adapters Codex, Claude Code e Gemini CLI com capability registry e instalador
  local idempotente, com manifest, hash, backup e preservação de conflitos.
- Relatório HTML offline e protótipo navegável com a identidade visual existente.
- Testes de contrato, integração, segurança, cobertura e tipagem.
- Workflow de CI e publicação PyPI preparado para releases por tag e Trusted
  Publishing OIDC.

### Changed

- O produto não exige Rust, Node.js, Bun, Docker, banco externo ou backend de LLM.
- O harness de construção e o Spec Kit ficam em `tramuri-harness-kit/`.

## [0.12.1]

Versão anterior publicada no PyPI. O histórico detalhado permanece no Git para
referência, mas a linha ativa do produto é Python-first a partir de 0.13.0.
