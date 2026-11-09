---
name: tramuri-experience
description: Generate and validate a complete offline product experience from product artifacts.
---

# Product Experience

Build a navigable offline experience with real links, local interactions and
traceability to the product definition. Select screens from the inferred domain;
do not reuse an unrelated journey. Include marketing, customer and admin
surfaces only when justified by the product model.

Quality: deterministic sensors must check links, screens, interactions, domain
coherence, accessibility signals and unresolved placeholders.

## When to use
Build an offline navigable product experience after the product definition has
enough scope and journeys to guide the surfaces.

## When not to use
Do not generate unrelated marketing, customer or admin screens, or present
simulated integrations as real services.

## Inputs
Product brief, scoped capabilities, user journeys, product UX decisions,
traceability links and validation constraints.

## Process
Create the smallest coherent connected experience that represents the product's
primary journey and justified surfaces. Use local interactions and labeled
synthetic data; run deterministic sensors and repair concrete failures.

## Output contract
Return the experience path, screen-to-requirement mapping, validation receipts,
simulation limits and unresolved issues.

## Quality checklist
- [ ] Main journey and navigation are usable offline.
- [ ] Screens map to product scope and decisions.
- [ ] Simulated data and integrations are labeled.
- [ ] Sensor findings and unresolved issues are reported honestly.
