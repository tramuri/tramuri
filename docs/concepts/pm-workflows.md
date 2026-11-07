# Product Management workflows

`tramuri pm run` and `tramuri pm demo` execute a local, deterministic workflow.
The same state is recorded whether a provider is present or not.

| Mode | Stages | Result |
| --- | --- | --- |
| `poc` | Intent, framing, experiment, success, critical review, report | Smallest useful feasibility experiment |
| `mvp` | Intent, framing, discovery, hypotheses, shaping, success, delivery, review, report, navigation | Product brief, delivery increments, offline report and prototype |
| `enterprise` | MVP stages plus governance, traceability and risk review | Stronger decision/evidence traceability |

The runner never creates external research. It classifies generated material and
leaves validation pending where the input contains no evidence.

Each stage has an owner role: Product Strategist, Discovery Specialist, Product
Analyst, Scope Specialist or Critical Reviewer. The system records stage events,
attempts, artifact revisions and validation records in SQLite.
