<!-- tramuri:managed-reference:v1 -->

# Assistant platform integrations

Tramuri can install a discoverable skill and a managed instruction section for supported platforms.
Use the installed CLI's help to confirm syntax. The current commands include:

```text
tramuri install --project --platform <codex|copilot|claude|gemini> --path <project>
tramuri integrations status --path <project>
tramuri uninstall --project --platform <codex|copilot|claude|gemini> --path <project>
```

The installer owns only its marked instruction section, skill file, and reference files. It should
preserve surrounding user instructions and refuse to overwrite unmarked files at managed paths. Do
not remove or rewrite user-authored files to repair an integration; inspect status and resolve the
conflict explicitly.

Integration instructions are convenience entry points, not runtime dependencies. Tramuri's core and
CLI remain independent of any assistant platform. Installing an integration does not install hooks
or grant an agent authority to export data.
