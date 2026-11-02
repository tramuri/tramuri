---
name: tramuri-pe-requirements
description: Produce traceable candidate requirements, constraints, dependencies, acceptance criteria, and success criteria for the authorized DEFINE step.
---

<!-- tramuri:managed-skill:v1 -->

# Requirements and criteria

Use only when `requirements` is the authorized step. Derive candidates from accepted prior
artifacts. Keep functional and non-functional requirements, constraints, dependencies, acceptance
criteria, candidate success criteria, and gaps distinct. Do not silently revise an approved Success
Contract. Mark each proposal as candidate until the user approves it. Use stable IDs and references
where possible; do not invent technical constraints or integrations.

All required fields (`functional_requirements`, `non_functional_requirements`, `constraints`,
`dependencies`, `acceptance_criteria`, `success_criteria`, `gaps`, and `references`) are JSON arrays.
Empty arrays are structurally accepted when they explicitly represent no identified items. Read the
current `business_discovery` revision from `accepted_inputs` returned by `define next` and include it
in `references`, for example `{"step":"business_discovery","revision":1}`. References are
Tramuri artifact dependencies; put external source information under `provenance`.

Submit through `tramuri mission define submit`; Core acceptance is structural, not human approval of
the requirements.
