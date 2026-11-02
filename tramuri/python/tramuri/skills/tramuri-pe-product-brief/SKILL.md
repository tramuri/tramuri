---
name: tramuri-pe-product-brief
description: Create a traceable Product Brief from accepted DEFINE artifacts, keeping proposals separate from approved decisions.
---

<!-- tramuri:managed-skill:v1 -->

# Product Brief

Use only when the Core authorizes `product_brief`. Synthesize accepted prior artifacts into a concise
brief with objective, problem, audience, value proposition, main journey, requirements, proposed MVP
scope, exclusions, risks, assumptions, approved decisions, candidate success criteria, next action,
and references. Do not present candidates as approved requirements, claim external validation, or
turn DEFINE completion into a Mission Outcome. Describe the next useful activity without starting
BUILD unless the user separately asks.

Use non-empty strings for `objective`, `problem`, `value_proposition`, and `next_action`; all other
required fields (`target_audience`, `main_journey`, `requirements`, `mvp_scope`, `exclusions`, `risks`,
`assumptions`, `approved_decisions`, `candidate_success_criteria`, and `references`) are JSON arrays.
Read the current `knowledge_synthesis` revision from `accepted_inputs` returned by `define next` and
include it in `references`, for example `{"step":"knowledge_synthesis","revision":1}`. References
are Tramuri artifact dependencies; put external source information under `provenance`. Submit through
`tramuri mission define submit`. The Core validates structure and dependencies; the brief remains a
proposal where the source artifacts are uncertain.

After the Core accepts the brief, call `tramuri mission define next --mission <ID> --json` and use
its `brief_assessment_contract`. Fill every required field in `assessment_template`; do not submit its
placeholders unchanged. The contract supplies the current Brief artifact ID/revision, criterion IDs
and guidance, allowed results, limits, and evaluation boundary. The seven criteria are
`goal_alignment`, `target_consistency`,
`evidence_assumption_separation`, `success_criteria_fit`, `requirements_traceability`,
`decision_authorization`, and `internal_consistency`—each must appear exactly once.

Use `pass`, `concern`, or `not_assessed` per criterion and a non-empty rationale of at most 1000
characters. `pass` and `concern` require at least one `references` entry with an accepted artifact
`step`, `revision`, and existing JSON `field`; Product Brief references must use the current Brief
revision. Overall must be `pass`, `concerns`, or `incomplete`; it cannot be `pass` if any criterion is
`concern` or `not_assessed`. Include at most five non-empty limitations, each at most 500 characters,
and use the exact evaluation boundary returned by the contract. Submit with
`tramuri mission define review --assessment-file <assessment.json> --evaluator
<declared-host-or-reviewer> --json`. The evaluator label is not authenticated. This report is a
host-produced semantic assessment, not Core semantic verification, customer validation, or a Mission
Outcome. If the host cannot conduct the review, leave it unsubmitted and say so.
