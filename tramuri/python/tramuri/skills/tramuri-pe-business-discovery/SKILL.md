---
name: tramuri-pe-business-discovery
description: Analyze the product problem, audience, buyer, value proposition, and business objectives for the authorized DEFINE discovery step.
---

<!-- tramuri:managed-skill:v1 -->

# Business discovery

Use only when the Core returns `business_discovery` from `tramuri mission define next`. Build on the
accepted `objective_context` artifact listed in `accepted_inputs`. Use non-empty strings for
`problem` and `value_proposition`; use arrays for `target_audience`, `business_objectives`,
`uncertainties`, and `references`. Empty arrays are structurally allowed when they explicitly mean
that no items are known. `references` must include the accepted predecessor, for example
`{"step":"objective_context","revision":1}` using its actual revision from `accepted_inputs`.
These references point to Tramuri artifacts; put external source information under `provenance`.

Identify the problem, target audience, business objectives, value proposition, business rules, and
uncertainties. Label unsupported audience, buyer, market, pricing, and business-model statements as
hypotheses. Do not claim customer or market validation without direct evidence. Do available useful
work before asking a material question.

Submit with `tramuri mission define submit`; do not mark a step complete yourself. Structural
validation is not evidence that a business hypothesis is true.
