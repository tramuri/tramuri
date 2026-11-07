# Changelog

Mudanças notáveis do Tramuri seguem [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
e [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
