# Changelog

Mudanças notáveis do Tramuri seguem [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
e [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
