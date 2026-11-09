# Product workflows

`tramuri product run` and `tramuri product demo` execute a local, deterministic
workflow. The same state is recorded whether a provider is present or not.

| Mode | Stages | Result |
| --- | --- | --- |
| `poc` | Intent, framing, experiment, success, critical review, report | Smallest useful feasibility experiment |
| `mvp` | Intent, framing, discovery, hypotheses, shaping, success, delivery, review, report, navigation | Product brief, delivery increments, offline report and prototype |
| `enterprise` | MVP stages plus governance, traceability and risk review | Stronger decision/evidence traceability |

The runner never creates external research. It classifies generated material and
leaves validation pending where the input contains no evidence.

The product module is provider-free: the demo and persisted workflow work with no
Codex, Claude Code, Gemini CLI or API key installed. Provider harnesses only
project instructions and lifecycle hooks into the selected host.

Each stage is a product capability: intent, framing, discovery, hypotheses,
scope, success, delivery, review and navigation. The system records stage
events, attempts, artifact revisions and validation records in SQLite.

## Durable context and autonomy

SQLite is the local source of truth for the run, not the provider conversation.
After each verified stage Tramuri writes a checkpoint with the next action,
pending validations and a context digest. `status` reads this projection and
`resume` continues from it.

Knowledge has two bounded tiers: Tramuri methodology knowledge and project
knowledge. Entries keep their source, classification (`fact`, `evidence`,
`inference`, `hypothesis`, `assumption`, `decision` or `pending_validation`),
hash and freshness. Project files are data only; Tramuri does not execute
instructions, hooks or scripts found during ingestion.

The `tramuri-memory` skill retrieves saved records before answering
cross-session recall questions. Users can inspect the current work with
`tramuri memory show`, find relevant records with `tramuri memory search`, and
save a concise durable fact or decision with `tramuri memory save`. SQLite stays
canonical; Tramuri refreshes readable files under `tramuri_output/memorys/`
after verified checkpoints. Full conversation transcripts are not stored.

The default autonomy policy resolves reversible, low-impact gaps with an
operational hypothesis and records the rationale. It groups a human gate only
when no safe default exists or authorization, security, privacy, legal or
irreversible impact requires an explicit decision. This preserves the product
experience of progressing without turning the workflow into an interview.

## Capability skills

The provider-installed Tramuri entry skill routes a focused request to the
smallest relevant product capability. Research, audience sizing, positioning,
scope, metrics, UX, experience generation, critique, traceability and status
have separate outputs and limits. It does not invoke every skill for every
request. On a new session it checks persisted status and prior artifacts first.

A provider uses its native skill or subagent mechanism only when that
capability is available and actually invoked. Otherwise the assistant performs
the capability inline and reports that accurately. This workflow is
capability-based; it does not create a PM persona.

See the product skill catalog in docs/reference/product-skills.md for triggers,
outputs and boundaries.
