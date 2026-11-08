# Run the offline demo

The demo is deterministic and does not call Codex, Claude, Gemini or a hosted
LLM. Use a temporary project directory when experimenting:

```powershell
tramuri --project-dir .tramuri-demo product demo --mode mvp
tramuri --project-dir .tramuri-demo runs list
tramuri --project-dir .tramuri-demo status
```

The command writes local state to `.tramuri-demo/tramuri_output/` and creates:

- `reports/<run-id>/report.html`, a navigable product report;
- `reports/<run-id>/prototype.html`, a scoped offline prototype;
- `portal/index.html`, the four-stage Product Experience map;
- `demo/index.html`, `demo/styles.css`, `demo/app.js` and
  `demo/traceability.json`, the offline Product Demo projection;
- `database/tramuri.sqlite3`, the source of run state and evidence references.

Use `--json` for automation. The demo output is synthetic and must not be
presented as user research or market evidence. Validate its deterministic
sensors with `tramuri --project-dir .tramuri-demo --json experience check RUN_ID`.
If Playwright is unavailable, the result reports browser validation as
`not_run`; that is a limitation, not a browser pass.

## Reopen a workflow

The workflow state is not held only in the terminal conversation. Inspect the
latest checkpoint and resume the same run after closing the provider session:

```powershell
tramuri --project-dir .tramuri-demo --json status
tramuri --project-dir .tramuri-demo runs show RUN_ID
tramuri --project-dir .tramuri-demo resume RUN_ID
```

`status` is read-only and shows the next action, persisted sensor gates and
pending validation. Low-impact product gaps are recorded as hypotheses; only
authorization, security, privacy, legal or irreversible choices become human
gates.
