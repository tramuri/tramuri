---
name: tramuri-missions
description: Define, inspect, and resume Tramuri Missions and their versioned Success Contracts.
---

<!-- tramuri:managed-skill:v1 -->

# Work with Tramuri Missions

Use this skill when the user wants to turn a product or customer objective into a Mission, clarify
success criteria, or resume previously recorded work.

- Start from the user's objective in natural language; do not ask them to issue CLI commands. When
  the objective is clear but acceptance criteria are not, create an exploratory Mission with an
  empty Success Contract and refine criteria during Discovery. Ask only for information that
  materially affects the next action, one focused question at a time. A Mission does not require
  Git, source code, an agent, or a scan.
- At a new Mission, establish the work perspective (Product Engineer, FDE, another perspective, or
  no preference). Infer it only when the request makes it clear, say what you inferred, and allow
  correction. Record the Mission perspective and its source (`explicit`, `current_intent`,
  `profile`, or `inferred`) separately from the Execution actor. Reuse a recorded perspective on
  resume and switch only when asked or when the work clearly changes. The perspective is not the
  executor's identity.
- For a new, substantive objective, prefer `tramuri start --goal "..." --perspective ...` so an
  exploratory Mission, perspective source, and planned Discovery activity are recorded together.
  Current explicit intent can override the configured default. Ask one concise question if routing
  remains ambiguous. Routing is a suggestion from a small deterministic phrase set; do not claim
  the Core understands arbitrary natural language.
- Keep planned activity separate from Context Readiness. Starting Discovery leaves readiness
  `not_assessed`; never promote it automatically. Avoid one Execution per conversational question.
- Update a Mission's current focus with `tramuri mission workflow activity --mission <id> --activity
  <key>` when its phase materially changes. This preserves the work perspective and its provenance;
  it does not complete an Execution or assess readiness.
- Use `tramuri mission list` to discover Missions. If more than one could apply, ask the user to
  select one; never choose an arbitrary Mission.
- If a relevant older Mission has no perspective recorded, do not invent a user selection. When its
  objective and the current request clearly imply a work perspective, say the suggestion and record
  it with `tramuri mission workflow set --mission <id> --perspective <value> --source inferred
  --activity <first-activity>`. Leave it unrecorded or ask one short question if still ambiguous.
- Use `tramuri mission show --mission <id>` for its current contract and
  `tramuri resume --mission <id>` to recover recorded context.
- When continuing a task, perform this sequence without asking the user to issue each command:
  inspect status/list, select the relevant Mission safely, resume it, do the authorized work, then
  record the actual Execution, Evidence, and criterion assessments supported by the work. Ask only
  when the Mission is ambiguous or a material decision is missing.
- Record the actual executor using `actor_kind` and `actor` (for example, `agent` / `Codex` or
  `human` / a name the user supplied). Do not label Codex as a human or treat the Product Engineer
  or FDE perspective as the actor.
- Preserve prior contract revisions. Do not rewrite criteria based on assumptions; ask the user to
  confirm revisions.
- Knowledge and Capabilities are Mission-relevant only when explicitly linked by the user or through
  a command they authorized.
- At the end, summarize the recorded work, actor, outcome state, and next concrete action or the
  exact decision needed from the user. Do not end with a generic “what next?” when the workflow
  already has a clear next step.

For Mission lifecycle, contract revisions, and outcome semantics, read the installed Tramuri skill's
`references/missions-and-outcomes.md`.
