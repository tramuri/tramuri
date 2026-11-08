# CLI reference

Global options can appear before a command:

```text
tramuri [--project-dir PATH] [--json] [--no-color] COMMAND
```

Commands:

- `init --provider auto|all|codex|claude|gemini [--dry-run]` detects and installs project-local components.
- `doctor` and `providers list` show provider detection and capability status.
- `install`, `update` and `uninstall` manage provider manifests safely.
- `status [RUN_ID]`, `runs list`, `runs show RUN_ID` and `resume RUN_ID` inspect persisted state.
- `explain [SUBJECT] [--run-id RUN_ID]` explains persisted product sources and connections without inventing facts.
- `product run --idea-file IDEA.md --mode poc|mvp|enterprise` executes a workflow.
- `product demo --mode ...` executes the deterministic scheduling example.
- `report open RUN_ID --no-open` regenerates an offline report without launching a browser.
- `experience check RUN_ID` validates demo files, links, screens, interactions,
  traceability, accessibility and offline behavior; it also reports optional
  browser validation.
- `experience skills TASK_KIND` shows the contextual harness skill selection
  when the local development kit is available.

JSON mode prints one JSON document and suppresses progress output so it is safe
for scripts.

## Status and resume

`tramuri status` is read-only. It selects the latest local run, or an explicit
`RUN_ID`, and reports the semantic state (`ready`, `running`, `waiting`,
`blocked`, `failed`, `stale` or `completed`), real stage progress, current and
last completed stage, persisted sensor gates, checkpoint availability and a
next action. `--json` exposes the same projection without terminal decoration;
`--no-color` keeps the human layout readable in plain terminals.

`tramuri resume RUN_ID` reuses the latest valid SQLite checkpoint and skips
verified stages. Checkpoints contain bounded state digests and pending work,
not raw prompts, tool arguments or credentials. A provider session can close
and reopen without depending on its conversation transcript.

Product commands return report, portal, demo, review and sensor paths/results;
generated Tramuri-owned files are placed under `tramuri_output/`.
The Product Demo is a local synthetic exploration and is intentionally distinct
from Product Engineering.
