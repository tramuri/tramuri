---
name: tramuri-start
description: Start or resume a Tramuri Mission whenever the user describes substantive new project, product, feature, integration, deployment, or handoff work, even without naming Tramuri; also use for setup questions.
---

<!-- tramuri:managed-skill:v1 -->

# Start using Tramuri

Use this skill for substantive new or continuing work in an integrated Tramuri workspace, even when
the user does not mention Tramuri. Product creation, a new feature, a customer integration,
deployment, or multi-step handoff starts or resumes a Mission. The user describes the goal in
natural language; never require CLI syntax or ask whether they want Tramuri when its project
integration is installed. Do not create a Mission for a simple one-shot question.

1. Check whether `tramuri` is installed (`tramuri --version`) and whether the selected workspace is
   initialized (`tramuri status`). Do not run a scan just to start a Mission.
   This check and start/resume are the first project-work actions; do not inspect files, load an
   unrelated workflow skill, or invoke another product/build tool first. Once Tramuri has established
   the Mission context, continue the user's requested work using the appropriate tools.
2. If Tramuri is missing, explain the install command for the current platform and do not pretend it
   ran. If the workspace is uninitialized, a clear substantive request in this opted-in project
   authorizes starting the Mission. Run `tramuri start --goal "..."` directly; it initializes local
   state only after a valid goal and perspective are resolved. Do not ask whether to use Tramuri or
   initialize, and do not initialize for one-shot questions.
3. Confirm setup with `tramuri status`, then continue in the same interaction. The user may give
   only a natural-language objective; do not make them learn or repeat CLI syntax. Create an
   exploratory Mission with an empty Success Contract when the objective is clear but criteria are
   not. Refine criteria during Discovery instead of blocking the start.
4. For a new substantive objective, use `tramuri start --goal "..."` through the native CLI, even
   when the user did not explicitly name Tramuri. Let the Core route clear current-intent phrases
   before the saved profile. Pass `--perspective` only when the user explicitly selected a perspective
   or confirmed it after clarification; otherwise do not encode an agent inference as explicit choice.
   If the objective or perspective remains materially ambiguous, ask one short question before starting.
   The CLI records an exploratory Mission, a transparent perspective source,
   and planned `discover` activity. Do not invent a complete Success Contract or mark readiness.
   Route from clear current intent before the saved preference; if the objective is materially
   ambiguous, ask one short question before starting. Treat routing as a deterministic suggestion,
   not general semantic understanding. Never infer a person's identity from perspective.
5. Ask only what is needed for the next activity. Do not initiate Project Discovery for a new product
   idea unless the user asks to inspect an existing project. Record only authorized facts, decisions,
   questions, or constraints with provenance; do not turn the conversation into Knowledge wholesale.
6. Create the Mission once its objective is clear; required criteria may be added later. A Mission
   does not need Git, source code, a scan, an agent, or a cloud service.

After creating or resuming a Mission, load `$tramuri-product-engineering` when its perspective is
Product Engineer or the user wants to shape a product. It helps the agent do useful framing and
discovery before asking a material question. Do not load it for FDE-only work or one-shot questions.

Keep work perspective separate from actor attribution. The perspective describes the professional
role for that Execution. Record who or what actually did the work using `actor_kind` (`human`,
`agent`, or `tool`) and `actor` when known. Actor references are user-supplied labels, not verified
identity; never invent a person's identity or claim there is a current assignee.

Project Discovery is optional. Run `tramuri scan .` only when the user asks to inspect the project
or authorizes discovery; report whether it was complete or partial and include warnings.

At the end of a coherent work session that changed Mission records, refresh the local reports once.
If the HTML has not been opened in this session and the user has not asked you to skip it, run
`tramuri status --open`; otherwise run `tramuri status --refresh` so the files include the latest
records without reopening the browser. Do not generate reports at Mission start or after every
individual write. Explain that both commands write `.tramuri/reports/tramauri.html`, `tramauri.md`,
and `tramauri.json`; only `--open` launches the browser. Use `tramuri status` for a read-only status
check. Never refresh before the workspace is initialized. End with a concise account of what was
recorded, who or what executed the work, the actual outcome state, the next concrete action or
unresolved decision, and whether the local report was refreshed. Do not leave the user to ask what
the normal next step is.

For scan safety and exclusions, read the installed Tramuri skill's
`references/project-discovery.md`.
