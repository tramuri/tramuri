# Changelog

Notable changes to Tramuri are recorded here.

This project follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). The package version is defined in the
Cargo workspace manifest and exposed to Python by maturin.

## [Unreleased]

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
