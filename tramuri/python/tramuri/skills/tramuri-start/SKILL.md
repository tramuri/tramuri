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
   Reuse a Mission automatically only when `tramuri start` reports one unique exact normalized
   objective match. Do not infer a match from broad topical similarity. If Tramuri reports
   `MissionSelectionRequired`, present the listed candidates and ask which one to resume; do not
   create another Mission or combine their context. For other relevant open Missions, resume the
   selected one with `tramuri resume --mission <id>`.
   If none matches, start a new Mission only after its work perspective is selected. Never choose an
   arbitrary Mission when several are plausible; ask which one to resume. Do not use `start --help`
   or a read-only query as a substitute for start/resume.
4. Confirm setup with `tramuri status`, then continue in the same interaction. The user may give
   only a natural-language objective; do not make them learn or repeat CLI syntax. Create an
   exploratory Mission with an empty Success Contract when the objective is clear but criteria are
   not. Refine criteria during Discovery instead of blocking the start.
5. A new Mission always requires the user's perspective choice. If the user has not already chosen
   one in the current request, ask this single-choice question before calling `start`:
   - **Product Engineer** — understands the product problem, helps decide what to build, and checks
     whether it worked.
   - **FDE** — solves a customer's need by connecting systems and validating the integration.
   - **Mixed work** — combines product evolution with a customer need.
   Do not infer the choice from the goal, a saved default, available skills, or the agent doing the
   work. Wait for the user's answer. In an interactive terminal, `tramuri start --goal "..."`
   displays the same three-option keyboard selector; in non-interactive use, ask in chat and pass
   `--perspective product_engineer`, `--perspective fde`, or `--perspective mixed`. The CLI rejects
   a new Mission without a selection and persists the selected perspective with the Mission in
   SQLite. Existing exact Mission matches resume with their recorded perspective without asking
   again. This perspective describes the work, not the human or agent identity. Never infer a
   person's identity from it.
6. Ask only what is needed for the next activity. Do not initiate Project Discovery for a new product
   idea unless the user asks to inspect an existing project. Record only authorized facts, decisions,
   questions, or constraints with provenance; do not turn the conversation into Knowledge wholesale.
   Required criteria may be added later. A Mission does not need Git, source code, a scan, an agent,
   or a cloud service.

After creating or resuming a Mission, load `$tramuri-product-engineering` when its perspective is
Product Engineer or the user wants to shape a product. It helps the agent do useful framing and
discovery before asking a material question. Do not load it for FDE-only work or one-shot questions.

For a Product Engineer Mission, `tramuri start` initializes the persisted DEFINE controller. Before
product discovery or project tools, call `tramuri mission define next --mission <id> --json`, load
the exact `skill` returned by the Core, produce that step's structured result, and submit it with
`tramuri mission define submit`. Repeat only after an accepted submission; a rejection keeps the
same step active. On resume, read `tramuri mission define status` and continue its current step.
Never infer completion from a narrative response or report a step complete until the CLI accepts it.
This is host guidance: the Core enforces state transitions, but cannot prove a host loaded a skill or
prevent arbitrary prose outside calls routed through Tramuri.

To change an accepted result, use the explicit `tramuri mission define review` operation. First
retrieve the current step and revision, then submit a replacement with a reason and declared author.
Do not edit SQLite or overwrite an artifact file as a substitute. The Core preserves prior revisions,
revalidates the step, and marks dependent steps stale when their accepted input revisions changed.
Re-run stale steps in order. Review of a Product Brief is distinct from semantic assessment:
submit a host-produced, rubric-versioned assessment with `review --assessment-file <file>
--evaluator <label>`. The Core checks its structure and references; it does not certify semantic
truth. After any write, the report files refresh best-effort without reopening the browser. A report
already open is a static snapshot; use its refresh control to reload the current files.

Keep work perspective separate from actor attribution. The perspective describes the professional
role for that Execution. Record who or what actually did the work using `actor_kind` (`human`,
`agent`, or `tool`) and `actor` when known. Actor references are user-supplied labels, not verified
identity; never invent a person's identity or claim there is a current assignee.

Project Discovery is optional. Run `tramuri scan .` only when the user asks to inspect the project
or authorizes discovery; report whether it was complete or partial and include warnings.

DEFINE start, accepted step submissions, and question transitions through the CLI refresh the local
report files best-effort without opening the browser. Do not run an extra refresh after each such
write. At the end of a coherent work session, if the HTML has not been opened in this session and
the user has not asked you to skip it, run `tramuri status --open`; otherwise the transition already
refreshed the files, so do not reopen an existing browser. Use `tramuri status --refresh` at the end
only when records were changed through an interface that does not refresh reports (for example,
direct Python SDK calls). Both commands write `.tramuri/reports/tramauri.html`, `tramauri.md`, and
`tramauri.json`; only `--open` launches the browser. Use `tramuri status` for a read-only status
check. Never refresh before the workspace is initialized. End with a concise account of what was
recorded, who or what executed the work, the actual outcome state, the next concrete action or
unresolved decision, and whether the local report was refreshed. Do not leave the user to ask what
the normal next step is.

For scan safety and exclusions, read the installed Tramuri skill's
`references/project-discovery.md`.
