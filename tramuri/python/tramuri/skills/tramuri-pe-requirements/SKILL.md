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

Return the Core-required JSON fields. Explicit empty arrays are allowed for genuinely absent
dependencies or gaps. Submit through `tramuri mission define submit`; Core acceptance is structural,
not human approval of the requirements.
