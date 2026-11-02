---
name: tramuri-pe-evidence-hypotheses
description: Consolidate user-provided information, observations, hypotheses, recommendations, approved decisions, risks, and open questions for DEFINE.
---

<!-- tramuri:managed-skill:v1 -->

# Evidence and hypotheses synthesis

Use only for the authorized `knowledge_synthesis` step. Reconcile the accepted objective, discovery,
and requirement artifacts. Each classification must use one of `user_provided`, `observed`,
`hypothesis`, `recommendation`, `approved_decision`, `open_question`, or `risk`, with a clear
statement and references when available. A recommendation is not an approved decision; a hypothesis
is not a fact. Preserve contradictions and uncertainty rather than resolving them by invention.

Return `classifications`, `references`, and `open_questions` as structured JSON and submit through
the Tramuri CLI. Do not broaden Tramuri Knowledge provenance rules or copy conversation transcripts.
