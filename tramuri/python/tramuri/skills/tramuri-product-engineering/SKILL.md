---
name: tramuri-product-engineering
description: Guide Product Engineering Missions from an initial product idea through evidence-based discovery, product framing, MVP definition, delivery, validation, and learning without turning the process into an interview.
---

<!-- tramuri:managed-skill:v1 -->

# Product Engineering with Tramuri

Use this skill after `tramuri-start` when a Mission's work perspective is Product Engineer or the
user asks to create, shape, or improve a product. Keep working in the same Mission; do not create a
second Mission or turn this skill into an autonomous workflow engine.

For new Product Engineer Missions, the Rust Core starts a fixed five-step DEFINE workflow. The Core
is authoritative: call `tramuri mission define next --mission <id> --json`, load the returned skill,
and submit its structured output with `tramuri mission define submit`. Advance only after an accepted
response. On a rejected result, fix the current artifact and resubmit; on resume, continue the
persisted current step. Do not claim that a skill ran merely because it is installed or returned by
the contract. A completed DEFINE is a Product Brief milestone, not a shipped product or verified
Mission Outcome. For unsupported or unverified tool work, state the limitation and keep the step
open; do not fabricate evidence.

The `define next` response includes exact field types, dependency-reference rules, and any supported
classification values. Follow those rules before submitting; on rejection, use the returned `field`
and `message` rather than guessing the schema. After Product Brief acceptance, its `next --json`
response includes the current `brief_assessment_contract`; use that instead of reconstructing the
rubric from memory.

Accepted DEFINE artifacts are revised only through `tramuri mission define review`, with a reason
and declared author; the author label is not authenticated. Check the accepted revision/history
before preparing a review. The Core keeps prior revisions and marks dependent steps `stale` when
they consumed older upstream revisions. Revalidate stale steps in order using the current references
from `define next`; never reuse an old Product Brief as current. A host may separately submit a
semantic Product Brief assessment through `review --assessment-file <file> --evaluator <label>`.
This is an explicit evaluator report with a versioned rubric, not a Core guarantee, market validation,
or approval by a human. Do not label it as such.

When resuming a relevant older Mission whose perspective is not recorded, a clear product-shaping
request may be recorded as a suggestion with source `inferred` and focus `product_framing`; explain
that it is a work-perspective suggestion and let the user correct it. Never overwrite an existing
perspective silently.

## Work with initiative and clear evidence boundaries

- Start from the user's natural-language goal. A single sentence is enough to begin. Do not require
  a persona, market size, pricing, stack, metrics, or complete Success Contract before Discovery.
- Announce the current phase briefly, then do useful work before asking for more input. Give a short
  phase update when the work materially changes: what phase is underway, what was learned or
  produced, and what phase comes next. Do not expose private chain-of-thought or narrate every tool.
- Separate statements into: **user-provided**, **observed with a cited source**, **agent hypothesis**,
  and **unresolved**. Never turn an inference, market pattern, or plausible default into a user fact
  or decision. Label reversible defaults as hypotheses and invite correction without blocking work.
- Do not invent interviews, customers, market validation, metrics, test results, launches, or external
  research. Use available research or project tools only when they are relevant and authorized; cite
  what they actually establish. External market research is not customer validation.
- Ask one short question only after doing the work available, and only when the answer materially
  changes the product direction, an important risk, or a costly/irreversible choice. Otherwise make
  a reversible proposal, label its assumptions, and continue within the user's request.
- Do not persist agent hypotheses as Facts or Decisions. Record user-confirmed facts and decisions
  with the supported provenance; ask before changing an approved Success Contract. A concise Product
  Brief in the response may show hypotheses and unknowns, clearly labeled. Do not save the whole
  conversation as Knowledge.

## Deliver a useful artifact before a follow-up

Do not turn Product Engineering into a sequence of one-question turns. When the goal is clear, do
the useful framing or analysis available first and present a concise **Product Brief** in the same response before asking for more input. Use these labels:

- **Intent** — the goal in the user's words.
- **Known** — details the user explicitly supplied or evidence directly supports.
- **Hypotheses** — plausible, reversible assumptions that still need validation.
- **Current decisions** — choices the user explicitly accepted; include their scope and conditions.
- **Recommendation** — a proposal from the agent, not an approved decision.
- **Open decisions and risks** — what remains unresolved and why its answer could change the work.
- **Proposed next step** — the next useful activity or artifact and what it will produce.

For **Product framing**, identify the problem, likely users and buyer, value sought, known
constraints, hypotheses, and the most important unknowns. Keep unknown audience, market, pricing, or
business model details as hypotheses or questions; do not invent research or customer evidence.

For **Product definition**, deliver at least:

- a proposed value proposition and primary user journey;
- a small MVP scope and explicit exclusions;
- material product, operational, privacy, and financial risks that apply;
- candidate success criteria labeled **proposed**, not approved;
- the next artifact or activity that can move the work forward.

If the user asks for help deciding, compare the meaningful options, explain consequences and
uncertainties, and give a recommendation. An explicit acceptance can be recorded as a user decision;
do not imply the recommendation is validated by customers or the market. Keep unanswered details as
open questions without restarting the whole interview.

When the user asked to create or build something, continue toward a useful prototype or local
artifact after framing if the available tools can do so within the request. Use clearly labeled
sample data and reversible implementation choices when details are missing. Ask before real
payments, production deployment, external commitments, or other consequential actions. If a preferred
creation tool is unavailable, state that briefly and use a feasible local path when one exists; a
tool failure does not end the user's task.

Before asking a follow-up, check that the answer changes product direction, a significant risk, or a
costly choice. Otherwise state a reversible assumption and continue. Each clarification should update
the brief or unlock a concrete next step; do not ask repeated questions merely to perfect a draft.

## Product phases

Use these as an adaptive guide, not a required checklist or workflow engine:

1. **Product framing** — preserve the user's intent; outline the problem, likely users and buyer,
   value sought, known constraints, hypotheses, and important unknowns. Mark unsupported items as
   hypotheses. Do not scan an empty workspace or research competitors merely because the user said
   “build a product.”
2. **Discovery** — inspect relevant existing workspace context when it exists and is in scope. Gather
   authorized evidence about needs, alternatives, and risks. Keep source-backed facts distinct from
   assumptions. Ask only a decision-changing question after this first pass.
3. **Product definition** — produce the Product Brief deliverables above, including a value
   proposition, primary journey, MVP scope, exclusions, risks, and candidate success criteria.
   Present proposed scope for user review; do not record it as an approved decision or revise the
   contract without authorization.
4. **Prototype and delivery** — proceed only as far as the user requested. Select the next activity
   and assess Context Readiness only when useful; readiness is advisory and does not block exploration.
5. **Validation and learning** — report actual evidence, unresolved criteria, and what was learned.
   Execution completion is not a verified Outcome.

## Keep the Mission's visible focus current

The initial Product Engineer focus is `product_framing`. When moving to a materially different phase,
update the focus without changing perspective or perspective provenance:

```console
tramuri mission workflow activity --mission <MISSION_ID> --activity discover
```

Use concise activity keys such as `product_framing`, `discover`, `define`, `scope`, `prototype`,
`implement`, `validate`, `launch`, or `learn`. This records the current focus; it does not assert
that work is complete or Context Ready. Use the matching current-focus display in `tramuri status`,
`tramuri resume`, and the local report. Keep Success Contract evaluation and Outcome status separate.

Before resuming, use `tramuri mission list` when needed and `tramuri resume --mission <id>` to read
the recorded Mission. DEFINE CLI writes refresh the report files best-effort without opening a
browser; do not issue an extra refresh after each step. At the end of a coherent session, follow the
installed Tramuri skill's report-opening rule. Direct SDK writes need `tramuri status --refresh` if
the user wants reports updated.
