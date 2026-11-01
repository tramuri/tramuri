---
name: tramuri-start
description: Set up Tramuri in a new or uninitialized local workspace and explain the first useful steps.
---

<!-- tramuri:managed-skill:v1 -->

# Start using Tramuri

Use this skill when a user wants to start work with Tramuri, not only when they ask about setup.
The user can describe the desired outcome in natural language; do not make them learn CLI syntax.
Use a Mission for substantive work with a clear objective, multiple steps, acceptance criteria, or a
handoff. Do not force Mission setup for a simple one-shot question unless the user asks for it.

1. Check whether `tramuri` is installed (`tramuri --version`) and whether the selected workspace is
   initialized (`tramuri status`). Do not run a scan just to start a Mission.
2. If Tramuri is missing, explain the install command for the current platform. If the workspace is
   uninitialized and the user explicitly asked to set up or track work with Tramuri, run
   `tramuri init .`; that request authorizes creating `.tramuri/database/`. Otherwise ask once before
   initializing.
3. Confirm setup with `tramuri status`, then continue in the same interaction. The user may give
   only a natural-language objective; do not make them learn or repeat CLI syntax. Create an
   exploratory Mission with an empty Success Contract when the objective is clear but criteria are
   not. Refine criteria during Discovery instead of blocking the start.
4. At the start of a new Mission, establish the work perspective: Product Engineer, FDE, another
   perspective, or no preference. If the user's request makes it clear, state the inferred choice
   and let them correct it instead of asking. Ask at most one focused question at a time, and only
   when a missing answer materially affects the next action. Record the perspective on each Execution using
   `--work-perspective product_engineer|fde|other`; leave it unset when there is no preference.
   Reuse it when resuming; change it only when the user asks or the work clearly changes.
5. Create the Mission once its objective is clear; required criteria may be added later. A Mission
   does not need Git, source code, a scan, an agent, or a cloud service.

Keep work perspective separate from actor attribution. The perspective describes the professional
role for that Execution. Record who or what actually did the work using `actor_kind` (`human`,
`agent`, or `tool`) and `actor` when known. Actor references are user-supplied labels, not verified
identity; never invent a person's identity or claim there is a current assignee.

Project Discovery is optional. Run `tramuri scan .` only when the user asks to inspect the project
or authorizes discovery; report whether it was complete or partial and include warnings.

At the end of a coherent work session that changed Mission records, run `tramuri status --open`
once to refresh and open the local report, unless the user asks not to. Explain that it writes
`.tramuri/reports/tramauri.html`, `tramauri.md`, and `tramauri.json` locally and opens the HTML.
Do not open a report just to read status; use `tramuri status` for that. Never run the report before
the workspace is initialized. End with a concise account of what was recorded, who or what executed
the work, the actual outcome state, and the next concrete action or unresolved decision. Do not
leave the user to ask what the normal next step is.

For scan safety and exclusions, read the installed Tramuri skill's
`references/project-discovery.md`.
