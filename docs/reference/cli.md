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
- `memory show`, `memory search QUERY` and `memory save` inspect and update curated project memory.
- `explain [SUBJECT] [--run-id RUN_ID]` explains persisted product sources and connections without inventing facts.
- `product run --idea-file IDEA.md --mode poc|mvp|enterprise [--research-file PACK.json]` executes a workflow.
- `product research --run-id RUN_ID --research-file PACK.json` validates, persists and resumes a source-backed research handoff.
- `product demo --mode ...` executes the deterministic appointment fixture; it is explicitly synthetic.
- `report open RUN_ID --no-open` regenerates an offline report without launching a browser.
- `experience check RUN_ID` reruns demo, traceability, accessibility, offline and
  optional browser checks. It persists the current browser result, critical review,
  correction state and delivery decision in SQLite.

The delivery status is `completed` only when the product run, deterministic
checks, review and browser journey pass. If Playwright is unavailable while the
other checks pass, the delivery is `completed_with_browser_not_run`. A browser
journey or deterministic-check failure leaves delivery `blocked`; an executed
browser failure is reported as `failed`, not `not_run`.

JSON mode prints one JSON document and suppresses progress output so it is safe
for scripts.

## Status and resume

`tramuri status` is read-only. It selects the latest local run, or an explicit
`RUN_ID`, and reports the semantic state (`ready`, `running`, `waiting`,
`blocked`, `failed`, `stale` or `completed`), real stage progress, current and
last completed stage, persisted sensor gates, checkpoint availability and a
next action, research state/source count and selected agent execution mode.
`--json` exposes the same projection without terminal decoration;
`--no-color` keeps the human layout readable in plain terminals.

`tramuri resume RUN_ID` reuses the latest valid SQLite checkpoint and skips
verified stages. Checkpoints contain bounded state digests and pending work,
not raw prompts, tool arguments or credentials. A provider session can close
and reopen without depending on its conversation transcript.

## Project memory

`tramuri memory show` refreshes and displays a readable projection of saved
project memory and the latest workflow state. `tramuri memory search QUERY`
retrieves relevant active knowledge records, including their classification and
source. Add a concise durable fact, preference or decision with:

```bash
tramuri memory save --title "Launch region" --content "Start in Brazil" \
  --classification decision --source-ref user_request
```

Classifications are `fact`, `evidence`, `inference`, `hypothesis`, `assumption`,
`decision` and `pending_validation`. Save short, useful records rather than
conversation transcripts or credentials. SQLite remains canonical; readable
projections are regenerated under `tramuri_output/memorys/`.

Product commands return report, portal, demo, review and sensor paths/results;
generated Tramuri-owned files are placed under `tramuri_output/`.
The Product Experience is a local synthetic exploration and is intentionally
distinct from Product Engineering. A missing browser runtime is `not_run`, not
success.
