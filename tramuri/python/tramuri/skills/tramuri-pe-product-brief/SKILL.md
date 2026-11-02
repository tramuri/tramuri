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

Return the required JSON fields and submit through `tramuri mission define submit`. The Core validates
structure and dependencies; the brief remains a proposal where the source artifacts are uncertain.

After the Core accepts the brief, assess its semantic consistency against
`tramuri.product-brief-rubric.v1`: goal alignment, target consistency, evidence/assumption separation,
success-criteria fit, requirements traceability, decision authorization, and internal consistency.
Use `pass`, `concern`, or `not_assessed` per criterion, cite the exact accepted artifact fields, and
state limitations. Submit the assessment with `tramuri mission define review --assessment-file
<assessment.json> --evaluator <declared-host-or-reviewer> --json`. The declared evaluator is not
authenticated. Do not present the result as a Core semantic verification, customer validation, or
Mission Outcome. If the host cannot conduct this review, leave it unsubmitted and say so.
