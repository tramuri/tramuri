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

The default autonomy policy resolves reversible, low-impact gaps with an
operational hypothesis and records the rationale. It groups a human gate only
when no safe default exists or authorization, security, privacy, legal or
irreversible impact requires an explicit decision. This preserves the product
experience of progressing without turning the workflow into an interview.
