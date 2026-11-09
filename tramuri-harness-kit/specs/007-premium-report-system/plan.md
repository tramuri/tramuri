# Implementation Plan: Premium Product Reports

**Feature**: 007-premium-report-system  
**Spec**: [spec.md](./spec.md)

## Summary

Replace the single-page generic report with a self-contained, bilingual product
intelligence document. Reuse the existing persisted run projection and brand
assets; derive the journey and evidence graph from stored state without changing
workflow semantics. Put shared presentation tokens/components in report
templates, and keep the design practice in the repository harness only.

## Technical Context

- Python 3.12+, current Jinja2, no new runtime dependency.
- Existing generator: `src/tramuri/reports/generator.py`.
- Existing data source: `Database.run_details()` and `build_status()`.
- Existing package asset: `src/tramuri/assets/logo.png`.
- Rendering: Jinja templates embedded into one offline HTML file.
- Validation: pytest, Playwright when available, browser screenshots, Impeccable
  detector, and the Web Interface Guidelines review.
- Persistence: no SQLite schema, migration, Markdown or JSON contract change.

## Architecture

1. Add a provider-neutral report presentation projection for safe external URLs
   and deterministic relationship nodes/edges derived only from stored IDs,
   dependencies and source references.
2. Split the HTML into a shared base, reusable macros, centralized design
   tokens/styles, and a report-specific content template. The generator embeds
   all CSS and JavaScript during rendering.
3. Build the report around an executive overview, truthful PDLC journey,
   product intelligence, evidence/decision relationships, outcomes and
   traceability. Retain complete tables/lists for underlying records.
4. Add a bilingual UI dictionary and semantic theme controls in small
   dependency-free JavaScript. Product-authored content remains as stored.
5. Document the approved visual system in `docs/design/report-design-system.md`
   and add a harness-only skill routed by report-design tasks. External
   development skills are selected by harness policy; the Tramuri runtime does
   not discover, list or load them.

## Compatibility and Risk Controls

- Keep `ReportGenerator.generate()` return keys and file names unchanged.
- Keep prototype generation and archive behavior unchanged.
- Preserve autoescaping; reject unsafe URL schemes in clickable source links.
- Keep graph layout deterministic and provide a text relationship list.
- Provide an explicit empty state for missing data; do not fill charts with
  invented metrics.
- The harness skill is selectively unignored and versioned; installed external
  harness skills remain ignored and are not packaged.

## Verification

Run focused report/security tests, the full pytest suite, lint/type checks if
configured, harness validation, browser interaction checks at desktop and
mobile widths, print-to-PDF, screenshots, and visual review. Record unavailable
checks as not run with the reason.
