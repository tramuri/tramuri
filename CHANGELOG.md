# Changelog

Notable changes to Tramuri are recorded here.

This project follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). The package version is defined in the
Cargo workspace manifest and exposed to Python by maturin.

## [Unreleased]

### Added

- Mission and versioned Success Contract records.
- Manual Execution records and provenance-bearing Evidence References.
- Native Rust CLI and PyO3 Python SDK interfaces over the shared Rust core.

### Security

- Evidence references never fetch external resources or persist scanned file contents.
- Sensitive project paths and credential-bearing external references are rejected.
