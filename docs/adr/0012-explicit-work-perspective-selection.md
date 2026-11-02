# ADR-0012 — Explicit Work Perspective Selection Before New Missions

- **Status:** Accepted — maintainer approval in chat on 2026-11-02
- **Date:** 2026-11-02
- **Related:** ADR-0009 Guided Mission Lifecycle; ADR-0010 Native Workflow Gate;
  `specs/006-guided-mission-lifecycle/`

## Context

Product and customer-integration requests can be classified by an agent or a small deterministic
phrase router before the person has said how they want the work conducted. That can silently start a
Mission under the wrong perspective. Intent inference is useful as a suggestion, but it must not
stand in for the person's choice.

## Decision

1. Every newly created Mission requires an explicit `product_engineer`, `fde`, or `mixed` perspective.
   Current-intent inference and saved Work Profile defaults are suggestions only.
2. A new Mission requested from an interactive terminal shows a single-choice keyboard selector with
   short descriptions for those three perspectives. The selected value is persisted in the existing
   Mission workflow row in SQLite. No extra file or schema migration is required.
3. A non-interactive CLI or SDK caller must supply the selected perspective. Without one, Core returns
   the existing selection-required error before initializing the database or creating records.
4. An exact, unique open Mission match is resumed with its stored perspective. It is not reclassified
   on resume. Different Missions can use different perspectives.
5. Installed agent instructions ask the user in chat when the perspective was not already explicit,
   then call `tramuri start` with the selected value. A host hook only enforces its documented tool
   boundary; it cannot authenticate who supplied the value or prevent arbitrary assistant prose.
6. Perspective remains a description of the Mission's work, not a person, role permission, or
   execution actor. Existing `other` records and the public enum remain readable for compatibility;
   the new Mission selector and CLI start flag expose only the three approved choices.

## Consequences

- Creating a new Mission through Rust Core/Python without a perspective now returns the selection
  required error, even when an intent phrase or profile default could suggest one. This is a public
  behavioral change; function signatures and SQLite schema remain unchanged.
- Intent routing remains available for suggestions and diagnostics but cannot create a Mission.
- Existing Missions, stored perspectives, profile files, reports, and workflow history are not
  rewritten.
- A real host runtime test is still required to verify that an interactive terminal selector is
  reachable through each agent's command tool. Hosts without interactive terminal support must use
  their normal chat for the explicit choice and invoke the non-interactive CLI with that answer.
