# Tasks: Premium Product Reports

**Input**: [spec.md](./spec.md), [plan.md](./plan.md)  
**Tests**: Add behavior checks before implementation and run the repository test suite.

## Phase 1 — Contract and reusable presentation model

- [x] T001 Add tests for safe source URLs and explicit-only relationship edges.
- [x] T002 Implement the report presentation projection without changing storage.
- [x] T003 Add report integration assertions for bilingual labels, theme controls,
  journey state, source safety and preserved report content.

## Phase 2 — Visual system and report experience

- [x] T004 Create shared report base, macros, embedded CSS tokens and JavaScript.
- [x] T005 Rebuild the report around overview, PDLC journey, product
  intelligence, relationships, execution and outcomes.
- [x] T006 Implement language/theme controls, responsive layout, focus states,
  reduced motion and print styles.
- [x] T007 Keep all generated content escaped and add safe HTTP(S)-only links.

## Phase 3 — Harness and documentation

- [x] T008 Write and link the report design system in `docs/`.
- [x] T009 Keep `tramuri-report-design` and external-skill selection in the
  harness; route report work there and verify the Tramuri package excludes them.
- [x] T010 Record verified evidence and known limits in
  [verification-evidence.md](./verification-evidence.md).

## Phase 4 — Verification and convergence

- [x] T011 Run focused and full Python tests and configured static checks.
- [x] T012 Validate real generated HTML in browser at desktop and mobile sizes;
  test navigation, language, theme, accessibility basics and print export.
- [x] T013 Capture and inspect desktop/mobile screenshots; fix visible issues.
- [x] T014 Run harness validation, Impeccable detector and Web Interface
  Guidelines review; update this task list to match actual evidence.
