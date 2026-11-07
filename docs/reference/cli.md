# CLI reference

Global options can appear before a command:

```text
tramuri [--project-dir PATH] [--json] [--no-color] COMMAND
```

Commands:

- `init --provider auto|all|codex|claude|gemini [--dry-run]` detects and installs project-local components.
- `doctor` and `providers list` show provider detection and capability status.
- `install`, `update` and `uninstall` manage provider manifests safely.
- `status`, `runs list`, `runs show RUN_ID` and `resume RUN_ID` inspect persisted state.
- `pm run --idea-file IDEA.md --mode poc|mvp|enterprise` executes a workflow.
- `pm demo --mode ...` executes the deterministic scheduling example.
- `report open RUN_ID --no-open` regenerates an offline report without launching a browser.

JSON mode prints one JSON document and suppresses progress output so it is safe
for scripts.
