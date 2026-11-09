# ADR-0005: Capability-based product skills without a PM persona

## Status

Accepted

## Context

Tramuri's public workflow is named Product and its outcomes are product
artifacts. The installed skill set already contains focused capabilities, but
many skills state only a short responsibility and the entry skill does not
explain how to select them. Provider agent definitions also describe broad
delegation without a consistent routing contract. A profession-shaped PM
persona would not solve these gaps and conflicts with Tramuri's product-first
experience.

## Decision drivers

- Keep Tramuri organized around product intent, decisions and verifiable
  artifacts rather than a professional role.
- Make skill activation distinct and useful across Codex, Claude Code and
  Gemini CLI.
- Reuse SQLite checkpoints, evidence and prior artifacts instead of adding
  another memory store.
- Preserve few-question autonomy, truthful provider capabilities and inline
  fallback.
- Avoid importing a large external skill catalog or external implementation.

## Considered options

### Option 1: Add a PM agent and a large set of role-specific subagents

Rejected. This adds a profession persona and increases the number of
definitions without proving that the provider invokes them.

### Option 2: Keep each skill as a short responsibility statement

Rejected. The current templates leave activation boundaries, inputs, expected
outputs and quality checks implicit, making correct selection and handoff less
reliable.

### Option 3: Define contracts for existing product capabilities

Accepted. The existing skills receive distinct triggers, non-triggers, inputs,
bounded steps, output contracts and quality checks. The existing Tramuri entry
skill routes only the capabilities needed by the user's intent and current
persisted state. Providers use native invocation only when verified; otherwise
the work runs inline.

## Decision

Product capabilities remain the units of selection and execution. No PM role,
persona, agent or public command is introduced. Each distributed skill carries
its own concise contract. The entry skill maps product situations to the
existing skill names and expected artifacts, consults status and prior
artifacts first, and selects the smallest useful set.

This decision covers Tramuri-authored product capability skills only.
Development skills such as frontend design, testing and documentation belong
to the repository harness. The distributed CLI MUST NOT discover, select or
list those harness skills, and their definitions MUST NOT enter product
templates or package data.

A native subagent or skill invocation is reported only when it actually occurs.
Provider files alone do not prove invocation, and no durable invocation record
is implied without an observable provider receipt. An inline fallback is valid
work but is not reported as delegation. Reviewers remain read-only. SQLite
remains the durable source of run state and knowledge.

## Consequences

### Positive

- Users can ask Tramuri for a focused product task without invoking the entire
  catalog.
- Skill selection is grounded in visible triggers and persisted workflow state.
- Skill outputs have explicit handoffs and quality criteria.
- Provider limitations are represented honestly.
- No second project-memory file or new runtime dependency is required.

### Negative

- Skill descriptions and contracts require maintenance as the product workflow
  changes.
- A clear skill contract does not prove that a provider activated or executed
  the skill; live provider behavior still requires separate evidence.

## Evidence and verification

The decision is based on the local review of the 19 Tramuri product skills and
the temporary engineering review of product-on-purpose/pm-skills v2.35.0.
Structural and provider behavior checks are recorded in Spec Kit feature
006-product-skill-contracts. The external project is used as a design
reference only; its skill text, agents, hooks and outputs are not copied.

## Related decisions

- ADR-0002: Separate the Tramuri core from provider harnesses.
- ADR-0003: Use product as the public workflow boundary.
- ADR-0004: Use evidence-backed research packs and product experience projections.
- ADR-0006: Keep SQLite canonical while generating a readable project-memory projection.
