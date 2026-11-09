# ADR-0001: Generate product demos from an explainable experience contract

- **Status:** Accepted
- **Date:** 2026-11-08
- **Owners:** Tramuri maintainers

## Context

The existing Product Experience flow generated a few fixed web archetypes. It
could not represent products whose primary experience is a phone, terminal,
desktop application, API, data pipeline or connected device. A new demo flow
would duplicate PDLC state and create conflicting artefacts. The runtime must
remain local-first, offline-capable and usable without Node.js or a hosted model.

## Options considered

1. **Keep the fixed web archetypes.** Lowest maintenance cost, but it cannot
   represent different product environments or a mixed product journey.
2. **Generate a fresh web app per request with a model/runtime.** Flexible, but
   it duplicates discovery, adds runtime dependencies and can imply
   integrations or maturity that do not exist.
3. **Classify persisted PDLC context and render a versioned spec through
   registered local templates.** Adds a category registry and tests, while
   keeping generation deterministic, traceable and extensible.

## Decision

Use an explainable context-weighted `ProductExperienceClassifier`, a
`tramuri.demo.v1` intermediate specification, a category `DemoTemplateRegistry`
and local renderers, all called by the existing `ProductDemoBuilder` in the
current report/Product Experience flow. Persist classification evidence and
manual corrections. Mark weak classifications as ambiguous and use a
contextual concept template. Simulate interactions with local state; never run
arbitrary commands or contact live services from a demo.

## Consequences

Positive: phone, web, CLI, desktop, API, pipeline, IoT, hybrid and contextual
surfaces have dedicated representations; demos can be re-rendered from a
versioned spec; screen and action sources remain traceable; no runtime library
or Node.js dependency is added.

Costs: each template needs responsive and accessibility review; category
corrections and new categories need registry coverage; deterministic cues can
remain ambiguous and must expose their evidence. Optional browser validation
may be unavailable, in which case it stays `not_run`.

Compatibility: existing Product Experience entry points, report keys, offline
artefacts and the report design system remain in place. Demos add
`demo_spec.json`, traceability details and category metadata. Reversal can
restore the earlier renderer while leaving existing reports and run state
readable.

## Evidence and verification

The engine tests use independent mobile finance, developer CLI, project SaaS,
data API, hybrid field product, desktop, pipeline, IoT and contextual-other
fixtures. See `tests/integration/test_product_demo_engine.py`,
`docs/design/product-demo-system.md` and the harness skill
`.agents/skills/tramuri-product-demo-engine/`.
