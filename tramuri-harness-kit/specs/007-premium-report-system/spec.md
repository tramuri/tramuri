# Feature Specification: Premium Product Reports

**Feature Branch**: 007-premium-report-system  
**Status**: Complete  
**Input**: Redesign Tramuri's offline product report as a reusable, bilingual, accessible product intelligence experience and make its design practices available to the development harness only.

## User Scenarios & Testing

### User Story 1 — Understand the product outcome (Priority: P1)

As a product builder, I want a clear executive overview and a truthful view of
the workflow so that I can understand what Tramuri concluded, what remains
uncertain, and what action comes next.

**Independent Test**: Generate a report from a persisted run and verify that its
overview, actual stage states, progress, next action, risks and deliverables
match the run details.

### User Story 2 — Explore product reasoning (Priority: P1)

As a reviewer, I want to inspect product framing, research, assumptions,
hypotheses, decisions, evidence, scope, success criteria and execution so that
I can trace claims back to recorded information.

**Independent Test**: Generate reports from both a deterministic local demo and
a run with an imported research pack; verify that absent evidence remains
explicit and relationships are drawn only from stored dependencies or source
references.

### User Story 3 — Read and navigate the report in context (Priority: P1)

As a Tramuri user, I want the report to work offline on desktop and mobile, in
English or Brazilian Portuguese, with accessible navigation, light and dark
themes, and print styling.

**Independent Test**: Open generated HTML using a browser at desktop and mobile
viewports, switch language and theme, follow section and graph links, inspect
keyboard focus and reduced-motion behavior, and export a print-ready PDF.

### User Story 4 — Keep report quality consistent (Priority: P2)

As a Tramuri contributor, I want future reports to reuse a documented design
system and harness skill so report updates keep the Tramuri identity and
evidence rules without distributing development skills to product users.

**Independent Test**: Inspect the versioned design specification, report skill,
and harness routing policy; confirm the report skill lives outside the Python
package and is selected for report work.

## Functional Requirements

- **FR-001**: The HTML report MUST use Tramuri's existing logo and official
  palette, with a distinctive editorial hierarchy and reusable report
  components.
- **FR-002**: The report MUST preserve all currently rendered product,
  research, stage, decision, hypothesis, evidence, validation, checkpoint,
  sensor, knowledge and experience information.
- **FR-003**: The overview MUST show the run's actual semantic state, progress,
  relevant decisions, risks and next action from persisted details.
- **FR-004**: The PDLC journey MUST represent actual persisted stage order,
  status, owner and attempt information. It MUST NOT imply completion from
  generated-file existence alone.
- **FR-005**: The report MUST distinguish recorded evidence, facts, inferences,
  hypotheses, assumptions, decisions and pending validation when those
  classifications are present.
- **FR-006**: A decision/evidence visualization MUST include only explicit
  relationships available from stage dependencies, artifact lineage or source
  references. It MUST describe missing relationships honestly.
- **FR-007**: The generated report MUST remain a self-contained offline HTML
  file with no required external fonts, scripts, image hosts or runtime server.
- **FR-008**: The report MUST provide working English and Brazilian Portuguese
  UI labels, working light/dark theme controls, keyboard-visible focus,
  responsive layouts, reduced-motion support and print/PDF styling.
- **FR-009**: Source-provided product text MUST remain unchanged and HTML
  escaped. External source links MUST allow only HTTP or HTTPS destinations.
- **FR-010**: Markdown, JSON, SQLite schemas and workflow state transitions MUST
  remain unchanged by this presentation feature.
- **FR-011**: The report design specification and `tramuri-report-design`
  skill MUST be versioned in the repository harness and MUST NOT be copied into
  `src/tramuri`, package data or installed product skills.
- **FR-012**: Browser validation MUST exercise generated reports at desktop and
  mobile sizes, verify interactions and capture review screenshots. A missing
  browser runtime MUST be reported as `not_run`, never as a pass.
- **FR-013**: The distributed Tramuri CLI MUST NOT discover, select, list or
  package development-harness skills. Tramuri-authored product capability
  skills remain a separate product feature.

## Non-Functional Requirements

- Report generation adds no mandatory runtime dependency.
- Generated HTML must avoid horizontal page overflow at common mobile widths.
- UI controls use semantic elements and accessible names; status meaning is
  expressed by text and shape as well as color.
- Design tokens and shared templates are centralized for future report types.

## Success Criteria

- **SC-001**: Existing report content and security tests pass unchanged or with
  equivalent stronger assertions.
- **SC-002**: Generated HTML is self-contained and contains no required CDN
  reference.
- **SC-003**: Browser checks pass for language switching, theme switching,
  anchor navigation, print mode, mobile overflow and console/page errors.
- **SC-004**: Screenshots are captured for desktop and mobile review.
- **SC-005**: The harness-only skill and design specification are linked from
  their respective harness and documentation indexes.
- **SC-006**: No workflow, SQLite, research-pack or product-skill contract is
  changed.
- **SC-007**: Product code and CLI contain no runtime dependency on installed
  harness skills; selection policy remains in the development harness.
