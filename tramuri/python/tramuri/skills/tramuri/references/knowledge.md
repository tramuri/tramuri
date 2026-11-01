<!-- tramuri:managed-reference:v1 -->

# Persistent knowledge

Use Tramuri's Rust-backed SDK or CLI for knowledge operations when available. Do not create a
parallel knowledge store in assistant-specific files when Tramuri's supported operation covers the
task.

When recording or using a claim, preserve its kind and provenance:

- **Observed fact:** directly supported by a source or event.
- **Human decision:** an explicit choice made by an identified person.
- **Inference:** a conclusion derived from evidence; keep it labeled as an inference.
- **Unknown or unresolved:** information that has not been established.

Keep source paths, revision or content hashes, capture time, and freshness information when the
available API provides them. Do not upgrade summaries, inferences, stale records, or partial scans
into verified facts. If provenance is missing or a source changed, report that limitation.

Use the installed version's CLI help and SDK API documentation to determine which knowledge
operations are currently available; do not imply a planned operation already exists.

## Add and retrieve records

Initialize the local workspace first. For file-backed facts, scan the project so Tramuri can attach
current source provenance. Human decisions and open questions can be recorded without a scan.

```text
tramuri init .
tramuri scan .
tramuri knowledge add . --type fact --statement "The service uses SQLite" --source docs/architecture.md --json
tramuri knowledge add . --type decision --statement "Keep data local-first" --human-input --json
tramuri knowledge list . --json
tramuri knowledge list . --include-history --json
```

The Python SDK calls the same Rust-backed behavior:

```python
from pathlib import Path

import tramuri

workspace = Path(".")
tramuri.init(workspace)
record = tramuri.knowledge_add(
    workspace,
    type="decision",
    statement="Keep data local-first",
    human_input=True,
)
records = tramuri.knowledge_list(workspace, include_history=True)
tramuri.knowledge_withdraw(workspace, record["id"])
```

Use `type="fact"`, `"decision"`, `"inference"`, or `"question"`. Mark human input explicitly;
keep claims concise and non-sensitive. `sources` in the SDK accepts project-relative paths, for
example `sources=["docs/architecture.md"]`. Superseding and withdrawing records preserve history;
they do not erase prior records.
