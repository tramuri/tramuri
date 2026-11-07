# Run the offline demo

The demo is deterministic and does not call Codex, Claude, Gemini or a hosted
LLM. Use a temporary project directory when experimenting:

```powershell
tramuri --project-dir .tramuri-demo pm demo --mode mvp
tramuri --project-dir .tramuri-demo runs list
```

The command writes local state to `.tramuri-demo/.tramuri/` and creates:

- `reports/<run-id>/report.html`, a navigable PM report;
- `reports/<run-id>/prototype.html`, a scoped offline prototype;
- `database/tramuri.sqlite3`, the source of run state and evidence references.

Use `--json` for automation. The demo output is synthetic and must not be
presented as user research or market evidence.
