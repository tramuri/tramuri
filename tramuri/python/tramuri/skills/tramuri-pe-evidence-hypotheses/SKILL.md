---
name: tramuri-pe-evidence-hypotheses
description: Consolidate user-provided information, observations, hypotheses, recommendations, approved decisions, risks, and open questions for DEFINE.
---

<!-- tramuri:managed-skill:v1 -->

# Evidence and hypotheses synthesis

Use only for the authorized `knowledge_synthesis` step. Reconcile the accepted objective, discovery,
and requirement artifacts. The required top-level fields `classifications`, `references`, and
`open_questions` are JSON arrays. Each classification must be an object with a non-empty `statement`
and a `kind` exactly equal to one of `user_provided`, `observed`, `hypothesis`, `recommendation`,
`approved_decision`, `open_question`, or `risk`. Read the current `requirements` revision from
`accepted_inputs` returned by `define next` and include it in `references`, for example
`{"step":"requirements","revision":1}`. References are Tramuri artifact dependencies; put
external source information under `provenance`.

A recommendation is not an approved decision; a hypothesis is not a fact. Preserve contradictions
and uncertainty rather than resolving them by invention.

Return `classifications`, `references`, and `open_questions` as structured JSON and submit through
the Tramuri CLI. Do not broaden Tramuri Knowledge provenance rules or copy conversation transcripts.
