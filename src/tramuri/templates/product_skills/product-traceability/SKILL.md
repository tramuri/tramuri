---
name: tramuri-traceability
description: Keep product decisions, evidence, screens and validations connected.
---

# Product Traceability

Map each important product claim or decision to its source artifact, evidence
classification, screen/interaction and validation receipt. Highlight missing
links and stale dependencies. Never imply that a generated artifact proves a
user outcome.

## When to use
Check whether product decisions and claims remain connected to their evidence,
artifacts, experience screens and validation receipts.

## When not to use
Do not create trace links for every sentence or claim that a link proves a user
outcome.

## Inputs
Product artifacts, decisions, evidence references, screens and validation
receipts from the current run.

## Process
Map material claims and decisions to their source, classification, dependent
artifact or screen, and verification receipt. Identify missing or stale links.

## Output contract
Return a traceability map, uncovered claims, stale dependencies and specific
repair recommendations.

## Quality checklist
- [ ] Material claims retain source and evidence classification.
- [ ] Screen and artifact links refer to real generated paths.
- [ ] Missing validation is explicit.
- [ ] No outcome is inferred from artifact existence alone.
