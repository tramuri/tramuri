# Changelog

Notable changes to Tramuri are recorded here.

This project follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). The package version is defined in the
Cargo workspace manifest and exposed to Python by maturin.

## [Unreleased]

## [0.6.0] - 2026-11-01

### Added

- Persistent local Work Profiles, transparent deterministic intent routing, and `tramuri start` for exploratory Missions with a planned Discovery activity separate from Context Readiness.
- A Product Engineering skill that frames product work, performs useful discovery before material questions, and labels rather than persists unsupported hypotheses.
- Product Engineer Missions begin in `product_framing`; planned focus can advance independently through the Rust Core, CLI, Python SDK, Resume, and reports.
- Installation instructions for Codex, Claude Code, Cursor, Gemini CLI, and GitHub Copilot direct agents to start/resume Tramuri before other project-work tools, then continue the requested task.
- Mission workflow metadata and perspective-change history, visible through Core, CLI, Python SDK, Status, Resume, and the local reports.
- Explicit transitions from in-progress Executions to terminal states, with end timestamps and terminal-state protection.
- Report wording that distinguishes human review from external evidence availability, plus session guidance to open the report no more than once.

### Changed

- SQLite schema v9 adds additive Mission workflow and work-perspective history tables. Status and report JSON advance to v3; downstream Rust struct literals for `MissionStatus` must now supply `workflow`.
- The CLI now provides `tramuri profile show|set`, `tramuri mission workflow set|activity`, and `tramuri execution update`.
- Mission workflow perspective updates accept an explicit provenance source, including `inferred` for transparently classified legacy Missions.

## [0.5.0] - 2026-11-01

### Added

- Activity-based Mission context readiness with append-only assessments, explicit applied checks, gaps, blockers, accepted uncertainties, and freshness detection.
- Exploratory Missions without a Success Contract criteria list; their outcomes remain pending until criteria are added and reviewed.
- Focused assistant skills for Tramuri setup, discovery, Missions, and data integrity.
- A guided first-use flow that lets users describe outcomes and select a Product Engineer, FDE, or other work perspective in natural language.
- Assistant guidance that starts an exploratory Mission from a natural-language objective and refines criteria during Discovery, plus end-to-end Product Engineer and FDE tutorials in English and Brazilian Portuguese.
- End-of-session refresh and opening of local reports after coherent Mission updates, with an explicit `tramuri status --open` fallback.
- A bilingual, responsive, offline workspace report in English and Brazilian Portuguese, with clearer Mission, criterion, execution, evidence, capability, and outcome sections.
- Optional per-Execution work perspective (`product_engineer`, `fde`, or `other`), displayed separately from the recorded actor in local reports.
- README guidance for updating Tramuri with `uv tool upgrade` or `pipx upgrade` after a release.

### Changed

- Assistant integrations now guide the complete Mission workflow, distinguish work perspective from execution actor attribution, and clarify that project scans are optional.
- Installation instructions explain what to do next without requiring users to memorize Tramuri commands.
- SQLite schema v7 adds an optional Execution work perspective through an additive migration; existing Execution history is preserved.
- SQLite schema v8 adds append-only readiness assessments. Status and report JSON contracts advance to v2; the public Rust `MissionStatus` gains a readiness field, so downstream struct literals must supply it.
- The public Rust `Execution` and `ExecutionInput` structs gain `work_perspective`; downstream Rust struct literals must supply this field. The Python SDK parameter is optional and appended to preserve existing calls.

## [0.4.0] - 2026-11-01

### Added

- Declarative discovery of agent capabilities in the authorized workspace, including supported agent, skill, instruction, hook, and MCP configuration formats. Discovery records provenance and does not execute discovered resources.
- Workspace status and local HTML, Markdown, and JSON reports, plus capability queries through the Rust Core, CLI, and Python SDK.
- Explicit Mission links to Knowledge and declared Capabilities, with provenance and freshness information available during Mission resume.

### Changed

- SQLite schema v6 adds transactional Mission–Knowledge and Mission–Capability link tables while preserving existing records.
- Mission resume JSON now includes explicitly linked context. Workspace status does not expose linked Knowledge content.
- The public Rust error enum adds Capability relationship errors; downstream exhaustive matches may need to handle the new variants.

## [0.3.1] - 2026-10-31

### Fixed

- Assistant integration install, status, and uninstall now recognize managed skill front matter with
  either LF or CRLF line endings, including Windows checkouts.

## [0.3.0] - 2026-10-31

### Added

- A `tramuri` console command distributed with the PyO3/maturin package and backed by the shared
  Rust CLI dispatcher.
- Project instructions and a Tramuri skill for Codex, Copilot, Claude Code, and Gemini CLI, with
  safe install, status, and uninstall commands.
- Versioned local discovery report output under `.tramuri/`.
- Rust Core, CLI, and Python SDK read operations for retrieving Mission executions and evidence
  references without following or revalidating stored locators.

### Changed

- Local SQLite state is organized under `.tramuri/database/`; existing database files are migrated
  from the previous location without replacing a conflicting database.
- The Python package source now lives in the top-level `tramuri/` directory.
- The public Rust `TramuriError` enum adds `EvidenceNotFound`. Downstream exhaustive matches must
  handle this new variant; this is a source-compatibility consideration for Rust consumers.

## [0.2.0] - 2026-10-31

### Added

- Criterion assessment history and deterministic Mission outcome aggregation.
- CLI and Python SDK operations for outcome assessment and retrieval.

### Changed

- SQLite schema v5 adds assessment tables through an additive migration; existing Mission,
  Execution, and Evidence records are preserved.

## [0.1.0] - 2026-10-31

### Added

- Local project discovery with `tramuri init` and `tramuri scan`.
- SQLite inventory and source-backed persistent knowledge with provenance.
- Mission and versioned Success Contract records.
- Manual Execution records and provenance-bearing Evidence References.
- Native Rust CLI and PyO3/maturin Python SDK over the shared Rust core.

### Security

- Project scanning does not execute repository files and excludes sensitive paths by default.
- Evidence references never fetch external resources or persist scanned file contents.
- Sensitive project paths and credential-bearing external references are rejected.
