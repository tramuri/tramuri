---
name: tramuri-start
description: Start or resume a Tramuri Mission whenever the user describes substantive new project, product, feature, integration, deployment, or handoff work, even without naming Tramuri; also use for setup questions.
---

<!-- tramuri:managed-skill:v1 -->

# Start using Tramuri

In installed workspaces, native project hooks enforce this sequence before local project tools run.
While locked, the hook also permits a narrow set of non-Mission-writing Tramuri preflight commands:
`tramuri --version`, `tramuri status [--json]`, `tramuri mission list [--json]`, and direct help
queries. These commands do not unlock the gate. Only a direct `tramuri start ...` or
`tramuri resume ...` command that finishes successfully unlocks project tools for the current
prompt/turn. If a hook blocks an action, follow its instruction; do not work around the gate. The
hooks inspect only platform metadata and the matched start/resume tool result; they do not read or
persist prompt text, Mission records, or project files. If the host disables or does not trust
project hooks, follow this skill as fallback guidance.

Use this skill for substantive new or continuing work in an integrated Tramuri workspace, even when
the user does not mention Tramuri. Product creation, a new feature, a customer integration,
deployment, or multi-step handoff starts or resumes a Mission. The user describes the goal in
natural language; never require CLI syntax or ask whether they want Tramuri when its project
integration is installed. Do not create a Mission for a simple one-shot question.

1. Run the direct preflight commands `tramuri --version`, `tramuri status`, and
   `tramuri mission list` as needed to identify the installed CLI, workspace state, and existing
   Missions. They do not create or change Mission/workflow records and do not unlock the gate.
   Opening an older database can run the CLI's existing additive migrations. Do not run a scan just
   to start a Mission. These checks
   are the first project-work actions; do not inspect files, load an unrelated workflow skill, or
   invoke another product/build tool first. Once Tramuri has established Mission context, continue
   the user's requested work using the appropriate tools.
2. If Tramuri is missing, explain the install command for the current platform and do not pretend it
   ran. If the workspace is uninitialized, a clear substantive request in this opted-in project
   authorizes starting the Mission. Run `tramuri start --goal "..."` directly; it initializes local
   state only after a valid goal and perspective are resolved. Do not ask whether to use Tramuri or
   initialize, and do not initialize for one-shot questions.
3. In an initialized workspace, match the request against the Missions from `tramuri mission list`.
   If a relevant open Mission exists, resume it directly with `tramuri resume --mission <id>`.
   If none matches, start a new Mission with `tramuri start --goal "..."`. Never choose an arbitrary
   Mission when several are plausible; ask one brief question only if the choice materially changes
   the work. Do not use `start --help` or a read-only query as a substitute for start/resume.
4. Confirm setup with `tramuri status`, then continue in the same interaction. The user may give
   only a natural-language objective; do not make them learn or repeat CLI syntax. Create an
   exploratory Mission with an empty Success Contract when the objective is clear but criteria are
   not. Refine criteria during Discovery instead of blocking the start.
5. Let the Core route clear current-intent phrases
   before the saved profile. Pass `--perspective` only when the user explicitly selected a perspective
   or confirmed it after clarification; otherwise do not encode an agent inference as explicit choice.
   If the objective or perspective remains materially ambiguous, ask one short question before starting.
   The CLI records an exploratory Mission, a transparent perspective source,
   and planned activity. Product Engineer Missions start at `product_framing`; FDE and other clear
   non-ambiguous perspectives start at `discover`. Do not invent a complete Success Contract or
   mark readiness.
   Route from clear current intent before the saved preference; if the objective is materially
   ambiguous, ask one short question before starting. Treat routing as a deterministic suggestion,
   not general semantic understanding. Never infer a person's identity from perspective.
6. Ask only what is needed for the next activity. Do not initiate Project Discovery for a new product
   idea unless the user asks to inspect an existing project. Record only authorized facts, decisions,
   questions, or constraints with provenance; do not turn the conversation into Knowledge wholesale.
   Required criteria may be added later. A Mission does not need Git, source code, a scan, an agent,
   or a cloud service.

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
