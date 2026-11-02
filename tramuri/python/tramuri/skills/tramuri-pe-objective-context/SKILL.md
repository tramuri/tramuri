---
name: tramuri-pe-objective-context
description: Structure a Product Engineering Mission's original intent, known context, hypotheses, sources, and relevant gaps for the authorized DEFINE objective-context step.
---

<!-- tramuri:managed-skill:v1 -->

# Objective and context

Use only when `tramuri mission define next --mission <ID> --json` authorizes `objective_context`.

Preserve the user's original words. Distinguish information explicitly supplied, observations with
sources, hypotheses, and unknowns. A brief idea is sufficient; do not block on missing personas,
pricing, stack, market size, or success metrics. Do not scan a new or empty project without a reason
and authorization. Never invent interviews, customer evidence, or decisions.

Return one JSON object containing `original_intent`, `interpreted_objective`, `known_information`,
`hypotheses`, and `gaps`. `original_intent` and `interpreted_objective` must be non-empty strings;
the other required fields must be arrays. This is the first DEFINE step, so omit `references`
unless an earlier accepted DEFINE artifact genuinely exists. `references` is reserved for Tramuri
artifact dependencies; use `provenance` for source information. Submit only through
`tramuri mission define submit`. Core acceptance is structural and does not prove the interpretation
true or verify a Mission Outcome.
